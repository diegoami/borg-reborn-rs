# Escape tier 4: act before the long teleport (4.2)

*#1342. A plan, not a change: no code and no shipped setting moves with this
page. It describes how the compromised escape tier (`# Danger Level 4.1` /
`4.2`) reaches a long teleport, why that is a problem in the recorded
Gnome Mage `Borg261127142222`, and settings, each off by default, that would
make the Borg look, shoot or boost first and teleport only if that fails. Every threshold below is a hypothesis until borg-zoo
measures it (RULES 14). Gameplay evidence belongs in borg-cemetery
(AGENTS.md); this page quotes only what the plan needs and links the issue.*

## What the code does today

`try_escape` in `borg-core/src/borg_escape.rs` (build `9f55466f`, `track-p`)
runs numbered tiers. Tier 4 (`// 4- not too scary but I'm compromised`,
line 2314) fires when the danger where the Borg stands exceeds a share of its
current HP (the Borg's `avoidance`, set to current HP each update):

| clause | danger above | and |
|---|---|---|
| compromised | 0.8 × HP | clevel < 35, or HP ≤ ⅓ of max |
| unique | 0.9 × HP | a unique near, clevel < 35, or HP ≤ ⅓ |
| ordinary | 0.6 × HP | clevel ≤ 20, no unique near |
| general | 0.6 × HP | clevel ≤ 35 |

At clevel 13–15 the last two clauses make the tier a plain "danger above 60%
of current HP" test; the Borg need not be hurt. Inside it:

1. **4.1** (line 2335): stairs underfoot, else Phase Door.
2. **4.2** (line 2353): the long teleport: Dimension Door, a teleport spell,
   an activation, a Scroll of Teleportation, then the Staff of Teleportation.
3. If neither acted: start fleeing and leaving ("failed to teleport"), then
   **4.3**, the weak caster's emergency Phase.

With `escape_phase_adjacent_rule = 1` (the shipped setting, #584), Phase Door
is refused whenever no awake enemy is adjacent: the verdict is `Shoot`, and
the danger is never compared with HP (`phase_rule_verdict`, line 1192). The
rule's note says "other escapes, else attack at range or boost", but in this
tier "other escapes" is 4.2, which runs before `phase_rule_act` (called from
`borg_caution` only after `borg_escape` returns false) can attack. So a Phase
refusal at range becomes a long teleport on the same predicate.

## The four 4.2 escapes in the recording

Turns 1,368,600 to 1,467,919 hold 24 Staff of Teleportation escapes: 15 at
tier 1, 5 at 2.1 and 4 at 4.2. Tiers 1 and 2.1 fired at 1.25 to 30 times HP
and look like real emergencies by the Borg's own estimate. The four 4.2
cases (HP and danger are the Borg's; "without fear" is #878's damage-only
danger, which the Phase rule already prints). All four are real threats in
the owner's reading; the question is whether the long teleport is the
right answer to them.

| case | turn, DL | HP | danger | without fear | what the log shows |
|---|---|---|---|---|---|
| A | 1,368,677, DL7 | 60/60 | 45 | 0 | nothing visible; the Borg retreated one grid at 1,368,657, then `borg_flow_dark` walked it back to the same grid |
| B | 1,377,235, DL7 | 66/67 | 66 | 0 | "You hear an echoing roar. You are hit by noise": an unseen sound breath, 1 HP lost, stunned |
| C | 1,403,464, DL7 | 67/67 | 54 | 8 | a cave spider group woke at 5+ grids (speed +20, 1d4 bites, `friends` 2d8); the next escape came 17 thinks later |
| D | 1,443,183, DL8 | 68/68 | 48 | 48 | an orc tracker about 6 grids away and seven cave orcs 7-13 grids off; the Borg had walked up to shoot the tracker (two Frost Bolts had failed); 24 SP left |

- **B was not necessarily invisible.** "Something" in the Borg's notes means
  unseen, and the engine's message for an unseen sound breath is "an echoing
  roar" (`monster_spell.txt`, `BR_SOUN`). Sound breathers up to DL9 are the
  giant gold dragon fly (DL6) and the baby gold dragon (DL9); 1 HP lost fits
  the dragon fly (an inference; the log does not name it).
- **What the Borg held.** Reveal Monsters (the Mage's detect-invisible,
  level 15) was learned at turn 1,434,747, so the Borg had it at D but not
  at A, B or C; the first Scroll of Detect Invisible was picked up at
  1,457,725 and the first Potion of Speed at 1,450,789, after all four cases.
  At A, B and C it had none of the three; at D it had no speed potion.
- **Detect Monsters** (Mage level 3, 2 SP, `DETECT_VISIBLE_MONSTERS`) was not
  cast in the 1,500 game turns before any of the four cases; at D the
  detection was about 2,300 game turns old (the cave orcs and the tracker
  were last placed at arrival, turn 1,440,896).
- **After D** the danger stayed at 0 for 1,001 thinks (a trip to town
  followed).

Phase Door is not the answer for any of the four: it lands within 10 grids,
which keeps an archer in line of sight (D), leaves an unseen breather in
range (B) and buys one turn against fast spiders (C). The rule's refusal is
right. The long teleport is a heavy answer to threats that a look, a shot or
a boost may settle, and it is what the code reaches first.

**Why escape first.** In a think, `borg_caution` (check source
`borg_think_dungeon.rs:945`) runs before `borg_recover` (`:1174`), where the
#831 "invisible attacker" mode reads or casts a detection. Inside
`borg_caution` the order is: retreat, then `borg_escape`, then
`phase_rule_act` (attack at range, or boost: Speed first, then Heroism,
Berserk Strength, Bless). So every other answer waits until no escape acted.
Detect Monsters is cast on arrival and not again.

The cost over the window: 24 charges, 5 casts of Recharging (5 SP each)
gaining 23 charges. Two short chains of escapes emptied the staff (turns
1,385,578 and 1,403,641, at 1.3 and 1.6 times HP), so the next emergency
had to wait for a recharge.

## The plan: look, shoot or boost, then teleport

All settings default to 0, "retains today's play", like the other escape
settings, so no shipped run changes until the owner moves a value after a
zoo result. Staged so each can be measured alone.

### Stage 1: when threatened, do something about it first

The principle (owner): a Borg that feels threatened asks what it can do
about it (look, heal, shoot, boost), acts, and judges the danger again on
the next think; the long teleport is for when that has not worked.

- **One flag.** `escape_compromised_act_first` (flag, default 0). With 0
  nothing below is read and today's play is kept. The settings that follow
  take effect only with the flag on.
- **When it applies.** Tier 4 has fired (danger above 60% of current HP),
  no awake enemy is adjacent (the Phase verdict for this think is `Shoot`:
  `phase_rule_verdict(st, PhaseSite::Compromised, Some(b_q))`, cached per
  think), the danger without fear is below
  `escape_compromised_act_danger_percent` of current HP (default 100: by the
  Borg's own estimate it survives until it next acts), HP is at least
  `escape_compromised_act_hp_percent` of max (default 50), and fewer than
  `escape_compromised_act_rounds` (default 3) rounds have been spent this
  way in the same episode. The episode ends when the danger falls below the
  tier's threshold or the level changes. Outside these limits 4.2 runs as
  today.
- **The ladder, first one that can act:**
  1. *Look*: Detect Monsters (the arrival check's own cast) when the monster
     picture is older than `escape_compromised_refresh_age` game turns
     (default 1000); with the #831 unseen mode on, `detect_invisible_now`
     instead (Reveal Monsters covers both once known).
  2. *Heal*: the quaff `phase_rule_act` already starts with, when hurt.
  3. *Shoot*: the best attack at range (`borg_attack`; with no enemy
     adjacent only a ranged one can be chosen).
  4. *Boost*: `cornered_boost` (Speed first, then Heroism, Berserk Strength,
     Bless), then `borg_defend`'s resistances and shields, when an awake
     monster is in view.
  The existing `phase_rule_act` already does steps 2 to 4 in that order;
  the new part is step 1 and calling the ladder before 4.2 instead of after
  every escape has declined. A look counts as acting, not as an escape (like
  `EscapeAction::Healed`: no `escapes` count, flow intact).
- **Reevaluate next round.** Every think recomputes the danger and the tier.
  If it grew past the Borg's HP, tier 3, 2 or 1 teleports as today. If the
  danger persists, the round cap sends the Borg to 4.2 after
  `escape_compromised_act_rounds` rounds. If nothing on the ladder can act,
  4.2 runs at once. Tiers 1 to 3 and verdicts other than `Shoot` (an enemy
  adjacent) never use this path.
- A note per decision: `# Escape 4.2 held: {look|heal|shoot|boost}, round
  {n} of {cap}; danger {here} without fear (with fear {b_q}), HP {hp}` and,
  when it does not hold, `# Escape 4.2 not held: {why}`.

Effect on the four cases:

- **A:** nothing known to shoot; the look shows the picture; with no awake
  enemy in view the rest of the ladder has nothing to act on, so the round
  ends in the caution flow (back away, flow); after the cap, or if nothing
  can act, 4.2 runs.
- **B:** the look would show an ordinary unseen monster (or, with the #831
  mode, the detection reveals it); next round a shot at it.
- **C:** the Borg spends a round on a look or a boost with the group at
  range. This is the case at risk: the damage estimate counts the group at 8
  without fear, and its speed +20 can close the gap in one turn; the cap
  and tier 3 bound it.
- **D:** 48 is below 68, HP is full and an attack had positive utility one
  decision earlier, so the Borg shoots the tracker.

### Stage 2: keep a charge in reserve

- `escape_compromised_staff_reserve` (charges, default 0): in 4.2 only, the
  Staff of Teleportation is skipped when it holds this many charges or fewer;
  spells, scrolls and activations are not touched. Tiers 1 to 3 may use the
  last charges. Hypothesis, value 2: in the recording, 1,403,464 would have
  left the staff's last charge for the danger-105 escape 177 game turns
  later (turn 1,403,641).

### Not proposed

A gate that refuses 4.2 on the damage-only danger alone (an earlier draft of
this page: A, B and C are 0, 0 and 8 without fear) is left out: all three are
threats, and a gate would do nothing about them. It stays a possible arm if
stage 1 cannot act in a case like A, where nothing is known to act against.

## Open questions

- **C may be the case this breaks.** The damage estimate ignores the group's
  speed and numbers. Measure before trusting the 100% limit; a fast-group
  override may be needed.
- **A needs its own fix**: a retreat that the next flow undoes
  (`retreat_oscillation_veto` exists, #932). Separate task; not in this plan.
- **A round spent acting while the danger is above 60% of HP** costs a round
  of that danger. The refresh age, the 50% HP floor and the round cap bound
  it; the zoo decides if it pays.
- **An enemy adjacent and the fight not lost** (verdict `Fight`) has the same
  fall-through to 4.2 and is left out here: none of the four cases is one.
  Extend the ladder to it once the `Shoot` case is measured.
- **Danger 2000** (turn 1,385,545, no monster named after an acolyte died) is
  the `danger_maximum` cap. Its source is not in the log; not in this plan.
- The case dossier (all 24 escapes, with the context of each) is not in this
  repository. It belongs in borg-cemetery, with the owner's evidence bundle
  referenced from issue #1342.

## Measuring it (borg-zoo, RULES 14 and 20)

Population comparisons go to borg-zoo as a request issue. Proposed arms: the
shipped settings (flag 0); stage 1 (flag 1); stages 1 and 2, over all classes (the tier is not
Mage-only). The endpoint, its size and the confirmation suite are the zoo's
to prespecify before the run. Candidate measures: deaths and the depth
reached per game, escapes by tier and charges and recharges per level
gained, and deaths within a fixed number of thinks after a held 4.2. No
improvement is claimed here.

## Implementation outline (a later PR per stage)

- `borg-core/src/borg_escape.rs`: a pure `compromised_act_first(...)` for the
  decision (unit-tested on the table above), wrapped around 4.2 and the
  flee/leave flags; the refresh as a new `EscapeAction` that is not counted
  as an escape.
- `borg-core/src/params.rs`: the settings above, each citing the line that
  reads it (`scripts/check-param-citations.py`), with their `harness/pborg.yaml`
  entries at 0, and the entries in `docs/CALIBRATION.md`.
- `docs/PHASE-DOOR.md`: one line pointing the compromised site here.
- Default off keeps `harness/reference/smoke-borg-rs` unchanged; check it with
  the smoke diff, as the other settings do.
