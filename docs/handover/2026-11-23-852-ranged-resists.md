## 2026-11-23: caution_ranged_only without the reach block, with resistances (#852)

Follow-up of #838 (PR #843). Owner, 2026-11-21: "keep caution_ranged_only off
... keep 1, 2, 4 and take resistance into account; path finding is part of
another concern".

**Done.** Both settings compile and ship 0; play at the shipped settings is
unchanged.
- **Reach block dropped** (#852 item 1). `in_reach_of_kept`
  (`borg_caution.rs`) reads only `immobile_kept_at_range`
  (`caution_ranged_vs_immobile` 2, 3), so `caution_ranged_only` no longer
  keeps the flows out of a listed monster's grid and the eight around it.
  Kept: no melee blow (`borg_fight_attack.rs`), no hunt walk
  (`borg_flow_kill`), no flow step onto its grid (`borg_flow.rs`:
  `spread_blocked`'s refusal of a kept monster's grid, and `borg_play_step`'s
  refusal to walk into one), and missiles, spells and throws.
- **Resistances** (item 2). A `ranged` entry is kept only while one of its
  blows has an effect the character is not protected from (`listed_kept`).
  `effect_protection` is `borg_danger_one_kill`'s mapping: `PARALYZE`
  `BI_FRACT`, `CONFUSE` `BI_RCONF`, `POISON` `BI_RPOIS`, `BLIND` `BI_RBLIND`,
  `TERRIFY` `BI_RFEAR`; every other effect has none. The blows are read for
  any race but 0 and the placeholder past the last (every race has its blow
  array, `mon-init.c:1826`).
- **Mode 2** (item 3): a separate switch, **`caution_immobile_resists`**
  (flag, 0), because mode 2 is borg-zoo#576's measured baseline. At 1, mode 2
  counts a blow as annoying only when the character is not protected from
  it; it acts on everything that reads `immobile_kept_at_range` (melee, hunt,
  the flows out of its reach, the step away). Mode 3 is unchanged.
- Docs: `params.rs` (both rows), `docs/CALIBRATION.md` (a section; 89 rows'
  citations moved with the inserted lines, `check-param-citations.py --fix`),
  `harness/borg.txt` (comments, and the new switch at 0),
  `harness/scary_monsters.txt` (comment).

**Checked.** CI's four commands; `scripts/check-param-citations.py` (0
wrong); unit tests `effect_protections` and `listed_with_resists` (the
mapping, each shipped entry kind with and without each protection, a wrong
protection, mixed blows, plain damage); smoke at the shipped settings 3/3
identical to `smoke-borg-rs`. Replays from borg-cemetery on this branch
(`runs/replay-zoo-death.sh`, one start each, direction only, RULES 14):

| case (borg-cemetery) | control | list on (`caution_ranged_only` 1 + the 14 lines) |
|---|---|---|
| Druid 625213959, floating eye (`runs/2026-11-21-reborn-head-3b4924ba/replay/`) | the zoo's game | the zoo's game: the eye is first seen adjacent |
| Necromancer 1996566312, floating eye (same run) | the zoo's game | the zoo's game, same reason |
| Ranger 1241679603, floating eye, mode 2 (`runs/2026-11-21-reborn-immobile2-head/replay/`) | alive at 100,000 (main has moved; not the zoo's game) | identical to control |
| Mage 834355294, grey mushroom patch | the zoo's game | alive at 100,000, CL5; a later eye killed with a flask of oil |
| Necromancer 2074919561, grey mushroom patch | the zoo's game | alive at 100,000, CL5 |
| Mage 1770630957, patch, mode 2 | the zoo's game | the zoo's game (as in #838: the held cure and the confused walk, #837's switches off in the case) |
| Ranger 116037261, patch, mode 2 | the zoo's game | the zoo's game, same |

`caution_immobile_resists` 1 on the three mode-2 cases: identical to control
(Human and Dunadan at CL1 have none of the five protections), as it should be.

**A character with Free Action**: the three floating-eye starts as a Gnome
(`race_idx=4`, `p_race.txt` `FREE_ACT`). All three alive at 100,000 turns
with or without the list, and the list on plays key for key as off: the eye
is not kept. Druid 625213959 as a Gnome with the list on: "Walking into a
'floating eye'", "You have slain the floating eye" (game turn 18,053). The
Gnome Ranger with mode 2 and `caution_immobile_resists` 1 plays as with 0:
it shot every eye with arrows and never stood next to one, so mode 2's new
switch is exercised by the unit test only.

**Decided here, for the owner to confirm.**
- Plain damage (`HURT`, `NONE`) does not keep a listed monster that has
  another blow, so resisting a monster's only special blow frees it; an
  entry with only plain-damage blows (or none) stays kept, since it was
  listed by name. None of the 14 shipped entries has a plain-damage blow, so
  this matters only for entries added later.
- The protections are what `borg_danger_one_kill` reads: the five traits,
  and for poison also the temporary resistance (`temp.res_pois`), which its
  `MONBLOW_POISON` branch checks after `BI_RPOIS` (review of #1126: the first
  version read the trait only, so a Borg with only the temporary resistance
  still refused a spotted mushroom patch; `protected`, with the unit test
  `temporary_poison_resistance`). The other four have no temporary form there.

**Left.**
- Zoo measurement (RULES 20) of the new `caution_ranged_only` against
  shipped and against borg-zoo#593's `ranged` arm (the old four-part
  behaviour), and of `caution_immobile_resists` with mode 2. Not filed: the
  owner keeps the setting at 0; filed on request.
- The flows around a kept monster (a listed monster in a corridor, a Borg
  hunted next to one): the path-finding issue #852 points to.
- The trial replays above live in the scratchpad only; a Cemetery trial
  page can be written if the owner wants them retained.
