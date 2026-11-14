# The option-C pilot: results

*Option C of [REDESIGN.md](REDESIGN.md), chosen by the owner on
2026-11-14. Issues: #207 (C1), #208 (C2), #209 (C3). Measured on
2026-11-14 against borg-rs `4ed3263`.*

## C1: borg-rs's core inside our shell (#207): met

- **The vendored core.** borg-rs's crate is vendored at the pin as
  `borg-core/` (docs/PROVENANCE.md). `BORG_CORE=borg-rs` selects it at run
  time; our own core stays the default.
- **The footprint in borg-rs's code:**
  - its exported `do_cmd_borg` became a plain function that our
    `do_cmd_borg` calls;
  - three test-data paths were repointed;
  - the core reads its own `borg.txt` (`borg-core/borg.txt`).
- **Parity, byte for byte:**
  - Smoke: 3/3 runs identical (keys, messages, state) with borg-rs's own
    build at the pin.
  - Full: 45/45 identical.
  - Our core, the default, is unchanged: smoke and smoke-exact 3/3
    identical with our references.
- **borg-rs's committed references are stale.** They do not match
  borg-rs's own build at the pin; both builds diverge from them at the same
  keys. The oracle is therefore a fresh run of borg-rs's build at the pin
  (`harness/build.sh` in a scratch copy), not its `harness/reference/`.
- **borg-rs's suites have no `menu` flag.** Without it the headless
  frontend creates a different race or class than the suite line asks for
  in 15 of 45 runs (`harness/check-characters.py`). The parity runs keep
  borg-rs's suite as it is, so both sides play the same characters. Every
  measurement after parity uses `menu`.

The same seeds, through `tools/progress.py`:

| full suite, 45 runs | borg-rs (vendored core) | our core (#201) |
|---|---|---|
| alive | 35 | 13 |
| alive and moving | 31 | 9 |
| mean turns frozen | 5,778 | 7,667 |
| mean max depth | 3.53 | 1.67 |
| mean character level | 5.47 | 3.98 |
| reached DL5 | 18 | 0 |

## C2: errands over the core (#208): trips, in progress

**No stair-scumming, no town scumming.** borg-rs's `borg.txt` has the
munchkin start off, but `borg_munchkin_by_class = TRUE` turned it back on
for Warriors, Druids and Rangers: they stair-scummed to
`borg_munchkin_class_depth` (6). `borg_self_scum = TRUE` let a level-10
character wait in town until it could afford a wanted item. Both are off
(owner, 2026-11-14). They are switched off in `borg-core/overrides.txt`,
which `harness/run.sh` applies over borg-rs's own `borg.txt`; the vendored
file stays as borg-rs ships it.

**The errand is trips** (owner, 2026-11-14): explore DL1 and go back to
town; then DL1, DL2, DL1 and back to town; and so on. The player writes
the plan.
- **The syntax.** `BORG_ERRAND=trips:1*/1,2,1*/1,2,3,2,1*` lists trips
  separated by `/` and levels by `,`.
- **Repeating trips.** A trip marked `*` repeats until borg-rs is prepared
  for the next trip's deepest level; the last one repeats until the budget.
  The player writes the ladder, borg-rs's readiness climbs it: a mage stays
  on town-DL1-town longer, a warrior moves on sooner.
- **On a level of the plan, borg-rs plays as itself.** The level is
  explored when borg-rs decides to leave it (`borg_leave_level`, hooked).
- **Between levels, the errand walks the stairs**, after `borg_caution`,
  with borg-rs's own stair flows. A trip starts from town by the down
  staircase, never by a recall.
- **No level is forced.** A planned level borg-rs is not prepared for
  (`borg_prepared`) is skipped and noted. If borg-rs must restock, the trip
  ends at home.
- **With no errand, every hook returns at once.**

**The first errand, dive to N, was dropped.** It took weak characters
deeper than their level, and killed them: 5 to 8 extra deaths in 45, mostly
casters on DL2–4 (owner: an errand must not dive unprepared, and must not
cancel the way home). Tracing its two early deaths showed that its
readiness override also switched off borg-rs's own "bored and unable to
dive: go to town". The override is gone.

**Trial.** One character per class (the `-0` line of each class in the
full suite, `menu`), with no munchkin and no town scumming:

| | no errand (60,000 turns) | `trips:1*/1,2,1*/1,2,3,2,1*` (budget 55,000) |
|---|---|---|
| deaths | 2 (druid: small kobold; paladin: grey mold) | 1 (druid: giant yellow centipede) |
| character levels | 6, 4, 1, 3, 5, 2, 5, 5, 2 | 5, 3, 1, 3, 3, 4, 5, 4, 2 |
| max depth | 1–3 | 1–2 |
| trips home | — | 23 to 106 per character |

- **The characters gain levels at about borg-rs's own pace**, in explicit
  town-level-town cycles, and reach DL2 only when borg-rs judges them
  prepared.
- **A trip is short.** borg-rs at levels 1–3 tires of a DL1 level in a few
  hundred turns; the errand turns each of those into a trip home.

## Next

1. C2:
   - measure the ladder on all 45 characters against no errand;
   - add *shop*;
   - add the `^Z` keys and `run.sh` flags.
2. C3 (#209): danger and caution into the domain behind a shadow check;
   measure the cost; report.
