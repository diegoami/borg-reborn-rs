#!/usr/bin/env python3
"""Shopping audit: what the Borg buys, uses, stores, sells and leaves unspent.

  python3 -I tools/shopping-audit.py RUNDIR... [--json OUT] [--class NAME]

Each RUNDIR is one game's harness run directory (messages.log, state.log,
summary.json). Read only; nothing about play changes. Sources:

* bought: "You bought (a|an|N) NAME for G gold." followed by the pack line
  "You have ... NAME (x)", which names a flavour the purchase identifies;
* used: the Borg's own notes "# Quaffing/Reading/Eating/Aiming/Zapping/
  Using/Activating/Wearing/Wielding ...", "Throwing painful object '...'", "Firing missile";
* the town plan's actions (borg_town_record::action), noted as
  "# Town plan executing action N at shop S slot K": 3 sell from the pack,
  4 sell from the Home, 5 store in the Home, 2 take from the Home; the next
  "You had/drop/have ... NAME" line names the item (a sale's "You had"
  line is the real name even when the flavour was unknown);
* dropped: "# Town plan dropping worthless stack", then "You drop ...";
* gold: state.log's `au` on the last town line before a descent (d0 -> d>0)
  and on the first town line after arriving.

Under birth_no_selling (the zoo's setting) a sale pays nothing: selling
only frees a slot. Counts are copies where the message gives a count,
else events.
"""
import argparse
import collections
import glob
import json
import os
import re
import statistics

MSG = re.compile(r'^\d+ (\d+) \S+ ?(.*)$')
BUY = re.compile(r'^You bought (an? |\d+ )?(.+?) for (\d+) gold')
HAVE = re.compile(r'^You have (an? |\d+ )?(.+?) \([a-z]\)\.?$')
HAD = re.compile(r'^You had (an? |\d+ |no more )?(.+?) \([a-z]\)\.?$')
DROP = re.compile(r'^You drop (an? |\d+ )?(.+?) \([a-z]\)\.?$')
ACTION = re.compile(r'^# Town plan executing action (\d) at shop (\d+) slot (\d+)')
USE = re.compile(r"^# (?:(?:Reading|Quaffing|Eating|Aiming|Zapping|Using|Activating|Wearing|Wielding) (.+?)\.?$"
                 r"|Throwing painful object '(.+?)'|Firing missile '(.+?)')")
STATE = re.compile(r'^(\d+) \S+ d(\d+)/(\d+) .* au(\d+) ')

# Kinds the core uses on purpose in a fight or to survive. Selling or
# storing one is worth a look; the list is a reading aid, not a rule.
USEFUL = re.compile(r'Cure (Light|Serious|Critical) Wounds|Potion of (Healing|\*Healing\*|Life|Speed|'
                    r'Heroism|Berserk|Resist|Neutralize Poison|Restore Mana)|Phase Door|Teleport|'
                    r'Word of Recall|Deep Descent|Flask of oil|Magic Mapping|Detect Invisible|'
                    r'Light|Blessing|Holy Chant|Recharging|Wand of|Rod of|Staff of|'
                    r'Sling|Bow|Crossbow|Arrow|Bolt|Shot|Pebble')


def norm(name):
    name = re.sub(r'\s*\{.*?\}', '', name).strip()
    name = re.sub(r'^(an? |the |\d+ |no more )', '', name)
    name = re.sub(r' \(\d+ charges?\)| \(\d+ turns\)| \(charging\)', '', name)
    for plural, single in [('Scrolls', 'Scroll'), ('Potions', 'Potion'), ('Flasks', 'Flask'),
                           ('Rations', 'Ration'), ('Pints', 'Pint'), ('Wounds', 'Wound'),
                           ('Shots', 'Shot'), ('Arrows', 'Arrow'), ('Bolts', 'Bolt'),
                           ('Pebbles', 'Pebble'), ('Torches', 'Torch'), ('Biscuits', 'Biscuit'),
                           ('Handfuls', 'Handful'), ('Apples', 'Apple'), ('Strips', 'Strip'),
                           ('Slices', 'Slice'), ('Books', 'Book'), ('Swigs', 'Swig')]:
        name = re.sub(rf'\b{plural}\b', single, name)
    name = name.replace('Wound', 'Wounds')
    return re.sub(r' \([+-]?\d+(,[+-]?\d+)?\)| \[\d+(,[+-]\d+)?\]| <[+-]\d+>| \(x\d\)', '', name).strip()


def count(prefix):
    prefix = (prefix or '').strip()
    return int(prefix) if prefix.isdigit() else 1


def town_departures(path, shop_turns):
    """(gold on arrival, gold on departure, turn) per town visit that
    entered a shop; stair bounces through the town level are not visits."""
    visits, arrive, arrive_turn, prev = [], None, 0, None
    for line in open(path, errors='replace'):
        m = STATE.match(line)
        if not m:
            continue
        turn, depth, gold = int(m.group(1)), int(m.group(2)), int(m.group(4))
        if depth == 0 and (prev is None or prev[1] != 0):
            arrive, arrive_turn = gold, turn
        if depth != 0 and prev is not None and prev[1] == 0 and arrive is not None:
            if any(arrive_turn <= t <= turn for t in shop_turns):
                visits.append((arrive, prev[2], turn))
        prev = (turn, depth, gold)
    return visits


CAPS = ["light turns", "nutrition", "healing HP", "poison cure", "phase", "Recall",
        "ranged damage", "teleport", "level escape", "teleport other", "digging", "trap/door detection",
        "monster detection", "invisible detection", "mapping", "illumination", "speed", "resist fire",
        "resist cold", "resist poison", "resist acid", "resist electricity", "combat buff", "mana",
        "restore experience", "remove curse", "enchantment", "recharging", "confusion cure", "blindness cure",
        "fear cure", "glyph", "strength", "intelligence", "wisdom", "dexterity", "constitution", "charisma",
        "book 0", "book 1", "book 2", "book 3", "book 4", "book 5", "book 6", "book 7", "book 8",
        "banishment", "destruction", "wound/stun cure", "identify runes", "gain experience",
        "recharge target", "equipment opportunity", "cut reduction", "stun reduction", "monster control",
        "Ranger usable shots"]  # borg_town_capability::NAMES
UNIT = 1000
W_GOLD, W_SPACE = 14, 22  # indices in the record's params (borg_h order from borg_town_plan)


def coverage(row, amount):
    # borg_town_capability::coverage
    if amount <= 0 or row[1] <= 0:
        return 0
    if row[2] < 0:
        return row[1] * min(amount, -row[2]) // -row[2]
    return row[1] * amount // (amount + max(row[2], 1))


def leave_steps(path):
    """At each departure from town (plan action LEAVE), price one more copy
    of every affordable shop ware (shops 0-5) with the record's own curves:
    its marginal gain, the pack-slot burden, and the gold-retention loss
    (borg_town_solve::Cash). The binding reason is:
    'demand met' - no ware adds more than its slot costs;
    'gold kept'  - some ware adds value, but less than spending its price loses;
    'priced out' - the wares that add value cost more than the purse."""
    out = []
    for line in open(path, errors='replace'):
        try:
            r = json.loads(line)
        except ValueError:
            continue
        if r.get('paction') != 7 or 'capability_curves' not in r:
            continue
        curves, gold = r['capability_curves'], r['gold']
        p = r.get('configured_params') or r['params']
        w_gold, w_space = p[W_GOLD] * UNIT, p[W_SPACE] * UNIT
        free = r['limits'][1]
        supplies = collections.defaultdict(list)
        for scope, slot, cap, n in r['capability_supplies']:
            if scope <= 5:
                supplies[(scope, slot)].append((cap, n))
        price = {(s, w[0]): w[4] for s, ws in r['shops'] for w in ws if s <= 5}
        opens = {(w[0], w[1]): w[4] for w in r.get('wares', [])}  # slots_cost: 0 joins a stack
        best = None  # (net, cap, price, cash)
        reason = 'demand met'
        for key, sup in supplies.items():
            if key not in price:
                continue
            gain = sum(coverage(curves[c], curves[c][3] + n) - coverage(curves[c], curves[c][3]) for c, n in sup)
            net = gain - w_space * opens.get(key, 1) // (max(free, 0) + 1)
            if net <= 0:
                continue
            cost = price[key]
            if cost > gold:
                if reason == 'demand met':
                    reason = 'priced out'
                continue
            cash = w_gold * cost // (gold - cost + 1)
            cap = max(sup, key=lambda cn: coverage(curves[cn[0]], curves[cn[0]][3] + cn[1]) - coverage(curves[cn[0]], curves[cn[0]][3]))[0]
            if best is None or net - cash > best[0] - best[3]:
                best = (net, cap, cost, cash)
            reason = 'gold kept'
        out.append({'gold': gold, 'level': r['level'], 'reason': reason,
                    'cap': CAPS[best[1]] if best and best[1] < len(CAPS) else None,
                    'net': best[0] if best else 0, 'cash': best[3] if best else 0})
    return out


def audit(rundir):
    s = json.load(open(os.path.join(rundir, 'summary.json')))
    ev = collections.defaultdict(collections.Counter)  # name -> event -> copies
    gold_spent = collections.Counter()
    pending = None  # (kind, name, copies)
    dropping = False
    shop_turns = []
    for line in open(os.path.join(rundir, 'messages.log'), errors='replace'):
        m = MSG.match(line.rstrip('\n'))
        if not m:
            continue
        text = m.group(2)
        if text.startswith("# Entering a '") and not text.startswith("# Entering a '7'"):
            shop_turns.append(int(m.group(1)))
        if pending and pending[0] == 'buy':
            h = HAVE.match(text)
            name = norm(h.group(2)) if h else pending[1]
            ev[name]['bought'] += pending[2]
            if name != pending[1]:
                # The purchase identified a flavour: a knowledge purchase.
                ev[name]['bought_unknown'] += pending[2]
            gold_spent[name] += pending[3]
            pending = None
            if h:
                continue
        b = BUY.match(text)
        if b:
            pending = ('buy', norm(b.group(2)), count(b.group(1)), int(b.group(3)))
            continue
        a = ACTION.match(text)
        if a:
            pending = ({'2': 'taken', '3': 'sold', '4': 'sold_home', '5': 'stored'}.get(a.group(1)), None, 0)
            continue
        if text.startswith('# Town plan dropping worthless stack'):
            dropping = True
            continue
        if pending and pending[0] in ('sold', 'sold_home'):
            if text.startswith('# Selling '):
                pending = (pending[0], norm(text[len('# Selling '):]), 0)
                continue
            h = HAD.match(text) or DROP.match(text)
            if h:
                name = norm(h.group(2))
                ev[name][pending[0]] += count(h.group(1))
                if pending[1] and pending[1] != name:
                    # The Borg's note named a flavour: it sold an unknown.
                    ev[name]['sold_unknown'] += count(h.group(1))
                pending = None
            continue
        if pending and pending[0] == 'stored':
            h = DROP.match(text)
            if h:
                ev[norm(h.group(2))]['stored'] += count(h.group(1))
                pending = None
            continue
        if pending and pending[0] == 'taken':
            h = HAVE.match(text)
            if h:
                ev[norm(h.group(2))]['taken'] += 1
                pending = None
            continue
        if dropping:
            h = DROP.match(text)
            if h:
                ev[norm(h.group(2))]['dropped'] += count(h.group(1))
                dropping = False
            continue
        u = USE.match(text)
        if u:
            ev[norm(next(g for g in u.groups() if g))]['used'] += 1
    return {
        'run': os.path.basename(rundir.rstrip('/')),
        'class': s.get('class', '?'), 'race': s.get('race', '?'),
        'stop': s.get('stop_reason'), 'clevel': s.get('clevel'), 'max_depth': s.get('max_depth'),
        'items': {k: dict(v) for k, v in ev.items()},
        'gold_spent': dict(gold_spent),
        'visits': town_departures(os.path.join(rundir, 'state.log'), shop_turns),
        'leaves': leave_steps(os.path.join(rundir, 'plan.jsonl'))
        if os.path.exists(os.path.join(rundir, 'plan.jsonl')) else [],
    }


def pct(xs, q):
    if not xs:
        return 0
    xs = sorted(xs)
    return xs[min(len(xs) - 1, int(q * len(xs)))]


def report(games):
    n = len(games)
    tot = collections.defaultdict(collections.Counter)
    in_games = collections.defaultdict(collections.Counter)
    spent = collections.Counter()
    for g in games:
        for name, e in g['items'].items():
            tot[name].update(e)
            for k, v in e.items():
                if v:
                    in_games[name][k] += 1
        spent.update(g['gold_spent'])
    print(f'# Shopping audit: {n} games\n')

    print('## Bought, by gold spent\n')
    print('| item | bought | as unknown flavour | games | gold | used | stored | sold | dropped | bought, never used (games) |')
    print('|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|')
    for name in sorted(spent, key=lambda k: -spent[k])[:40]:
        t = tot[name]
        never = sum(1 for g in games if g['items'].get(name, {}).get('bought') and not g['items'][name].get('used'))
        print(f"| {name} | {t['bought']} | {t['bought_unknown']} | {in_games[name]['bought']} | {spent[name]} | {t['used']} | "
              f"{t['stored']} | {t['sold'] + t['sold_home']} | {t['dropped']} | {never} |")

    print('\n## Bought and then stored at the Home or sold (same game)\n')
    print('| item | games bought | of those, stored | of those, sold | of those, used |')
    print('|---|---:|---:|---:|---:|')
    rows = []
    for name in tot:
        b = [g['items'][name] for g in games if g['items'].get(name, {}).get('bought')]
        if not b:
            continue
        st = sum(1 for e in b if e.get('stored'))
        so = sum(1 for e in b if e.get('sold') or e.get('sold_home'))
        if st or so:
            rows.append((st + so, name, len(b), st, so, sum(1 for e in b if e.get('used'))))
    for _, name, nb, st, so, us in sorted(rows, reverse=True)[:30]:
        print(f'| {name} | {nb} | {st} | {so} | {us} |')

    print('\n## Sold or dropped, by copies (pack and Home)\n')
    print('| item | sold | of those, flavour unknown | dropped | games | useful kind | of those games, later used one |')
    print('|---|---:|---:|---:|---:|:---:|---:|')
    sold = sorted(tot, key=lambda k: -(tot[k]['sold'] + tot[k]['sold_home'] + tot[k]['dropped']))
    for name in sold[:40]:
        t = tot[name]
        q = t['sold'] + t['sold_home'] + t['dropped']
        if not q:
            break
        gs = in_games[name]['sold'] + in_games[name]['sold_home'] + in_games[name]['dropped']
        later = sum(1 for g in games if (g['items'].get(name, {}).get('sold') or g['items'].get(name, {}).get('sold_home'))
                    and g['items'][name].get('used'))
        print(f"| {name} | {t['sold'] + t['sold_home']} | {t['sold_unknown']} | {t['dropped']} | {gs} | "
              f"{'yes' if USEFUL.search(name) else ''} | {later} |")

    print('\n## Gold left unspent when leaving town\n')
    deps = [(a, d) for g in games for a, d, _ in g['visits']]
    first = [g['visits'][0] for g in games if g['visits']]
    later = [v for g in games for v in g['visits'][1:]]
    for label, vs in [('first departure', first), ('later departures', later), ('all', [(a, d, 0) for a, d in deps])]:
        if not vs:
            continue
        left = [d for _, d, _ in vs]
        share = [d / a for a, d, _ in vs if a > 0]
        print(f'- {label}: {len(vs)} departures; gold left median {statistics.median(left)}, '
              f'p75 {pct(left, .75)}, p90 {pct(left, .9)}; share of arrival gold kept median '
              f'{statistics.median(share):.0%}, p75 {pct(share, .75):.0%}' if share else '')
    rich = [(d, g['run'], g['class']) for g in games for a, d, _ in g['visits'] if d >= 500]
    print(f'- departures with 500+ gold unspent: {len(rich)} in {len({r for _, r, _ in rich})} games')
    by_class = collections.defaultdict(list)
    for g in games:
        for a, d, _ in g['visits']:
            if a > 0:
                by_class[g['class']].append(d / a)
    print('\n| class | departures | median share kept | p75 |')
    print('|---|---:|---:|---:|')
    for c, xs in sorted(by_class.items()):
        print(f'| {c} | {len(xs)} | {statistics.median(xs):.0%} | {pct(xs, .75):.0%} |')

    leaves = [lv for g in games for lv in g['leaves']]
    if not leaves:
        return
    print('\n## Why the gold stayed in the purse (plan record, every town departure incl. stair bounces)\n')
    print('Reasons: *demand met*, no ware adds more than its pack slot costs; *gold kept*, '
          'a ware adds value but less than spending its price loses (borg_town_w_gold); '
          '*priced out*, the wares that add value cost more than the purse.\n')
    print('| purse at departure | departures | demand met | gold kept | priced out |')
    print('|---|---:|---:|---:|---:|')
    for lo, hi in [(0, 100), (100, 300), (300, 1000), (1000, 3000), (3000, 10 ** 9)]:
        b = [lv for lv in leaves if lo <= lv['gold'] < hi]
        if b:
            c = collections.Counter(lv['reason'] for lv in b)
            print(f"| {lo}-{hi if hi < 10 ** 9 else ''} | {len(b)} | {c['demand met']} | {c['gold kept']} | {c['priced out']} |")
    kept = collections.Counter(lv['cap'] for lv in leaves if lv['reason'] == 'gold kept')
    print('\nWhen the gold was kept, the capability the best ware would have added: '
          + ', '.join(f'{k} {v}' for k, v in kept.most_common(10)))


def main():
    ap = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    ap.add_argument('rundirs', nargs='+', help='run directories or globs')
    ap.add_argument('--json', help='write the per-game audit as JSON here')
    ap.add_argument('--class', dest='cls', help='only games of this class')
    args = ap.parse_args()
    dirs = sorted({d for p in args.rundirs for d in glob.glob(p)
                   if os.path.exists(os.path.join(d, 'summary.json'))})
    games = [audit(d) for d in dirs]
    if args.cls:
        games = [g for g in games if g['class'] == args.cls]
    if args.json:
        json.dump(games, open(args.json, 'w'), indent=1)
    report(games)


if __name__ == '__main__':
    main()
