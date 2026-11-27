# Escape tier 4: gating the long teleport (4.2)

*#1342. A plan, not a change: no code and no shipped setting moves with this
page. It describes how the compromised escape tier (`# Danger Level 4.1` /
`4.2`) reaches a long teleport, why that is a problem in the recorded
Gnome Mage `Borg261127142222`, and three settings, each off by default, that
would change it. Every threshold below is a hypothesis until borg-zoo
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
danger, which the Phase rule already prints):

| case | turn, DL | HP | danger | without fear | what the log shows |
|---|---|---|---|---|---|
| A | 1,368,677, DL7 | 60/60 | 45 | **0** | nothing visible; the Borg retreated one grid at 1,368,657, then `borg_flow_dark` walked it back to the same grid |
| B | 1,377,235, DL7 | 66/67 | 66 | **0** | an invisible caster ("hit by noise", stunned, 1 HP lost); region fear only; the Borg rested at once |
| C | 1,403,464, DL7 | 67/67 | 54 | **8** | a cave spider group woke at 5+ grids (speed +20, 1d4 bites, `friends` 2d8); the next escape came 17 thinks later |
| D | 1,443,183, DL8 | 68/68 | 48 | **48** | an orc tracker about 6 grids away and seven cave orcs 7–13 grids off; the Borg had walked up to shoot the tracker (two Frost Bolts had failed); 24 SP left |

A, B and C are fear-map escapes with little or no monster damage behind
them. D has real damage, but a Frost Bolt kills a 55 HP tracker in about two
casts, and the Borg judged an attack worth taking one decision earlier. After
D the danger stayed at 0 for 1,001 thinks (a trip to town followed).

Phase Door is not the answer for any of the four: it lands within 10 grids,
which keeps an archer in line of sight (D), leaves an unseen caster in
range (B) and buys one turn against fast spiders (C). The rule's refusal is
right; the issue is what follows it.

The cost over the window: 24 charges, 5 casts of Recharging (5 SP each)
gaining 23 charges. Two short chains of escapes emptied the staff (turns
1,385,578 and 1,403,641, at 1.3 and 1.6 times HP), so the next emergency
had to wait for a recharge.

## The plan: three settings, staged

All default to 0, "retains today's play", like the other escape settings, so
no shipped run changes until the owner moves a value after a zoo result.

### Stage 1: gate 4.2 on damage-only danger

- `escape_compromised_teleport_gate` (flag): 1 = the 4.2 rung (and the
  "failed to teleport" flee/leave flags that follow it) runs only if the
  danger without fear, `borg_danger_damage_only` at the Borg's grid (the
  figure `phase_rule_verdict` already computes), is at least
  `escape_compromised_teleport_damage_percent` of current HP.
- `escape_compromised_teleport_damage_percent` (percent of current HP,
  default 60): the same share the tier uses, applied to the damage alone.
  Hypothesis; no measurement chose it.
- 4.1 stays as is (stairs underfoot are cheap and safe, and Phase is already
  ruled). 4.3 and tiers 5 to 8 stay too: the gate wraps 4.2 and the
  flee/leave flags only, so a refused 4.2 does not set `fleeing` or
  `leaving` for an escape the Borg did not try.
- The gate stays **open** (today's play) while the state #831 keeps for the
  "invisible attacker near" mode is set (`borg_recover.rs:840`), or the Borg
  is stunned, confused or blind: the damage estimate cannot see such an
  attacker, and case B is that attacker.
- A note per decision: `# Escape 4.2 gate: danger {here} without fear (with
  fear {b_q}), {pct}% of HP: teleport allowed|refused`.

Effect on the four cases at 60%: A (0 of 36) refused; B (0 of 40) refused
unless the unseen-attacker guard opens it; C (8 of 40) refused; D (48 of 40)
allowed. Tiers 1 to 3 never see the gate.

### Stage 2: let the rule's attack run before the staff

- `escape_compromised_shoot_first` (flag): 1 = skip 4.2 (as in stage 1) when
  the Phase verdict for this think is `Shoot` (no awake enemy adjacent;
  `phase_rule_verdict(st, PhaseSite::Compromised, Some(b_q))`, cached per
  think), the damage-only danger is below
  `escape_compromised_shoot_danger_percent` of current HP (default 100: the
  Borg survives the next turn), HP is at least
  `escape_compromised_shoot_hp_percent` of max (default 50), and
  `borg_attack_estimate` finds a ranged attack with positive return (the same
  estimate #625's fight test uses). `borg_caution` then reaches
  `phase_rule_act`, which attacks at range or boosts, exactly as the rule's
  note promises.
- D: 48 is below 68, HP is full and the earlier attack utility was positive,
  so the Borg shoots instead of teleporting. If the shot is not enough, the
  next think faces a higher danger and the earlier tiers (2.1, 3, tier 1)
  still escape.
- Verdicts other than `Shoot` (an enemy adjacent) never use this path.

### Stage 3: keep a charge in reserve

- `escape_compromised_staff_reserve` (charges, default 0): in 4.2 only, the
  Staff of Teleportation is skipped when it holds this many charges or fewer;
  spells, scrolls and activations are not touched. Tiers 1 to 3 may use the
  last charges. Hypothesis, value 2: in the recording, 1,403,464 would have
  left the staff's last charge for the danger-105 escape 177 game turns
  later (turn 1,403,641).

## Open questions

- **C may be a real threat.** The damage estimate counts bites at normal
  speed and not the group's speed or numbers. Stage 1 refuses it. Measure
  before adding a fast-group override.
- **B depends on the unseen-attacker guard**, which must be checked against
  the #831 state's reset rules.
- **A also needs its own fix**: a retreat that the next flow undoes
  (`retreat_oscillation_veto` exists, #932). Separate task; not in this plan.
- **Danger 2000** (turn 1,385,545, no monster named after an acolyte died) is
  the `danger_maximum` cap. Its source is not in the log; not in this plan.
- The case dossier (all 24 escapes, with the context of each) is not in this
  repository. It belongs in borg-cemetery, with the owner's evidence bundle
  referenced from issue #1342.

## Measuring it (borg-zoo, RULES 14 and 20)

Population comparisons go to borg-zoo as a request issue. Proposed arms: the
shipped settings; stage 1 alone; stages 1 and 2; stages 1 to 3, over all
classes (the tier is not Mage-only). The endpoint, its size and the
confirmation suite are the zoo's to prespecify before the run. Candidate
measures: deaths and the depth reached per game, escapes by tier and charges
and recharges per level gained, and deaths within a fixed number of thinks
after a refused 4.2. No improvement is claimed here.

## Implementation outline (a later PR per stage)

- `borg-core/src/borg_escape.rs`: a pure `compromised_teleport_open(...)` for
  the decision (unit-tested on the table above), wrapped around 4.2 and the
  flee/leave flags; the shoot-first test in the same function.
- `borg-core/src/params.rs`: the settings above, each citing the line that
  reads it (`scripts/check-param-citations.py`), with their `harness/pborg.yaml`
  entries at 0, and the entries in `docs/CALIBRATION.md`.
- `docs/PHASE-DOOR.md`: one line pointing the compromised site here.
- Default off keeps `harness/reference/smoke-borg-rs` unchanged; check it with
  the smoke diff, as the other settings do.
