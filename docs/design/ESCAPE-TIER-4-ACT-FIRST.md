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
| C | 1,403,464, DL7 | 67/67 | 54 | 8 | 16 cave spiders known, the nearest two at (59,65) and (60,65) with the Borg at (59,67): 2 grids away, the rest 3 to 9 (speed 120 against the Borg's 110: two moves to its one; 1d4 bites); the next escape came 17 thinks later |
| D | 1,443,183, DL8 | 68/68 | 48 | 48 | an orc tracker about 6 grids away and seven cave orcs 7-13 grids off; the Borg had walked up to shoot the tracker (two Frost Bolts had failed); 24 SP left |

- **B was not necessarily invisible.** "Something" in the Borg's notes means
  unseen, and the engine's message for an unseen sound breath is "an echoing
  roar" (`monster_spell.txt`, `BR_SOUN`). Sound breathers up to DL9 are the
  giant gold dragon fly (DL6) and the baby gold dragon (DL9); 1 HP lost fits
  the dragon fly (an inference; the log does not name it).
- **What the Borg held** (the excerpt starts at turn 1,368,607, so it
  cannot rule out an earlier pickup). Reveal Monsters (the Mage's
  detect-invisible, level 15) was learned at turn 1,434,747, so the Borg had
  it at D but not at A, B or C; the first Scroll of Detect Invisible was
  picked up at 1,457,725 and the first Potion of Speed at 1,450,789, after
  all four cases.
- **Detect Monsters** (Mage level 3, 2 SP, `DETECT_VISIBLE_MONSTERS`) is cast
  often: 73 casts in the window, from the Borg's panel-based check
  (`borg_light.rs`), not only on arrival. Before B, C and D the last cast was
  2,794, 3,573 and 2,300 game turns earlier; for A the excerpt covers only
  the 70 turns before the case. At D the monsters were last placed at turn
  1,440,883 to 1,440,896.
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

The cost over the window: 24 charges, 5 casts of Recharging (5 SP each)
gaining 23 charges. Two short chains of escapes emptied the staff (turns
1,385,578 and 1,403,641, at 1.3 and 1.6 times HP), so the next emergency
had to wait for a recharge.

## The plan: look, shoot or boost, then teleport

The two switches, `escape_compromised_act_first` (stage 1) and
`escape_compromised_staff_reserve` (stage 2), default to 0, "retains today's
play", like the other escape settings, so no shipped run changes until the
owner moves one after a zoo result. They are independent: the reserve can be
measured with stage 1 off, and stage 1 with the reserve at 0.

Stage 1's four limits (`..._act_danger_percent`, `..._act_hp_percent`,
`..._act_rounds`, `..._refresh_age`) have their **working values as their
defaults, in `params.rs` and in `harness/pborg.yaml` alike**: 100, 50, 3 and
1,000. They are read only while the flag is 1, so with the flag at 0 they
change nothing, and the stage-1 arm of a zoo run changes only the flag. A
limit set to 0 means what it says: a danger ceiling of 0% or 0 rounds never
holds 4.2, an HP floor of 0% has no floor, and a refresh age of 0 looks
every think.

### Stage 1: when threatened, do something about it first

The principle (owner): a Borg that feels threatened asks what it can do
about it (look, heal, shoot, boost), acts, and judges the danger again on
the next think; the long teleport is for when that has not worked.

- **One flag.** `escape_compromised_act_first` (flag, default 0). With 0
  nothing below is read and today's play is kept. The settings that follow
  take effect only with the flag on.
- **Which call may hold 4.2.** Only the normal caution call (`borg_caution`,
  after the retreat step). `borg_escape` gains a `may_hold` argument, false
  for the #934 poisoned-and-no-cure call (`poisoned_escape_first`), which asks
  for an escape before the retreat and must get a real answer.
- **When it applies.** Tier 4 has fired, no awake enemy is adjacent (the
  Phase verdict for this think is `Shoot`:
  `phase_rule_verdict(st, PhaseSite::Compromised, Some(b_q))`, cached per
  think), and all of:
  - the danger without fear is below `escape_compromised_act_danger_percent`
    of current HP (default 100: by the Borg's own estimate it survives until
    it next acts);
  - HP is at least `escape_compromised_act_hp_percent` of max (default 50);
  - **no awake known monster can reach the Borg before its next action**.
    The energy schedule (`game-world.c`: `turn_energy`, `extract_energy`;
    speed 110 gives 10 a tick, 120 gives 20, 130 gives 30, 140 gives 38)
    gives a conservative bound on a monster's actions between two Borg
    actions. The rates can change during the wait, so the bound takes the
    **worst case of each**:
    - the Borg's rate is the *minimum*: the engine's own speed without
      temporary haste (`player-calcs.c`: `TMD_FAST` and `TMD_SPRINT` give
      +10 and can expire on the next world tick) and with Slow kept
      (`TMD_SLOW` gives -10); the Borg acts at most every `T =
      ceil(100 / rate_min)` ticks;
    - the monster's rate is the *maximum*: its `mspeed` (the tracked
      `kill.speed` is the raw one) plus the engine's timed adjustment
      (`mon-move.c`: +10 with `MON_TMD_FAST`, minus 2 per slow level, which
      is given no credit, since it can expire), plus +10 more if its race has
      the `HASTE` spell (`monster_spell.txt`: `MON_TIMED_INC:FAST`);
    - a monster then gets at most `moves = floor(rate_max * T / 100) + 1`
      actions in that time (the `+1` is energy it already holds). A
      rounded-up ratio of the rates can undercount this (Borg speed 119
      against a monster at 140: 19 and 38 a tick, a ratio of 2, a bound of 3;
      a Borg at 110 with haste that lapses against the same monster: a
      bound of 4, three steps and a bite).
    The monster's distance is the Chebyshev one, `max(|dy|, |dx|)`, not
    `engine::distance` (which adds half the shorter leg: it says 4 for an
    offset of (3,3), though a monster steps there in 3 moves). The hold is
    refused when that distance is at most `moves + 1`. For the recorded
    spiders (20 against 10, no haste: `moves` = 3) that is 4 grids; the
    nearest were 2. **When a rate cannot be established** (a monster whose
    speed is not known, a timed state the Borg cannot read) the hold is
    declined and 4.2 runs as today. This is a fixed rule, not a setting. The
    danger sum does not certify it: its reach test simulates one step of the
    monster (`borg_danger.rs`, the "single grid movement" loop), so a fast
    monster two or three grids away adds nothing to the number;
  - fewer than `escape_compromised_act_rounds` rounds (default 3) have been
    spent this way in the episode.
  Outside these limits 4.2 runs as today.
- **The episode.** One counter. It resets when tier 4's own predicate is
  false for a think (one function, `tier4_applies(st, b_q, risky_boost)`,
  used by tier 4 and by the reset, so the strict `>` comparisons and the
  risky-play adjustment are the same), when the level changes, and when any
  other escape acts (including a call that bypassed tier 4).
- **The ladder, first one that can act:**
  1. *Look*, when the monster picture is older than
     `escape_compromised_refresh_age` game turns (default 1000): with the #831
     unseen mode on, `detect_invisible_now`, within #831's own freshness
     (`unseen_fresh`) and read limit (`recover_unseen_detect_reads`); where
     that has nothing to use (case B: Reveal Monsters not learned, no scroll),
     or the mode is off, Detect Monsters (the panel check's own cast).
  2. *Heal*: the quaff `phase_rule_act` already starts with, when hurt.
  3. *Shoot*: the best attack at range (`borg_attack`; it needs a target in
     view, so a look does not promise a shot).
  4. *Boost*: `cornered_boost` (Speed first, then Heroism, Berserk Strength,
     Bless), then `borg_defend`'s resistances and shields, when an awake
     monster is in view.
  `phase_rule_act` already does steps 2 to 4 in that order; the new part is
  step 1 and running the ladder before 4.2 instead of after every escape has
  declined. A look counts as acting, not as an escape (like
  `EscapeAction::Healed`: no `escapes` count, flow intact).
- **What a held turn skips.** The Borg acted, so `try_escape` returns at
  once: the "failed to teleport" flee and leave flags, tiers 5 to 8 and the
  back-away do not run that think. That is the intent; the next think asks
  again. If nothing on the ladder can act, 4.2 runs and nothing is skipped.
- **Reevaluate next round.** Every think recomputes the danger and the tier.
  If it grew past the Borg's HP, tier 3, 2 or 1 teleports as today. If the
  danger persists, the round cap sends the Borg to 4.2 after
  `escape_compromised_act_rounds` rounds. Tiers 1 to 3 and verdicts other
  than `Shoot` (an enemy adjacent) never use this path.
- A note per decision: `# Escape 4.2 held: {look|heal|shoot|boost}, round
  {n} of {cap}; danger {here} without fear (with fear {b_q}), HP {hp}` and,
  when it does not hold, `# Escape 4.2 not held: {why}`.

Effect on the four cases:

- **A:** nothing known to shoot; the look shows the picture; with no awake
  enemy in view the rest of the ladder has nothing to act on, so the round
  ends in the caution flow (back away, flow); after the cap, or if nothing
  can act, 4.2 runs.
- **B:** a look (Detect Monsters, since the unseen-mode detection was not
  available) shows an ordinary unseen monster if there is one; a shot
  follows only if it is in view.
- **C:** the nearest spiders are 2 grids away with two moves to the Borg's
  one, inside `moves + 1`, so the reach rule refuses and 4.2 runs as today.
  Whether a group a little farther off needs a rule of its own is open.
- **D:** the monster picture is 2,300 game turns old, so the first round is a
  look (Detect Monsters). A shot is possible the round after; the last
  positive attack decision was five thinks before the escape, with movement
  since, and none was evaluated at the escape decision.

### Stage 2: keep a charge in reserve

- `escape_compromised_staff_reserve` (charges, default 0): the Staff of
  Teleportation is skipped when it holds this many charges or fewer, in
  **every non-emergency rung that can use it**: 4.2, 5.2 (very low level),
  6.2 (out of mana) and tier 8 (twitching, which calls the staff after the
  same alternatives). The staff rungs of tiers 1 to 3 may use the last
  charges. Tier 8's other teleports (spells, scrolls, off-level) are not
  touched. One helper, `staff_reserved(st)`, read by those rungs, so 5.2, 6.2 and
  tier 8 do not spend what 4.2 left. The charges are those of the stack the use
  routine selects: `borg_slot(st, TV_STAFF, sval)`, its `pval`
  (`borg_use_staff_fail`). Spells, scrolls and activations are not touched.
  Hypothesis, value 2: in the recording, 1,403,464 would have left the
  staff's last charge for the danger-105 escape 177 game turns later
  (turn 1,403,641).

### Not proposed

A gate that refuses 4.2 on the damage-only danger alone (an earlier draft of
this page: A, B and C are 0, 0 and 8 without fear) is left out: all three are
threats, and a gate would do nothing about them. It stays a possible arm if
stage 1 cannot act in a case like A, where nothing is known to act against.

## Open questions

- **C is handled only for the nearest spiders.** The reach rule refuses
  inside `moves + 1`; a group arriving a round later is bounded by the round
  cap and tier 3. Measure before trusting the 100% limit; a rule on group
  size may be needed.
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

Population comparisons go to borg-zoo as a request issue, each tied to a
build that contains what it varies. Stage 1 can merge before stage 2, so
the first request, after the stage-1 issue (diegoami/borg-reborn-rs#1348), is
shipped settings (flag 0) against stage 1 (flag 1). Once the reserve exists
(diegoami/borg-reborn-rs#1349), a request adds the reserve alone and the
reserve with stage 1, on a build that has both; the stages are independent
switches. All over all classes (the tier is not Mage-only). The endpoint, its size and the confirmation suite are the zoo's
to prespecify before the run. Candidate measures: deaths and the depth
reached per game, escapes by tier and charges and recharges per level
gained, and deaths within a fixed number of thinks after a held 4.2. No
improvement is claimed here.

## Implementation outline (a later PR per stage)

- `borg-core/src/borg_escape.rs`: `tier4_applies`, a pure
  `compromised_act_first(...)` for the decision (unit-tested on the table
  above, including the reach rule at the recorded spider positions, the
  diagonal (3,3) offset, a speed-119 Borg against a speed-140 monster (the
  rounded ratio gives 2 actions, the bound 3), and a hasted Borg whose haste
  lapses), wrapped
  around 4.2 and the flee/leave flags; `may_hold` on `borg_escape`; the look
  as a new `EscapeAction` that is not counted as an escape;
  `staff_reserved` in 4.2, 5.2, 6.2 and tier 8.
- `borg-core/src/params.rs`: the settings above, each citing the line that
  reads it (`scripts/check-param-citations.py`), with their `harness/pborg.yaml`
  entries: the two switches at 0 and stage 1's four limits at their working
  values, 100, 50, 3 and 1,000 (a zero there would disable every hold, so it
  is never the shipped value), and the entries in `docs/CALIBRATION.md`.
- `docs/PHASE-DOOR.md`: one line pointing the compromised site here.
- Default off keeps `harness/reference/smoke-borg-rs` unchanged; check it with
  the smoke diff, as the other settings do.
