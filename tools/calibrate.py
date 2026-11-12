#!/usr/bin/env python3
"""Threat calibration: predicted damage against damage taken (T4.7, #124).

  tools/calibrate.py DIR [DIR ...] [--md FILE] [--json FILE] [--seed N]

Each DIR is a run directory (it has summary.json, trace.jsonl, state.log)
or a suite output holding run directories. For every run, each think that
took game time is a *window*: from its game turn to the next think's later
turn. The think's trace line carries the assessment at the character's
grid over `threat_horizon` player turns (expected and worst hit points,
p_death, and from T4.7 the contributing monsters, `threat.sources`). The
state log gives the hit points the character actually lost in the window:
the sum of every drop between consecutive state lines (healing is not
netted against it). A run that died adds its last drop to the final
`summary.json` hit points.

What a window does not control, and the report says so:
* the prediction is for *standing on the grid*; the Borg acts in the
  window (attacks, steps away, drinks), so the outcome is conditioned on
  its choice (reported by choice kind);
* the window is one think's time, which is the declared horizon only at
  normal speed (the window length in game turns is kept);
* damage from no monster (hunger, poison and cut ticks, traps) lands in
  windows with no source and is reported apart, as unattributed.

Uncertainty is a percentile bootstrap over windows (deterministic seed),
which treats windows as independent; they are not (windows of one fight
are correlated), so the intervals are narrower than the truth and the
report says so.
"""

import argparse
import json
import random
import re
import sys
from pathlib import Path

STATE_HP = re.compile(r'^(-?\d+) .*? hp(-?\d+)/(-?\d+) ')
BINS = [0.0, 0.5, 1.0, 2.0, 4.0, 8.0, float('inf')]
PDEATH_BINS = [0.0, 0.001, 0.01, 0.05, 0.2, 1.0000001]


def find_runs(paths):
    runs = []
    for p in paths:
        p = Path(p)
        if (p / 'summary.json').is_file():
            runs.append(p)
            continue
        runs.extend(sorted(d for d in p.iterdir()
                           if d.is_dir() and (d / 'summary.json').is_file()))
    return runs


def read_state(path):
    """(turn, hp) per state line, in order."""
    out = []
    with open(path) as f:
        for line in f:
            m = STATE_HP.match(line)
            if m:
                out.append((int(m.group(1)), int(m.group(2))))
    return out


def read_messages(path):
    """(turn, text) per message line; the log is 'key turn type text'."""
    out = []
    if not path.is_file():
        return out
    with open(path, errors='replace') as f:
        for line in f:
            parts = line.rstrip('\n').split(' ', 3)
            if len(parts) == 4 and parts[1].lstrip('-').isdigit():
                text = parts[3].strip()
                if text:
                    out.append((int(parts[1]), text))
    return out


def choice_kind(choice):
    return re.split(r'[ ({]', choice, maxsplit=1)[0]


def windows(run):
    """The run's windows, as dicts."""
    summary = json.loads((run / 'summary.json').read_text())
    state = read_state(run / 'state.log')
    messages = read_messages(run / 'messages.log')
    thinks = []
    with open(run / 'trace.jsonl') as f:
        for line in f:
            t = json.loads(line)
            if thinks and thinks[-1]['turn'] == t['turn']:
                thinks[-1] = t  # the last think at a turn is the one that acted
            else:
                thinks.append(t)
    end_turn = summary['game_turns']
    dead = bool(summary.get('dead'))
    out = []
    si = 0
    mi = 0
    last_hp = state[0][1] if state else 0
    for i, t in enumerate(thinks):
        start = t['turn']
        stop = thinks[i + 1]['turn'] if i + 1 < len(thinks) else end_turn
        while si < len(state) and state[si][0] <= start:
            last_hp = state[si][1]
            si += 1
        hp_start = last_hp
        damage = 0
        while si < len(state) and state[si][0] <= stop:
            damage += max(0, last_hp - state[si][1])
            last_hp = state[si][1]
            si += 1
        last = i + 1 == len(thinks)
        died = dead and last
        if died:
            damage += max(0, last_hp - summary['hp'])
        while mi < len(messages) and messages[mi][0] <= start:
            mi += 1
        msgs = []
        j = mi
        while j < len(messages) and messages[j][0] <= stop:
            msgs.append(messages[j][1])
            j += 1
        threat = t.get('threat') or {}
        out.append({
            'run': run.name,
            'turn': start,
            'length': stop - start,
            'hp': hp_start,
            'choice': t.get('choice', ''),
            'kind': choice_kind(t.get('choice', '')),
            'mode': t.get('mode', ''),
            'expected': float(threat.get('expected', 0.0)),
            'worst': float(threat.get('worst', 0.0)),
            'p_death': float(threat.get('p_death', 0.0)),
            'sources': threat.get('sources', []),
            'actual': damage,
            'died': died,
            'messages': msgs,
        })
    return out


def ratio(ws):
    pred = sum(w['expected'] for w in ws)
    return sum(w['actual'] for w in ws) / pred if pred > 0 else None


def bootstrap(ws, rng, reps=1000):
    """90% percentile interval of actual/predicted over resampled windows."""
    if len(ws) < 2:
        return None
    vals = []
    n = len(ws)
    for _ in range(reps):
        s = [ws[rng.randrange(n)] for _ in range(n)]
        r = ratio(s)
        if r is not None:
            vals.append(r)
    if not vals:
        return None
    vals.sort()
    return vals[int(0.05 * (len(vals) - 1))], vals[int(0.95 * (len(vals) - 1))]


def group_stats(ws, rng):
    n = len(ws)
    pred = sum(w['expected'] for w in ws)
    actual = sum(w['actual'] for w in ws)
    exceed = sum(1 for w in ws if w['actual'] > w['worst'] + 1e-9)
    return {
        'windows': n,
        'predicted': round(pred, 2),
        'actual': actual,
        'ratio': None if pred <= 0 else round(actual / pred, 3),
        'ci90': (lambda c: None if c is None else [round(c[0], 3), round(c[1], 3)])(
            bootstrap(ws, rng)),
        'hit_rate': round(sum(1 for w in ws if w['actual'] > 0) / n, 3) if n else None,
        'over_worst': exceed,
    }


def dist_band(d):
    return '1' if d <= 1 else '2-3' if d <= 3 else '4-7' if d <= 7 else '8+'


def speed_band(s):
    return 'slow (<110)' if s < 110 else 'normal (110)' if s == 110 else 'fast (>110)'


def status_of(s):
    if not s.get('awake', True):
        return 'asleep'
    if s.get('confused') or s.get('stunned'):
        return 'confused/stunned'
    if not s.get('in_view', True):
        return 'remembered, out of view'
    return 'awake, in view'


def analyse(ws, seed=0):
    rng = random.Random(seed)
    threatened = [w for w in ws if w['expected'] > 0 or w['worst'] > 0]
    quiet = [w for w in ws if not (w['expected'] > 0 or w['worst'] > 0)]
    single = [w for w in threatened if len(w['sources']) == 1]
    rep = {
        'windows': len(ws),
        'threatened': group_stats(threatened, rng) if threatened else None,
        'quiet': {
            'windows': len(quiet),
            'with_damage': sum(1 for w in quiet if w['actual'] > 0),
            'damage': sum(w['actual'] for w in quiet),
            'deaths': sum(1 for w in quiet if w['died']),
        },
        'deaths': sum(1 for w in ws if w['died']),
    }

    bins = []
    for lo, hi in zip(BINS, BINS[1:]):
        b = [w for w in threatened if lo <= w['expected'] < hi]
        if b:
            bins.append({'range': [lo, hi], 'mean_predicted': round(
                sum(w['expected'] for w in b) / len(b), 3),
                'mean_worst': round(sum(w['worst'] for w in b) / len(b), 3),
                'mean_actual': round(sum(w['actual'] for w in b) / len(b), 3),
                **group_stats(b, rng)})
    rep['bins'] = bins

    pd = []
    for lo, hi in zip(PDEATH_BINS, PDEATH_BINS[1:]):
        b = [w for w in ws if lo <= w['p_death'] < hi]
        if b:
            pd.append({'range': [lo, min(hi, 1.0)], 'windows': len(b),
                       'mean_p_death': round(sum(w['p_death'] for w in b) / len(b), 4),
                       'deaths': sum(1 for w in b if w['died'])})
    rep['p_death'] = pd
    rep['brier'] = round(sum((w['p_death'] - (1.0 if w['died'] else 0.0)) ** 2
                             for w in ws) / len(ws), 6) if ws else None

    def by(key, pool):
        groups = {}
        for w in pool:
            groups.setdefault(key(w), []).append(w)
        return {k: group_stats(v, rng) for k, v in sorted(groups.items())}

    rep['single_source'] = group_stats(single, rng) if single else None
    rep['by_monster'] = by(lambda w: w['sources'][0]['name'], single)
    rep['by_distance'] = by(lambda w: dist_band(w['sources'][0]['dist']), single)
    rep['by_speed'] = by(lambda w: speed_band(w['sources'][0]['speed']), single)
    rep['by_status'] = by(lambda w: status_of(w['sources'][0]), single)
    rep['by_attack'] = by(lambda w: 'ranged estimate' if w['sources'][0]['ranged']
                          else 'melee only', single)
    rep['by_sources'] = by(lambda w: '1' if len(w['sources']) == 1 else
                           '2' if len(w['sources']) == 2 else
                           '3+' if w['sources'] else '0 (no detail)', threatened)
    rep['by_group'] = by(lambda w: 'alone' if max((s['group'] for s in w['sources']),
                                                    default=1) <= 1 else 'same-race group',
                         threatened)
    rep['by_choice'] = by(lambda w: w['kind'], threatened)

    def brief(w):
        return {'run': w['run'], 'turn': w['turn'], 'hp': w['hp'],
                'expected': round(w['expected'], 2), 'worst': round(w['worst'], 2),
                'p_death': round(w['p_death'], 4), 'actual': w['actual'],
                'died': w['died'], 'choice': w['choice'],
                'sources': [f"{s['name']} d{s['dist']} sp{s['speed']}"
                            f"{'' if s.get('awake', True) else ' asleep'}"
                            f"{' x' + str(s['group']) if s['group'] > 1 else ''}"
                            for s in w['sources']],
                'messages': w['messages'][:6]}

    over = sorted((w for w in threatened if w['actual'] > w['worst'] + 1e-9),
                  key=lambda w: (w['worst'] - w['actual'], w['run'], w['turn']))
    rep['outliers_over_worst'] = [brief(w) for w in over[:15]]
    unpred = sorted((w for w in quiet if w['actual'] > 0),
                    key=lambda w: (-w['actual'], w['run'], w['turn']))
    rep['outliers_unattributed'] = [brief(w) for w in unpred[:15]]
    rep['deaths_detail'] = [brief(w) for w in ws if w['died']]
    return rep


def fmt(v):
    if v is None:
        return '–'
    if isinstance(v, float):
        return f'{v:.3g}'
    return str(v)


def table(rows, title):
    lines = [f'| {title} | windows | predicted | actual | actual/predicted (90% CI) '
             '| hit rate | over worst |', '| --- | ---: | ---: | ---: | --- | ---: | ---: |']
    for k, s in rows.items():
        ci = '' if not s['ci90'] else f" ({s['ci90'][0]:.2f}–{s['ci90'][1]:.2f})"
        lines.append(f"| {k} | {s['windows']} | {s['predicted']:.1f} | {s['actual']} "
                     f"| {fmt(s['ratio'])}{ci} | {fmt(s['hit_rate'])} | {s['over_worst']} |")
    return '\n'.join(lines)


def markdown(rep):
    out = [f"Windows: {rep['windows']}; threatened (expected or worst > 0): "
           f"{rep['threatened']['windows'] if rep['threatened'] else 0}; "
           f"deaths: {rep['deaths']}.", '']
    if rep['threatened']:
        out += [table({'all threatened': rep['threatened'],
                       **({'one source': rep['single_source']}
                          if rep['single_source'] else {})}, 'pool'), '']
    q = rep['quiet']
    out += [f"Quiet windows (no prediction): {q['windows']}, {q['with_damage']} with damage, "
            f"{q['damage']} HP in all, {q['deaths']} deaths (unattributed).", '']
    out += ['**By predicted expected damage**', '',
            '| expected bin | windows | mean expected | mean worst | mean actual '
            '| actual/predicted (90% CI) | hit rate | over worst |',
            '| --- | ---: | ---: | ---: | ---: | --- | ---: | ---: |']
    for b in rep['bins']:
        ci = '' if not b['ci90'] else f" ({b['ci90'][0]:.2f}–{b['ci90'][1]:.2f})"
        out.append(f"| [{b['range'][0]:g}, {b['range'][1]:g}) | {b['windows']} "
                   f"| {b['mean_predicted']:.2f} | {b['mean_worst']:.2f} "
                   f"| {b['mean_actual']:.2f} | {fmt(b['ratio'])}{ci} "
                   f"| {fmt(b['hit_rate'])} | {b['over_worst']} |")
    out += ['', '**p_death against deaths in the window** (Brier score '
            f"{fmt(rep['brier'])})", '', '| p_death bin | windows | mean p_death | deaths |',
            '| --- | ---: | ---: | ---: |']
    for b in rep['p_death']:
        out.append(f"| [{b['range'][0]:g}, {b['range'][1]:g}) | {b['windows']} "
                   f"| {b['mean_p_death']:.4f} | {b['deaths']} |")
    for key, title in [('by_distance', 'distance (one source)'),
                       ('by_speed', 'monster speed (one source)'),
                       ('by_status', 'monster status (one source)'),
                       ('by_attack', 'attack form (one source)'),
                       ('by_sources', 'sources in the window'),
                       ('by_group', 'same-race group'),
                       ('by_choice', "the Borg's choice")]:
        if rep[key]:
            out += ['', table(rep[key], title)]
    mons = {k: v for k, v in rep['by_monster'].items() if v['windows'] >= 5}
    if mons:
        out += ['', table(dict(sorted(mons.items(), key=lambda kv: -kv[1]['windows'])),
                          'monster (one source, ≥5 windows)')]
    return '\n'.join(out) + '\n'


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    ap.add_argument('dirs', nargs='+')
    ap.add_argument('--md')
    ap.add_argument('--json')
    ap.add_argument('--seed', type=int, default=0)
    a = ap.parse_args(argv)
    runs = find_runs(a.dirs)
    if not runs:
        print('calibrate: no runs found', file=sys.stderr)
        return 1
    ws = []
    missing = []
    for r in runs:
        if not (r / 'trace.jsonl').is_file() or not (r / 'state.log').is_file():
            missing.append(r.name)
            continue
        ws.extend(windows(r))
    rep = analyse(ws, a.seed)
    rep['runs'] = len(runs) - len(missing)
    rep['runs_without_logs'] = missing
    if a.json:
        Path(a.json).write_text(json.dumps(rep, indent=1) + '\n')
    text = markdown(rep)
    if a.md:
        Path(a.md).write_text(text)
    else:
        sys.stdout.write(text)
    return 0


if __name__ == '__main__':
    sys.exit(main())
