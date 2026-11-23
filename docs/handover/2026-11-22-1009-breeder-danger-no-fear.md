## 2026-11-22: The breeder stand pause and boost on the danger without fear (#1009)

Branch `task/T1009-breeder-danger-no-fear`, from `track-p` `6bc4de1d`.
Track P, P8 (owner: "start working on those issues"). A follow-up of #878.

**Done.** `caution_breeder_danger_damage_only` (flag, compiled and shipped 0;
registry 2385 to 2386): the stand's one-think pause (`breeders_in_danger` in
`note_breeders_stoppable`) and the hard-breeder-fight boost read
`borg_danger_damage_only` (#878), each at the horizon of the danger it
replaces (1, and `caution_danger_horizon`).

**Checked.** CI's commands (see the PR); smoke at the shipped settings 3/3
identical to `smoke-borg-rs`. Named case, direction only: the Mage
461220874 save at turn 15037 on this branch, track-p's shipped settings,
100,000 turns:

| arm | first contact (15047) | turn 37549, worm adjacent, HP 31/31 | end |
|---|---|---|---|
| shipped | Phase Door (Phase rule, danger 32) | "Retreating (37 > 1)" | dead at 74323, attacking a grey mushroom patch on sight (#969) |
| `caution_breeder_danger_damage_only=1` | Phase Door (the same: the Phase rule acts first) | no retreat, attacks | alive at the cap, CL7, DL3 |

One case: the survival difference is not a result (RULES 14). The first
contact is unchanged because the Phase rule's verdict comes before the stand
is consulted; #878's `escape_phase_lost_damage_only` is the switch for that.

**Not done / left.**
- **Measurement:** borg-zoo arms `caution_breeder_danger_damage_only=1`,
  alone and with `danger_breeder_melee_extra_tenths=5` and with
  `escape_phase_lost_damage_only=1`, against the shipped settings. Not
  filed: the owner starts zoo work (RULES 20).
- Interacts with PR #893 (#882, `caution_breeder_hold`, open) and PR #953
  (dead ends near breeders, open): both touch breeder retreats; not merged
  with this branch.

### After the review (PR #1012, pinned `53a1d971`)

- Synchronized with `track-p` `72651d25` (citation conflicts, both
  CALIBRATION sections kept).
- **F1 (P2), the boost:** the damage-only danger drops the breeder bonus too,
  so it can only lower the boost's danger, and against one white worm mass
  the boost stops firing (2 x 10 < 20 x 5). That ran against the switch's
  "stand and fight", and I had not said so. Split: `caution_breeder_danger_damage_only`
  now moves only the stand pause; the new `caution_breeder_boost_damage_only`
  (shipped 0) moves the boost, so the zoo measures them apart. Registry
  2397 to 2399. In the Mage arms above the boost never fired (0 "Hard breeder
  fight" notes in both), so their result is the pause's alone.
- F2: `pick_danger` (generic, engine-free) chooses each site's danger and is
  tested, including that the unread one is never computed.
- F3: the boost's damage-only danger is computed only when the boost is
  otherwise due (on, standing): `breeder_fight_hard` checks the tenths and
  the stand first (both pure, so the order changes nothing at the shipped
  settings; smoke 3/3 identical).
- The CALIBRATION paragraph now says which danger each switch reads; each
  parameter cites its own read site.
- borg-zoo#692 is pinned to the pre-split specimen: its `stand` arm moved
  both sites. It needs a new pin and a `boost` arm (comment on #692).
