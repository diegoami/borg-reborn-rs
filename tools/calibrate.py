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
DISABLE_BINS = [0.0, 0.01, 0.05, 0.2, 0.5, 1.0000001]
# Onset texts of the four disabling effects (lib/gamedata/player_timed.txt:
# BLIND, PARALYZED, CONFUSED, AFRAID, on "grade" and "on-increase").
DISABLED = ('You are blind.', 'You are paralysed!', 'You are confused!',
            'You are more confused!', 'You are terrified!', 'You are more scared!')
# Choices after which the character is still on the assessed grid; a Walk,
# a staircase or Alter can end the window elsewhere, and a Read may be a
# Phase Door or a Teleportation (the escape ladder's own relocations).
STANDING = {'Attack', 'Hold', 'Rest', 'Quaff', 'Eat', 'Cast', 'Shoot',
            'Throw', 'Use', 'Wield', 'Get', 'Buy', 'Sell'}
FAINT = 'You faint from the lack of food.'
ROOT = Path(__file__).resolve().parent.parent
MONSTER_TXT = ROOT / 'engine/angband/lib/gamedata/monster.txt'


def read_blows(path=MONSTER_TXT):
    """Race name -> [(method, effect, mean damage)] from monster.txt."""
    out = {}
    name = None
    if not Path(path).is_file():
        return out
    with open(path) as f:
        lines = f.read().splitlines()
    for line in lines:
        if line.startswith('name:'):
            name = line[5:].strip()
            out[name] = []
        elif line.startswith('blow:') and name:
            parts = line.strip().split(':')
            mean = 0.0
            if len(parts) > 3:
                m = re.match(r'(\d+)d(\d+)', parts[3])
                if m:
                    mean = int(m.group(1)) * (int(m.group(2)) + 1) / 2
            out[name].append((parts[1], parts[2] if len(parts) > 2 else '', mean))
    return out


def ended(name, msgs):
    """Whether the monster died or fled in the window (it may not act)."""
    return any(f'slain the {name}' in m or f'destroyed the {name}' in m
               or (m.startswith((f'The {name} ', f'{name} ', f'{name}, '))
                   and ('flees in terror' in m or 'dies' in m))
               for m in msgs)


def blow_messages(name, msgs):
    """(hits, misses) the engine printed for this monster's blows."""
    heads = (f'The {name} ', f'{name} ', f'{name}, ')
    hits = misses = 0
    for m in msgs:
        if not m.startswith(heads) or ' you' not in m:
            continue
        if m.endswith('misses you.'):
            misses += 1
        else:
            hits += 1
    return hits, misses
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
    """(key, text) per message line. The log is 'key turn type text', where
    key is the state-log line (1-based) whose key's processing printed it:
    an action's own messages and the monster turns that follow it, up to
    the next key, carry that key's number."""
    out = []
    if not path.is_file():
        return out
    with open(path, errors='replace') as f:
        for line in f:
            parts = line.rstrip('\n').split(' ', 3)
            if len(parts) == 4 and parts[0].isdigit():
                text = parts[3].strip()
                if text:
                    out.append((int(parts[0]), text))
    return out


def choice_kind(choice):
    return re.split(r'[ ({]', choice, maxsplit=1)[0]


def windows(run):
    """The run's windows, as dicts.

    A think at game turn T is answered by the keys logged at T; its window
    runs from its first key to the first key of the next think at a later
    turn. Monsters with more energy than the character act at that later
    turn *before* its first key (game-world.c:run_game_loop, "Process
    monster with even more energy first"), so the window ends there, not at
    a turn boundary: hit points are read at both ends, and messages are
    assigned by key."""
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

    def first_key(turn, lo):
        """Index (0-based) of the first state line at or after `turn`."""
        while lo < len(state) and state[lo][0] < turn:
            lo += 1
        return lo

    out = []
    mi = 0
    k0 = first_key(thinks[0]['turn'], 0) if thinks else 0
    for i, t in enumerate(thinks):
        start = t['turn']
        last = i + 1 == len(thinks)
        stop = end_turn if last else thinks[i + 1]['turn']
        k1 = len(state) if last else first_key(stop, k0)
        hp_start = state[k0][1] if k0 < len(state) else (state[-1][1] if state else 0)
        damage = 0
        prev = hp_start
        for k in range(k0 + 1, min(k1 + 1, len(state))):
            damage += max(0, prev - state[k][1])
            prev = state[k][1]
        died = dead and last
        if died:
            damage += max(0, prev - summary['hp'])
        # Messages of keys k0+1 .. k1 (1-based key numbers).
        while mi < len(messages) and messages[mi][0] <= k0:
            mi += 1
        msgs = []
        j = mi
        while j < len(messages) and messages[j][0] <= k1:
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
            'disabling': float(threat.get('disabling', 0.0)),
            'sources': threat.get('sources', []),
            'actual': damage,
            # Hunger's faint paralyses too (player-util.c, "You faint from
            # the lack of food."); it is no monster's and is counted apart.
            'disabled': any(m in DISABLED and (k == 0 or msgs[k - 1] != FAINT)
                            for k, m in enumerate(msgs)),
            'fainted': FAINT in msgs,
            'died': died,
            'messages': msgs,
        })
        k0 = k1
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


def analyse(ws, seed=0, blows=None):
    rng = random.Random(seed)
    blows = read_blows() if blows is None else blows
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
    standing = [w for w in threatened if w['kind'] in STANDING]
    rep['standing'] = group_stats(standing, rng) if standing else None

    dis = []
    for lo, hi in zip(DISABLE_BINS, DISABLE_BINS[1:]):
        b = [w for w in ws if lo <= w['disabling'] < hi]
        if b:
            dis.append({'range': [lo, min(hi, 1.0)], 'windows': len(b),
                        'mean_disabling': round(sum(w['disabling'] for w in b) / len(b), 4),
                        'disabled': sum(1 for w in b if w['disabled']),
                        'fainted': sum(1 for w in b if w.get('fainted'))})
    rep['disabling'] = dis

    # The blow decomposition: one awake, in-view, adjacent source and the
    # character stays put, so every blow message in the window is its.
    dec = {}
    for w in single:
        src = w['sources'][0]
        if src['dist'] > 1 or not src.get('awake', True) or not src.get('in_view', True) \
                or w['kind'] not in STANDING or src['name'] not in blows:
            continue
        b = blows[src['name']]
        hits, misses = blow_messages(src['name'], w['messages'])
        d = dec.setdefault(src['name'], {'windows': 0, 'predicted_blows': 0.0, 'hits': 0,
                                         'misses': 0, 'actual': 0, 'predicted': 0.0,
                                         'ended': 0, 'survived_predicted_blows': 0.0,
                                         'survived_blows': 0,
                                         'blows_per_round': len(b),
                                         'mean_per_hit': round(sum(x[2] for x in b) / len(b), 2)
                                         if b else 0.0})
        d['windows'] += 1
        d['predicted_blows'] += src['rounds'] * len(b)
        d['hits'] += hits
        d['misses'] += misses
        if ended(src['name'], w['messages']):
            d['ended'] = d.get('ended', 0) + 1
        else:
            d['survived_predicted_blows'] = d.get('survived_predicted_blows', 0.0) \
                + src['rounds'] * len(b)
            d['survived_blows'] = d.get('survived_blows', 0) + hits + misses
        d['actual'] += w['actual']
        d['predicted'] += w['expected']
    for d in dec.values():
        d['survived_predicted_blows'] = round(d['survived_predicted_blows'], 1)
        d['predicted_blows'] = round(d['predicted_blows'], 1)
        d['predicted'] = round(d['predicted'], 1)
        n = d['hits'] + d['misses']
        d['blows_ratio'] = round(n / d['predicted_blows'], 3) if d['predicted_blows'] else None
        d['hit_rate'] = round(d['hits'] / n, 3) if n else None
        d['damage_per_hit'] = round(d['actual'] / d['hits'], 2) if d['hits'] else None
    rep['blows'] = dict(sorted(dec.items(), key=lambda kv: (-kv[1]['windows'], kv[0])))
    tot = {k: sum(d[k] for d in dec.values())
           for k in ('windows', 'predicted_blows', 'hits', 'misses', 'actual', 'predicted',
                     'ended', 'survived_predicted_blows', 'survived_blows')}
    tot['survived_ratio'] = round(tot['survived_blows'] / tot['survived_predicted_blows'], 3) \
        if tot['survived_predicted_blows'] else None
    n = tot['hits'] + tot['misses']
    tot['blows_ratio'] = round(n / tot['predicted_blows'], 3) if tot['predicted_blows'] else None
    tot['hit_rate'] = round(tot['hits'] / n, 3) if n else None
    tot['damage_per_hit'] = round(tot['actual'] / tot['hits'], 2) if tot['hits'] else None
    rep['blows_total'] = tot
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
                          if rep['single_source'] else {}),
                       **({'standing still (no Walk, stairs, Alter)': rep['standing']}
                          if rep['standing'] else {})}, 'pool'), '']
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
    out += ['', '**Disabling (fear, blindness, confusion, paralysis) against onset messages**',
            '', '| disabling bin | windows | mean predicted | windows with an onset '
            '| windows with a hunger faint (apart) |', '| --- | ---: | ---: | ---: | ---: |']
    for b in rep['disabling']:
        out.append(f"| [{b['range'][0]:g}, {b['range'][1]:g}) | {b['windows']} "
                   f"| {b['mean_disabling']:.4f} | {b['disabled']} | {b['fainted']} |")
    if rep['blows']:
        out += ['', '**Blows: one awake adjacent source in view, the character standing still** '
                '(blows from `monster.txt`; hits and misses from the engine\'s messages)', '',
                '| monster | windows | blows/round | predicted blows | observed blows '
                '| observed/predicted | hit rate | damage per hit (dice mean) '
                '| predicted HP | actual HP |',
                '| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- | ---: | ---: |']
        rows = [(k, d) for k, d in rep['blows'].items() if d['windows'] >= 3]
        for k, d in rows + [('**all**', {**rep['blows_total'], 'blows_per_round': '',
                                          'mean_per_hit': None})]:
            mean = '' if d.get('mean_per_hit') is None else f" ({d['mean_per_hit']:g})"
            out.append(f"| {k} | {d['windows']} | {d['blows_per_round']} "
                       f"| {d['predicted_blows']:.1f} | {d['hits'] + d['misses']} "
                       f"| {fmt(d['blows_ratio'])} | {fmt(d['hit_rate'])} "
                       f"| {fmt(d['damage_per_hit'])}{mean} | {d['predicted']:.1f} "
                       f"| {d['actual']} |")
        t = rep['blows_total']
        out += ['', f"In {t['ended']} of these {t['windows']} windows the monster was slain "
                f"or fled in the window. In the other windows the engine printed "
                f"{t['survived_blows']} blows against {t['survived_predicted_blows']:.1f} "
                f"predicted (ratio {fmt(t['survived_ratio'])})."]
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
