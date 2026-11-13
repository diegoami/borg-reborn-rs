# Handover

*Where the work stands, for the next session. Update it in every PR.*

## 2026-11-13: eat when Hungry, light the way (#172)

**Play change in 39 full runs; new full baseline. Smoke and smoke-exact
unchanged.**

- **`eat_below_food` 300 → 1500.** The engine's grades (`player_timed.txt`
  × `food-value` 100): Faint 400, Weak 800, Hungry 1500, Fed 9000. 300 was
  below Faint, so the Borg could faint before it ate. The manual (c01
  §1.16) eats when Hungry appears. The return-home rule for "starving with
  no food" reads the same threshold, so it now starts with 1,500 turns of
  food rather than 300. The fixture default `fed()` was 500 (Weak); it is
  5000 now.
- **The starvations' root cause was darkness, not the food rule.** A
  burnt-out torch is deleted from the light slot
  (`player-util.c:player_update_light:713-726`, `OF_BURNS_OUT`) and nothing
  wielded the spares: **all 45 full runs lost their first torch**. The
  runs that starved (ranger-0, rogue-1) spent their last ~25,000 turns
  unlit, with two Wooden Torches and a Word of Recall in the pack. Unlit,
  `player_can_read` refuses the Recall, and the unknown grid next to the
  Borg is never seen, so Explore and Return flipped between two grids
  (Walk 9 / Walk 1) until the food ran out. Found from a rerun to turn
  75,340 with `BORG_OBSERVATION_LOG`: the light slot empty, the panel
  grid (26,39) unknown beside a floor.
- **`equipment::light_to_wield`:** an empty light slot takes the aware pack
  light with the most fuel, before every step but a monster in reach (not
  gated on a quiet grid: unlit is itself the danger). 44 such wields in
  the suite, none refused.
- **UNLIGHT is left unlit.** The necromancer (`class.txt:975`) sees nearby
  grids without light (`cave-view.c:777-779`) and pays +25 spell failure on
  a lit grid (`player-spell.c:416-419`). Lighting it cost all four
  necromancer runs depth or a level in a scratch run. `Class` gains the
  engine's `pflags` (`Class::has_pflag`, `PF_UNLIGHT` = 12).
- **Measured (full, 45 runs):** alive 34 → **39**; max depth sum 44 → 44;
  max level sum 52 → 54; no run worse. No starvations (was 2). The
  remaining deaths: mage-4 poison at 1,390; druid-0 and druid-1 a small
  kobold; druid-2 a crow; priest-0 a giant white mouse; priest-4 a
  battle-scarred veteran. All are early casters who do not cast yet (#127).
- Dossiers: D1842–D1847 for the baseline. D1828–D1841 are the deaths of the
  two intermediate commits of this PR (each a committed source, RULES 12).

**Defect 1 of #172** (Return must outrank a live explore goal) did not
reproduce once the light was fixed. The return-home rule fires in three
full runs (necro-3 at 76,810, rogue-1 at 75,801, ranger-0 at 75,302) and
each reaches town within 250 turns, where Town mode takes over. It is not
changed here.

## 2026-11-13: shooting and Speed (#127, T5.3, first slice)

**Play change in three full runs (the rangers); new full baseline.**

- **`Action::Shoot`** is `h`, `do_cmd_fire_at_nearest`
  (`player-attack.c:1412-1443`): the first quiver ammunition the launcher
  takes, at the closest visible monster, with no prompt.
  `Character::shot` reads the launcher's ammunition type (`KF_SHOOTS_*`,
  10/11/12) and multiplier (`pval`) from its kind
  (`player-calcs.c:2265-2275`) and the damage as `ranged_damage` does
  (`:554-582`). The tactical response scores the share of the target's
  threat one shot removes; at range, that is the round it deals once
  adjacent. Never-movers at range are not shot; blind, confused or
  paralysed, no shot. One shot a turn; the hit chance is not modelled
  (declared, as for melee).
- **A Potion of Speed** (+10, `player-calcs.c:2176-2177`) is offered at
  the threat the same grid has at speed + 10, for one consumable.
- **bbc D0010 is rebuilt from its dossier** (`overall-assessment.md:16`:
  Main Gauche wielded, Short Bow x2 in the bow slot, eleven 1d4 arrows in
  the quiver). The fixture had the bow as the weapon, the arrows in the
  pack and wrong kinds. It now uses the engine's kinds 34/77/81/247 (added
  to the fixture knowledge from a `BORG_KNOWLEDGE_LOG` dump), and is
  active: the Borg shoots the kobold. The fixture loader gains a quiver.
- **D0014** now offers Speed, but the dagger's one-hit kill of the 6-HP
  falcon outscores it, so it stays a gap (#127): the owner's Speed→CLW
  line is a sequencing question.
- The readiness ranged gate uses `Character::shot` (ammunition the
  launcher fires).
- **Measured:** 9 shots in the full suite, none refused. `full-7-ranger-4`
  survives (was a floating eye at 3,880); `full-7-ranger-3` survives;
  `full-7-ranger-0` now starves at 88,180 — the starvation residue,
  filed as **#172**. Survival 34/45. Full reference regenerated; dossiers
  D1817–D1827.

**Left in #127:** spells (study, cast, Magic Missile's evaluation, priest
prayer randomness, book priorities), devices with failure chances, thrown
oil, and D0014's Speed→CLW sequence.

## 2026-11-13: equipment valued by effect (#126, T5.2)

**Play change in two full runs; new full baseline.**

- **The engine's tvals** (`world::tv`, `list-tvals.h`). The observation
  carries the engine's own tval, but `tactics::better_weapon` filtered on
  23, the pre-4.2 sword, which in 4.2 is hard armour: no real weapon was
  ever offered. The test data and all 24 decision fixtures used the old
  numbers and are corrected (dagger 9; mace, maul, whip 7; bow 5;
  mushroom 29).
- **No wield ever worked in a game.** The prompt catcher escaped "Wear or
  wield which item?" unconditionally, the C Borg's rule for stumbling
  into it. Our Wield opens that prompt itself with the item letter
  queued, so the letter fell on the map as a pickup. The prompt is now
  escaped only when nothing is queued, and a refused wield is barred like
  any refused action (#137). Before this fix the scratch suite had two
  runs with about 17,600 refused wields each, both starving.
- **`equipment.rs`:** a pack item is worn when, on a copy wearing it,
  melee per turn, known AC and (for a caster) the armour-weight mana
  penalty (`calc_mana:1505-1531`) are all no worse and one is strictly
  better. A launcher is not a weapon; jewellery is worn only when fully
  known and only into an empty slot. The strategy wears it when the grid
  is quiet and nothing is adjacent (rule "a better item in the pack:
  wear it").
- **D0011 is active:** the fixture's Maul is the dossier's 4d4 (+7,+6)
  (`overall-assessment.md:26`), not 2d4. The loader carries a pack
  weapon's numbers, and the Borg wields the Maul. OWNERS drops #126.
- **Measured:** smoke 3/3 and smoke-exact 3/3 unchanged; full 43/45.
  Two wields in the whole suite, both worn. `full-0-warrior-3` wears a
  Cloak [1,+0] and survives as before; `full-2-druid-0` diverges after
  its swap and dies to a small kobold at 3,098 (was the mouse swarm at
  7,912). Survival 34/45. Full reference regenerated; dossiers
  D1806–D1816.
- **Not valued:** resistances, slays, brands. Pickups and pack weight are
  unchanged (a swap does not change total weight). The town buys no
  equipment, so reserves come first by construction.

**Next:** #127 (T5.3 item use, spells, ranged).

## 2026-11-13: depth readiness (#125, T5.1)

**Play change; new baseline.** One readiness policy for every way down
(`readiness.rs`, from the strategy manual):

- **DEPTH-01:** depth *d* is allowed when the worst single turn
  (`danger::worst_turn`, c15 DANGER-01's `T_max`: moves times the larger
  of the full melee round and the largest spell) of every race native to
  *d*−2 … *d*+5 stays below `depth_hp_frac_pct` (50) of maximum hit points.
- **DEPTH-02:** the DL5, DL15 and DL20 kit gates (Recall, three cure
  potions, a ranged attack; 100 max HP and ten Phase Doors; Free Action).
  Kinds are matched by name *and* tval: Cure Light Wounds is also a staff,
  and a flask is tval 27. The DL25+ rows are not checked yet and fail.
  DL1 is always allowed.
- **Strategy (DEPTH-04):** not ready for the current depth → Return (go
  up); ready for depth+5 → dive before exploring; a down staircase only
  when ready for the next depth, otherwise an explored level rerolls
  upward under the existing rule.
- **Not covered:** trapdoors (the observation has no trap kind; a fall to
  an unready depth is answered by "go up"); Deep Descent (target
  `max_depth` + 5, but the Borg reads none today); a recall down (never
  read).
- **Declared parameters:** `depth_hp_frac_pct` 50 (the manual's
  `K_DEPTH_HP_FRAC`; its 0.3/0.5/0.7 sweep is not run),
  `readiness_band_below` 2, `readiness_band_above` 5.

**Measured (the declared change), full suite:**

| | before | after |
| --- | ---: | ---: |
| alive at the cap | 30/45 | **34/45** |
| mean max depth | 1.20 | **0.98** |
| mean max character level | 1.49 | **1.20** |
| deepest level | 3 | 2 |

The gate trades progress for survival. With the band reaching *d*+5,
Fang (DL5, +20, a 1d8 bite: 16 in one turn) keeps DL2 closed until about
33 maximum HP. On DL1 the Borg gains little experience, so most
characters never get there. "Not ready: go up" fired once in the suite
and "dive" never; the whole effect is the down staircase not taken.
`full-6-rogue-1` now starves on DL1 (the T4.6 residue: a Return goal that
never finds the way up, 1,100 turns, with 69 gold). The smoke mage
survives (it was Fang at 18,376). References regenerated (smoke 1,
smoke-exact 1, full 13 runs changed); dossiers D1795–D1805.

**For the owner:** progress below DL1 now needs experience the Borg does
not yet seek. #126 (equipment) and #127 (item use, spells, ranged)
should raise it; `depth_hp_frac_pct` and the band are the dials if the
gate is too strict, to be measured, not tuned by hand (RULES 14).

**Next:** #126 (T5.2 equipment and pack valuation).

## 2026-11-13: M5 source refresh (RULES 21)

**No play change.** M5 starts with its `--update` PR.

- **Pins moved:** borg-rs `4ed3263`, better-borg-c `4603703`,
  neo-angband-tracker `c2ddf2c`, borg-zoo `c5f1b04` and borg-runner
  `e55ac6f`. The copies in `reference/` and the death index are refreshed.
  Upstream Angband stays at `e473720` (the engine contract).
- **borg-agent retired:** its pin is dropped, with its mentions in RULES 7
  and 21 and the refresh script's docstring (left for this PR by #152).
- **Read:** recorded in `docs/research/KNOWLEDGE.md` (reading-state rows
  and an "M5 source refresh" note).
  - **borg-runner edition 7:** readiness is the destination's worst
    round, not its experience (`mmf-mage.3`); Deep Descent is `max_depth`
    + 5 (`effect-handler-general.c:1166-1168`); class kit facts for
    Priest, Ranger and Mage.
  - **better-borg-c:** no new deaths. `borg_guess_race` moved to lines
    874-889, so `observation.rs` cites the new lines.
  - **borg-zoo:** our milestone-E specimen is catalogued, there is a
    pristine `classes-1800` rerun (a G1 control candidate, not yet read),
    and the `clevel` stop mapping (borg-zoo#48), so the stale note in
    `docs/ZOO-INTERFACE.md` is corrected.
  - **borg-rs:** v11R carry; munchkin default back to FALSE.
- References reproduce 3/3, 3/3, 45/45; dossiers D1779–D1794 (the new
  source hash).
- The issue sections of the refresh report fail from cloud sessions
  (GitHub GraphQL is refused there); issues were not read on this
  refresh.

**Next:** #125 (T5.1 depth readiness).

## 2026-11-13: the M4 review's notes (#165)

**No play change** (measured): smoke 3/3, smoke-exact 3/3, full 45/45
identical to the references.

- `danger.rs`: sleep and confusion set the rounds a monster spends
  attacking, and expected, worst, `p_death`, `disabling` and `economic`
  all read them. Before, confusion scaled only expected and worst, and
  sleep left `disabling` whole. Stun is applied where the engine applies
  it: the hit chance falls by `STUN_HIT_REDUCTION`
  (`mon-attack.c:353-356`) and the rolled dice by `STUN_DAM_REDUCTION`,
  before armour (`:621-624`). Test: `confusion_and_stun_scale_every_term`.
- `strategy.rs`: `has_food` and the Eat step require an aware item (test
  `only_known_food_is_food`). The refusal bar's documentation names its
  granularity: an exact `Action`, pack index included; bar by kind if
  item refusals recur.
- `tools/calibrate.py`: only the engine's melee blow phrases
  (`blow_methods.txt` `act:` lines) and "misses you" count as blows. The
  blow decomposition is unchanged on this baseline (no spell line had
  matched).
- Dossiers D1763–D1778; `check-no-time.py` 0 violations.

**Next:** the M5 source refresh (its own PR), then #125.

## 2026-11-13: a wide blow is a distribution (#164)

**No play change.** M4 review finding 2. `danger.rs:blow_pmf` returned
`[0, mean]` for blows with dice × sides > 64: probability mass `mean` at
damage 1, which is not a distribution, so `p_death` came out clamped to 1.
The declared two-point model is now a real one: P(max) = mean / max and the
rest on 0. It sums to 1 and keeps the mean; the spread is overstated, which
is conservative. Only depth ≥ 36 monsters have such blows (20d10 at most).
Test: `a_wide_blow_is_a_distribution`. References reproduce 3/3, 3/3,
45/45; dossiers D1747–D1762.

## 2026-11-13: M4 reviewed and merged; growth factors are declared (#163)

**M4 review.** GLM-5.3 reviewed #162 at `612fa97`. It reproduced every
reference, the G0 checks, all 16 dossiers and every calibration number,
and gave the verdict: contract met, nothing blocking. The owner merged it
(`reviewed` advances by merge commit `ae5aa11`). **The `m4` tag could not
be pushed from the cloud session** (the proxy refuses tag refs and the tag
API); the owner creates it at `ae5aa11`. The findings are M5 issues #163,
#164 and #165.

**#163 (no play change):** the growth factors' citations are corrected.
The C Borg's code at `borg-danger.c:borg_danger_one_kill:2420-2436,
2700-2710` computes ×2.5 (breeders), ×2.8 / ×2.3 (monsters with friends)
and ×2.2 (its second pass). Its comments say +50%, +80% and +30%. Our 1.5
and 1.8 follow the comments; the 1.5 for summoners from CL15 is ours; the
C Borg has no summoner factor. `danger.rs` and the calibration report's
threshold table now call them declared. References reproduce 3/3, 3/3,
45/45; dossiers D1731–D1746 (the new source hash).

## 2026-11-12: remembered casters fade with confidence (#157, T4.8)

**Play change in one full run; new full baseline.**

- `danger.rs:one_threat` scales the ranged half of a monster's threat by
  `Track::confidence`, as it already scaled the melee half. It also scales
  `p_death` in both halves: the chance the monster is there times the
  chance it kills if it is. The melee `p_death` was unscaled too, a
  sibling of the same defect, fixed together here. Test:
  `a_forgotten_caster_adds_almost_nothing`.
- **Measured:** smoke 3/3, smoke-exact 3/3; full 44/45 identical.
  `full-6-rogue-3` leaves its 75,000-turn loop around three forgotten
  apprentices, reaches character level 3 and dies to Fang at 25,237
  (p_death 0.93 at the fatal think; it fought on at 1 HP). Full survival
  is 30/45, the T4.7 rate. The full reference is regenerated; dossiers are
  D1715–D1730.
- **Calibration:** pooled ratio back to 0.083 (it was 0.017 with the loop);
  remembered out-of-view sources 893 windows at 0.015. Addendum in
  [docs/M4-CALIBRATION.md](M4-CALIBRATION.md).

**M4's tasks are all closed with this PR.** Next is the M4 review PR
(`main` → `reviewed`), opened by the session with a review prompt. The
owner chooses the reviewer (RULES 6).

## 2026-11-12: the M3 review's small findings (#139)

**Play change in three full runs; new full baseline.** Five cleanups:

- `tactics`: responses and the weapon swap sort with `total_cmp`, through
  `ord()`, which folds -0.0 into 0.0. Ties then fall through to risk, gain
  and action rank exactly as under `partial_cmp`.
- `strategy`: Engage's progress measure has its own sentinel
  (`ENGAGE_NO_CONTACT`) when no monster is adjacent. Before, the empty case
  was 0, which reads as a monster at 0 HP; a track's `hp` is 0 when its
  health is unknown, so "no contact" and "contact with an unmeasured
  monster" looked like no progress.
- `strategy`: Explore's fallback stands still through `still()`, which
  never opens a shop by holding on its entrance.
- `executor`: Sell sends quantity digits and the prompt's Enter only for a
  stacked item. A single item has no quantity prompt, and digits there
  would be read as commands.
- `strategy`: `retreat_step`'s documentation states its tie-break: the
  first direction in `DIRS`, keypad order.

**Measured:**
- Smoke 3/3 and smoke-exact 3/3 are unchanged. Full: 42/45 unchanged.
- Bisected on seed 1002, the Engage sentinel alone accounts for all three
  differences: `full-0-warrior-2`, `full-0-warrior-3` (both still survive)
  and `full-2-druid-0` (dies to the same giant white mouse swarm at 7,912
  instead of 6,943). An Engage goal now keeps running while monsters come
  and go, instead of stalling.
- Full survival stays 31/45. The full reference is regenerated; dossiers
  are D1700–D1714. `check-no-time.py` finds 0 violations and
  `reconcile-deaths.py` is clean.
- Regression tests: `scores_are_totally_ordered_and_zero_has_one_sign`,
  `engage_progress_without_contact_is_not_a_dead_monster`, and the
  single-item Sell keys.

**Next:** #157 (remembered casters never fade), then the M4 review.

## 2026-11-12: the trace says why, and only scored candidates (#138)

**No play change; same baseline.** The M3 review's finding 3. The first
strategy chooses its mode by a priority rule, so the trace's single
candidate at score 1.0 claimed an alternative that was never weighed.

- `candidates` now holds only scored alternatives, which are the tactical
  responses of a contact think (T4.3). A rule-based think records none
  (in `full-0-warrior-0`, 9,385 of 9,522 thinks).
- New trace field `why` (`trace::Why`): the `choose_mode` rule that fired
  (for example "contact: a monster in reach", "a live goal keeps its
  mode"), the modes cooling down, and the goal's `goal_started` and
  `goal_progressed`. `Goal.started` is now read: age is
  `turn - goal_started`, stall is `turn - goal_progressed`. Old traces
  parse (serde defaults).
- Checked: smoke 3/3, smoke-exact 3/3, full 45/45 identical to the
  references; `check-no-time.py` 0 violations. The new source hash gives
  the 15 deaths new dossiers, D1685–D1699 (same games as D1670–D1684).
- **Next:** #139 (small cleanups), #157 (remembered casters), then the M4
  review.

## 2026-11-12: a refused action changes the next decision (#137)

**Play change; new full baseline.** The M3 review's finding 2: contact
returns Engage before any cooldown, so an action the executor reported
refused was chosen again. In the M4 baseline the refused action was a Read
the engine forbids.

- **`danger::can_read`** mirrors `player_can_read`
  (`player-util.c:1166-1197`): blind (`TMD_BLIND` 2), confused (4), amnesiac
  (25), or no light. No light is `no_light`, `!square_isseen` at the
  character's grid (`cave-view.c:932-935`); the observation carries it as
  `in_view` (`cave-map.c:119`) and the world as `seen == Some(turn)`. It
  gates the tactical Phase Door, Teleportation and Recall, and the
  Return/Stuck recall read.
- **Refusals bar the action:** the strategy remembers its last action, and
  a refusal bars that action until the mode's cooldown ends. Engage then
  takes the next-best response. Contact still overrides Engage's cooldown,
  which is the justification the issue asked for.
- **Measured:** two full runs change, and smoke and smoke-exact are
  identical. `full-6-rogue-3` survives the cap (was D1667: confused, it
  re-read a refused Phase Door for forty turns before a cutpurse killed
  it). `full-7-ranger-0` no longer reads Recall in the dark 13 times, and
  still starves at the same turn. Refused-read violations: 23 → 0. Full
  survival is 31/45. The full reference was regenerated at `798239a`, and a
  fresh run reproduces it 45/45 (smoke 3/3 against the kept references).
  Dossiers D1670–D1684; `reconcile-deaths.py` is clean.
- **Found: #157.** The rogue's survival is a 75,000-turn walk loop around
  three remembered apprentices. `one_threat` scales melee by track
  confidence but not the ranged estimate, so a forgotten caster keeps
  `expected 1.8`, `worst 14.4` at confidence 0.00015. It is opened as T4.8
  in M4. The calibration report has an addendum for this baseline.
- **Next:** #138, #139, #157 (M4), then the M4 review.

## 2026-11-12: threat calibration and the M4 survival gate (#124)

**No play change; same baseline.** T4.7 measures the threat model on the
M4 baseline's games and checks the M4 exit. The report is
[docs/M4-CALIBRATION.md](M4-CALIBRATION.md).

- **The trace names its monsters** (`danger.rs:summary_detailed`):
  `threat.sources` holds one line per monster with any non-zero term
  (race, distance, speed, sleep, confusion/stun, in view, same-race group,
  rounds, expected, worst, disabling, ranged). A floating eye's gaze has
  no dice, only paralysis, so it counts. The trace is written after the
  decision: the build reproduces smoke 3/3, smoke-exact 3/3 and full 45/45
  byte for byte. The source hash changes, so the 16 deaths have new
  dossiers, D1654–D1669 (same games as D1638–D1653).
- **`tools/calibrate.py`** (with tests): windows cut on the key stream, not
  on turns (`game-world.c:run_game_loop` lets faster monsters act before
  the next key). It reports predicted against taken damage by
  expected-damage bin, range, speed, status, attack form, group and choice;
  p_death against deaths; disabling against onset messages (hunger faints
  apart); a blow decomposition from `monster.txt` and the engine's hit and
  miss messages; bootstrap intervals; named outliers.
- **Found:** expected damage is about 10× what is taken (0.087, 90% CI
  0.07–0.10). Per blow the model is right (1.70 HP predicted per blow
  against 1.77 observed, hit rate 0.92). The gap is blows that never come:
  the Borg's own blow kills or frightens the monster first in 48% of
  standing windows, and survivors land 0.58 of the predicted blows (the
  low-HP `+3` speed and erratic movers are candidates). Worst case is
  exceeded in 0.14% of windows, only by unmodelled damage. 11 of 16 deaths
  came with p_death ≥ 0.2 and the Borg fought. The other 5 were
  starvation ×2, poison, an unseen biter while resting, and a floating eye.
  For the eye, the engine deals ≥ 1 HP per paralysing blow to a character
  already paralysed (`mon-blows.c:1019-1021`), and the Borg rested beside
  it at 1/15 HP.
- **Gate:** fixtures pass (21 active; the 3 gaps belong to M5), replay is
  deterministic, G0 holds with a 10-second stall timeout, and every death
  has a dossier. Survival 30/45 (67%) beside the zoo's C Borgs at 66%
  (4.2.6) and 69% (bbc-m7) of `classes-1800`, descriptively. Our Borg gets
  there at median depth 1 and character level 1; theirs reach depth 1–4
  and level 1.5–7.
- `decision_fixtures.rs` OWNERS drops #124.
- **Refresh report:** angband +12, borg-rs +21 (2 copied docs changed),
  better-borg-c +10 (new tags `4.2.6-impact-v8`…`v11`, `v9.1`),
  **neo-angband-tracker +52** (the T4.6 entry said unmoved), borg-zoo +33
  (catalogue changed), borg-runner +26 (new records and knowledge files);
  borg-agent unmoved. The report's issue sections fail from cloud sessions
  (GitHub GraphQL is refused there; the script would need the REST API).
  Pin moves are their own PRs (RULES 21).

**For the owner:**
- **M4 is not finished.** The milestone still has the deferred M3 review
  findings **#137** (Engage cooldown after refused actions), **#138** (real
  decision candidates and goal timing) and **#139** (small cleanups). The
  last handover said #124 was M4's last task. These come next. #137 may
  change play, and then the gate's survival table needs a rerun (one
  command, in the report). The M4 review PR waits for them.
- The report's *What follows from it* lists four play changes the
  calibration supports: blow frequency, the paralysis floor, unseen
  attackers, and status damage. Each needs its own baseline. Decide
  whether they belong to the M4 review or to M5.
- This session pushed from `ccr-e0e99f46-go3y0w`, not `task/T4.7-…`: the
  cloud session can push only to its own branch.

## 2026-11-12: recovery, return home and progress under oscillation (#123)

**Play change; new baseline.** T4.6 closes M4's behaviour half: the Borg
recovers under pressure, goes home when the level cannot feed it, and digs
out of sealed rooms. Smoke, smoke-exact and full references are regenerated
in this PR; the diff against the old baseline is the declared change.
(T4.4's merge line, carried along as announced: PR #154, merge commit
f7f8a689; main is now 1915f091 after the docs commit.)

- **The `return` mode** (`strategy.rs`): starving with no known food at
  depth outranks recovery itself and bans descending (bbc D0009's full-pack
  starvation, rust D1420's descend-on-the-staircase). The goal follows the
  manual's recall-and-home routine (c15 §12.12): the up staircase underfoot
  is always taken; a known Word of Recall is read early and only in safety;
  otherwise travel to the up staircase, explore for one before any
  down-stair fallback, then dig.
- **Threatened travel** (`travel_step`): a path step that lands on a grid
  more dangerous than where the Borg stands is refused, on the tactical
  layer's composite scale (`tactics.rs:composite`, now shared) — the u32
  `danger_with` rounds a small risk to zero and has no disabling half, so
  bbc-d0006's NEVER_MOVE mold (confusion, no hit-point risk) read as a free
  walk north. The retreat step reads the same scale.
- **Heal-while-fleeing**: a live Recover goal with a monster about rests
  anyway when no next blow can kill (`worst < hp`), letting the engine's
  disturbance interrupt the rest (c15 REST-01, `src/player-util.c`; bbc
  #408/#409) — a retreat loop (rust D1330) makes no progress under
  oscillation; a rest does.
- **Stuck escalates** (`dig_step`): with no staircase and no frontier
  reachable anywhere, the stuck goal digs (`borg::diggable` — rubble,
  magma, quartz, granite carry HURT_ROCK, engine `list-terrain.h:29-34`),
  walks to a diggable wall, or reads a known recall scroll; it no longer
  waits to starve (the M3 re-review's four sealed-room runs).
- **Fixtures:** `bbc-d0006-deep-descent-readiness`,
  `bbc-d0009-full-pack-starvation`, `rust-d1420-starvation-no-food` and
  `rust-stuck-sealed-room` are **active** (the sealed-room fixture is now
  the 3x3 its construction note always described). #123 leaves the gap
  OWNERS: #124, #126, #127.
- **Measured (the declared change):** smoke — the mage lives forty times
  longer (451 → 18376 turns, reaches depth 2, dies to Fang; dossier
  D1638) and the priest now survives the cap (was a grey mushroom patch
  at 3586); smoke-exact diverges at the identical turns, so determinism
  holds across both profiles. Full — 30 of 45 runs survive the 10001
  player-turn cap (was 11 of 45); the 15 natural deaths are dossiered
  (D1639–D1653) and `reconcile-deaths.py` is clean. Two late
  **starvation** deaths (full-5-paladin-2 at 91190, full-7-ranger-0 at
  89750 game turns) are the honest residue: runs the old baseline lost
  to early monsters now live long enough to starve; the M4 survival
  gate (#124) reads this baseline.
- **Refresh report:** angband +12 (head 42cd156a), borg-rs +21 (2 copied
  docs changed), better-borg-c +10, borg-zoo +33 (catalogue changed),
  borg-runner +26 (new records and knowledge files); borg-agent and
  neo-angband-tracker unmoved. Pin moves are their own PRs (RULES 21) and
  are not in this one.
- **Next:** #124 (T4.7 threat calibration report and the M4 survival gate,
  closing M4). #126/#127 are M5 and stay untouched.

## 2026-11-12: the manuals named as analysis sources

The owner asked that the repository make its manual sources visible to every
session: the **Angband Strategy Manual** and the **official user manual** in
`reference/c-manual/` (S14). They are now named where a session starts and
where the rules live:

- `AGENTS.md` reading list, as an entry of its own, and a new *Game
  knowledge, and where analysis starts* section: engine first, manuals
  second, recorded play third;
- `RULES.md` 16: read the relevant chapter before designing a rule, then
  check every number in `engine/angband/` and cite the engine;
- `README.md`: a *Game knowledge* section, and the manuals in the
  neo-angband-tracker row;
- `docs/SOURCES.md` S14 title and `docs/research/KNOWLEDGE.md`'s reading
  state: the user manual is read with the strategy manual, per task.

No pin, copy or code changed. `cargo test --workspace` checks the documents'
links (`borg-reborn-rs/tests/repository.rs`).

## 2026-11-12: the escape ladder (#121)

**Play change; new baseline.** T4.4 replaces the staircase rule and the
Phase placeholder with the escape ladder: step, Phase Door, Teleportation,
Word of Recall and the stairs are all scored on the one hit-point scale by
cost and sufficiency, not by a fixed monster list. Smoke, smoke-exact and
full references are regenerated in this PR; the diff against the old
baseline is the declared change.

- **The ladder** (`tactics.rs`): a relocation's gain is the threat it
  removes — a step's exactly as an escape's. A blink's `risk_after` is the
  mean assessment over the remembered enterable grids of its landing band
  (engine distance 8-12 for Phase Door's 10±25%; 45-75% of the longest
  cardinal edge distance for Teleportation's 60%±25% —
  `effect-handler-general.c:effect_handler_TELEPORT`); grids a visible
  monster stands on are skipped, and an unestimated landing (an empty
  band) is not an offered escape. A stair's landing is the declared
  `arrival_risk_hp` — 0 for up-to-town — and Word of Recall's is the
  25-turn mean wait beside what stands there; an active word is not
  re-read (reading cancels it). Walking stays available after the last
  consumable.
- **Last-item protection:** a consumable escape pays `escape_item_hp` (1);
  the last known escape scroll pays `last_escape_hp` (5) more. The last
  Phase Door is kept beside a dying yeek and a never-moving eye, and still
  spent at 4/16 HP against Fang.
- **Never-movers give no parting blow** (`danger.rs:never_move`): a step
  out of `RF_NEVER_MOVE` reach ends the threat for good (bbc D0013).
- **Ties end threats:** score ties break by smaller `risk_after`, then
  larger gain, then Attack over Walk (GOALS principle 6: killing the mold
  beats stepping off it, bbc D0012).
- **New parameters** (`params.rs`): `escape_item_hp` 1, `last_escape_hp`
  5, `arrival_risk_hp` 3 — declared, to be measured (#124). Also fixed a
  latent panic: `from_profile`'s match lacked six table names
  (`eat_below_food`, `risk_aversion_pct`, `threat_horizon` among them), so
  a profile setting one hit `unreachable!()`.
- **Fixtures:** three new active ones — `bbc-d0013-eye-step-not-phase`,
  `bbc-d0017-faster-pursuer-walk`, `bbc-d0018-dying-yeek-phase`.
  `bbc-d0014` is re-pinned (Attack now; the fatal Read ended; Speed/CLW
  stay #127). `bbc-d0012`'s worm masses moved to distance 2, where its own
  construction note always said they were; `fang-ac36` and
  `fear-blocks-the-attack` were widened to 15 columns so the blink band
  has remembered floor (both construction notes amended). #122 is closed,
  so the gap OWNERS are the open tasks only: #123, #124, #126, #127.
- **Refresh report:** angband moved +12 commits past the pin, borg-rs +18
  (one copied doc changed), better-borg-c +10 with new milestone tags
  `4.2.6-impact-v8`…`v11` (v9.1 among them — new zoo specimens to
  consider). Pin moves are their own PRs (RULES 21) and are not in this
  one.
- **Next:** #123 (T4.6 recovery/oscillation, owns four gap fixtures), #124
  (T4.7 calibration and the survival gate — the smoke mage's death to a
  grey mold's spores at 9/9 HP is calibration's first exhibit), #127
  (T5.3, owns d0010/d0014's class answers).

## 2026-11-11: play mode for models; borg-agent retired (owner, D11/D12 amended)

The owner wants models to be able to **play** through `borg-mcp`, not only
query it, and says borg-agent was a failed experiment that would need a
rebuild. Documents only:

- **GOALS:** D12 now covers a query *and play* surface; constraint 2 is
  "the measured Borg plays without a model" (play mode is a separate arm,
  logged and replayable, bound by D3 and D5); new roadmap stage **Q5,
  play mode**, and Q4 (live) is no longer optional, since Q5 needs it.
  *(Superseded the same day by the reorder below: Q4 became the diagnosis
  workflow, live hosting moved into Q5, and play mode is optional.)*
  D11: borg-agent retired as an input, its roles move to Q; borg-runner
  stays. Option E and the non-goal reworded to match.
- **RULES 24–25:** the zoo specimen is always the model-free Borg and the
  Borg's code never calls a model; a model game is a separate arm with its
  model, effort and tool calls recorded; in play mode the model chooses
  only executor actions, and rules 11, 12 and 15 bind it.
- **PLAN:** Q5's row and exit; Q4 no longer optional; the Neo Angband
  line no longer mentions borg-agent.
- **docs/QUERY-SURFACE.md:** *Play (Q5)*: tools `observe`, `options`,
  `act`, `autopilot` (hand-off conditions), three arms (model, hybrid,
  advised), the run record (`model.json`, `calls.jsonl`); *Workflows*:
  explaining a death, triage, reviewing a change, hybrid play, and which
  model for each; two new checks (play replay, arm separation).
- **SOURCES S7, ARCHITECTURE L3, AGENTS:** borg-agent retired; its pin is
  dropped in the next `--update` PR (RULES 21: pin moves are their own PRs).

**For the owner:** nothing new to decide before Q starts. The hand-off
conditions' defaults are an open question for Q5.

**Same day, Q reordered around diagnosis (owner).** The MCP's purpose is
diagnosing the model-free Borg's deaths after the fact; play mode is
optional and last. New roadmap: **Q0** pilot (no code, #153: diagnose
deaths from the dossiers as they are, measure a hit rate on known deaths,
list the missing tools), **Q1** facade (#149–#151, unchanged), **Q2**
diagnosis tools, **Q3** counterfactuals with **`branch`** (rerun the
seed, force an alternative action at a think through a `branch` cargo
feature the zoo never gets, let the Borg continue, report what followed;
one sample, not a probability), **Q4** the diagnosis workflow and its
evaluation, **Q5** play mode, optional, decided after Q4. The primary plan
(M4–M7) is unchanged.

**For the owner:** create the GitHub milestones Q0–Q5 (this replaces the
earlier ask for Q1–Q4): #153 goes into Q0, #149–#151 into Q1; open Q0 when
you want the pilot to run (it can run beside M4: it changes no code); fix
the hit-rate threshold for Q4's evaluation, and which cases it counts,
ideally after seeing Q0's numbers.

**#152's review (owner and a second reviewer), fixed in the same PR:**
- Q0's ground truth: the reference dossiers hold no trace or parameter
  table, so calibration is by deciding *turn*, cause and alternative, on
  the set #153 names;
- our dossiers keep the last 200 thinks (`tools/export-death.py`), so the
  triage gives the deciding think where the trace reaches it and the turn
  otherwise, and a longer trace is a named Q0 output;
- `branch` builds are marked `features=branch` in `BUILD_ID`, since the
  source hash cannot tell feature builds apart; the zoo-contract refusal
  is a new assertion to add in Q3, not an existing check;
- RULES 24's scope restored to every measured game build (ours or the
  zoo's), matching GOALS constraint 3;
- "the server does not link the engine" replaces "engine-free";
- the Q4 threshold is marked open in GOALS and PLAN;
- QUERY-SURFACE's mode headings say *query mode* and *play mode*, as the
  rules do;
- draft fixtures are not committed as tests;
- README no longer lists borg-agent as an input.

Left for the pin PR (RULES 21): `pins.json`, RULES 7 and 21, and
`scripts/refresh-sources.py`'s docstring, which still list borg-agent.

## 2026-11-11: no control runs requested from the zoo (owner)

The owner: borg-zoo already runs the original C Borg and better-borg-c in
its other experiments, so this repository does not request runs of them.
GOALS (observation rule, G1 prerequisites, G1/G2, D8), SPEC §6 and the M4
and M5 exits now say that a zoo run we ask for is of our Borg only, and
comparisons use the zoo's existing control results. Issues #124 (T4.7),
#128 (T5.4) and #131 (T6.3) are updated to match.

## 2026-11-11: secondary goal Q, the query surface (owner decision D12)

The owner asked for the Borg's judgement to be exposed to language models
over MCP, as a secondary goal with a roadmap, and for the rules to change
to allow it. Documents only, no code:

- **GOALS.md:** new section *Secondary goal Q* with its five constraints
  and the Q1–Q4 roadmap; the non-goal "a model in the play loop" stays and
  is clarified; D11 amended; **D12** added.
- **RULES.md:** RULES 2 lets a session work on a Q milestone once the owner
  opens it, never mixed with primary work; new RULES 24 (the Borg plays
  without a model; the core never depends on `borg-mcp`) and 25 (no
  judgement in the server; the versioned `query` facade).
- **PLAN.md:** *Secondary track Q* with milestones Q1–Q4 and their exits.
- **docs/QUERY-SURFACE.md:** the architecture draft. Decision: same
  repository, separate crate (`borg-mcp` depends on the core's new
  `query` facade, never the reverse); replay mode first (the
  `BORG_OBSERVATION_LOG`, `BORG_WORLD_LOG`, `BORG_KNOWLEDGE_LOG` logs and
  `trace.jsonl` already hold what is needed); live mode optional, one game
  per process.
- **ARCHITECTURE.md:** a pointer to the draft.

**Review (#148, owner's comment):** three findings, all fixed. (1) Q1's
exit said "the harness build's identity is unchanged", but
`harness/identity.sh` hashes all of `borg-reborn-rs/` and `Cargo.lock`, so
adding `query` changes it; the exit and the draft's checks now ask for
the references to reproduce play, and say a changed source hash with
unchanged play is not a new baseline (RULES 22). (2) RULES 2's wrap. (3)
The replay-determinism check was in both TQ1.3 and Q3's exit, and the
draft said CI from Q2; it is now Q1's alone.

**Found:** a replay rebuilds `World` from observations, but not the
strategy's own state (mode, goal, `Memory`, RNG). The draft replays the
strategy too, with the run's seed and profile, and checks each choice
against the trace, which makes every replay a determinism check.

**Q1's tasks are filed** (#149 `run.sh --record-state`, #150 `query::State`,
#151 strategy replay), labelled `task` but in no GitHub milestone: this
session cannot create one.

**For the owner:** create the GitHub milestones Q1–Q4 (move #149–#151 into
Q1) when Q is to start (the plan says after M4 is merged); decide after Q3 whether
Q4 (live) is wanted. The draft's open questions (transport library,
snapshot interval, `what_if` look-ahead, live pause point) are settled in
the stage named there. M4 continues unchanged.

## 2026-11-10: item actions and the escape items (#122)

The executor now has **Quaff**, **Read**, **Eat** and **Wield**: the
pack letter, the command, and an effect checked next think (the kind's
count fell, or the kind is in an equipment slot). The tactical layer uses
them:

- a known Cure Light Wounds is scored by hit points restored (2d8 mean,
  declared) and chosen when it beats the alternatives; bbc D0002 and
  rs D0005 now quaff it;
- a known Phase Door is scored by the risk it removes, with landing risk
  half for now; Fang AC36 reads it, and fear accepts it;
- a better pack weapon is scored by the rounds it saves on the most
  threatening adjacent monster;
- a hungry character with a known ration enters `Mode::Eat` before the
  timer reaches starvation (`eat_below_food`, new parameter, declared).

**Fixtures:** 14 active, 7 gaps. New active: D0002/rs D0005 (known cure),
Fang AC36 (known Phase), `rust-hungry-eats` (food). Fear accepts the
Phase escape. D0014 now pins the **last Phase at full HP** for #121
(landing risk and last-item protection); D0011's Wield is still not
chosen over the attack; D0010/shoot and casting stay #127; the
unknown-food/starvation and return-home gaps stay #122/#123.

**New baseline (play change).** References: smoke 3/3, smoke-exact 3/3,
full 45/45; boundary 3/3. 38 natural deaths, 0 unexported; 36 new
dossiers D1566-D1601. Smoke has one 20,000-turn survivor and two deaths;
**the full suite is 34/45 deaths and 11 survivors at the 100,000-turn cap**,
deepest depth 3, with 3 starvation and 10 floating-eye deaths. That is a
real behavioural change, not a survival claim: #124's calibration and the
M4 gate judge it, and the zoo has not measured it. The floating-eye
deaths are the escape-ladder case (#121), not a cure failure.

**Sequencing.** #121 was started first and parked: every escape-ladder
fixture but walking needs Phase/teleport actions, so #122 went first by
owner decision. #121 resumes with real actions next.

**Next:** #121 (T4.4), escape ladder and landing risk: last-item
protection (D0014), the floating-eye case, and a real landing-risk
estimate for Phase/teleport and level changes. Then #123.

## 2026-11-10: tactical alternatives and the re-review (#120)

`tactics.rs` replaces the first-applicable fight-or-retreat rule. Every
think in contact generates and scores, on one hit-point scale:

- an attack on each adjacent monster (only when `can_attack`; fear and
  paralysis are refused by the engine), gain = the share of its threat the
  attack removes;
- a step to each of the eight neighbours, with the free hit every adjacent
  monster gets when the step leaves its reach;
- the up staircase underfoot when a threat is present (landing risk is
  #121; a down staircase under fire is still not taken);
- holding.

The score is `gain - risk_after`, using T4.2's assessment, the one
`risk_aversion_pct` and `threat_horizon`. A disabling or summoning risk
costs expected hit points in the score (a declared upper bound), so a
paralysis gaze is not free. Every response, including the losing ones,
goes to the trace; #138 narrows the remaining contract.

**Fixtures.** Ten active, ten gaps. Promoted: rs D0006 (kills the 2-HP
mould), rust D1358 (attacks the 2-HP mouse instead of taking its free
hit), rust D1413 (ascends with Fang adjacent and the stair underfoot).
D0013/breeder accept the scored fight/step. The re-review's four
sealed-room starvation runs are attached to #123; `Mode::Stuck` now names
the state (`stuck` / "no mode can act") and `rust-stuck-sealed-room` plus
`a_sealed_room_names_the_stuck_state` pin it. Search/dig/recall is #123.
D0010 and Fang AC36 are gaps for #127 (shoot/cast); the cures and the
return home are #122/#123.

**New baseline (play change).** References regenerated: smoke 3/3,
smoke-exact 3/3, full 45/45; boundary 3/3. 49 natural deaths, 0
unexported; 47 new dossiers D1519-D1565 from the clean checkout. Smoke has
one 20,000-turn survivor and two deaths; full is 45/45 deaths, deepest
depth 3, longest 89,380 game turns, **20 starvation and 8 floating-eye
deaths**. No survival claim: food is #122, the floating-eye/paralysis
escape ladder is #121/#122, and the stuck exits are #123. The re-review's
finding 2, G0's missing smoke-warrior death and the four sealed-room runs
are recorded; the owner's #135 scope choice remains the owner's.

**Next:** #121 (T4.4), the escape ladder with landing risk; it should take
the stair-under-contact response beyond "upward only" and give the
floating-eye deaths an escape. #122 then adds the actions; #123 the stuck
exits.

## 2026-11-10: one threat assessment (#119)

`danger.rs` is now L4's assessment rather than the C Borg's full-damage sum.
For each grid and horizon it returns:

- `expected` damage: the engine's hit chance (`chance_of_monster_hit_base`,
  `player-attack.c:hit_chance`) and armour (`adjust_dam_armor`) over the
  attack rounds from the energy model;
- `worst`: every blow hits at its largest roll;
- `p_death`: exact convolution for small fights, a declared normal
  approximation for large ones;
- `disabling`: fear/blind/confuse/paralyse from blows and spells;
- `summoning`: shrieks and `S_*` spells;
- `economic`: theft, destruction and drains, never added to lethal risk.

Ranged attacks use the C Borg's breath/shot estimates as a floor
(`borg-danger.c:619-760`); uncommon effects use a conservative generic and
are marked as estimates, because `Knowledge` does not carry the spell
effect dice or breath divisors. LOS is a conservative Bresenham over the
remembered level. Summoners/breeders use the C Borg's 1.5/1.8 growth
factors. Remembered-unseen monsters scale by `Track::confidence`.

One risk number is `expected + risk_aversion_pct * (worst - expected)`.
Paths, rest, retreat, contact and combat pass the same tolerance;
`threat_horizon` sets the window. Both are new parameter-table entries
(50%, 1 turn) with evidence, not tuned values. Every trace line now has the
six numbers (`threat`), which #124 reads. Fear and paralysis set
`can_attack`/`can_act`; the engine refuses the attack, so the strategy
cannot choose it.

**Fixtures.** Seven active now: bbc D0008, D0012, D0013, D0019 (one
apprentice does not cause whole-level flight), the breeder pack, fear
(refused attack), and rust D1330 (the cat is seen and the stair is taken).
Twelve gaps remain, all #120/#122: stairs under contact, retreat at 1 HP,
the 2-HP mould, quaff/eat/read/wield. `bbc-d0006` pins Walk(7). The
fixture catalogue records the promotions.

**New baseline (play change).** Smoke/smoke-exact/full references are
regenerated and match 3/3, 3/3 and 45/45; the exact-boundary check is 3/3.
The smoke suite has one 20,000-turn survivor and two deaths; the full suite
is 45/45 deaths, deepest depth 3, longest 89,440 game turns, with nine
starvation deaths. 49 natural deaths in the runs, 0 unexported; 47 new
dossiers D1472-D1518. **No survival claim**: starvation is #122, and the
tactical layer that could use the new numbers is #120. The calibration
report (#124) is where the estimates are checked against damage taken.

**Next:** #120 (T4.3), tactical alternatives on one scale. It must promote
the twelve gaps: cure/attack/stairs under contact, retreat versus the free
hit. #122 then supplies the actions. The trace has the numbers; no
threshold was tuned silently.

## 2026-11-10: danger race flags, the #142 fix

The T4.1 fixtures' first finding is fixed. `danger.rs` named `MULTIPLY`,
`NEVER_BLOW` and `NEVER_MOVE` one higher than the engine enum (`RF_NONE` is 0
and the first valid flag is 1), so a grey mould's `NEVER_MOVE` was read as
`NEVER_BLOW`: it scored zero danger and the Borg rested beside it. Breeders
lost the multiplier and `RAND_25` was misread as `NEVER_MOVE`.

- Constants are now 17, 20 and 21, with the counting rule in the comment.
- `race_flag_numbers_match_the_engine_enum` checks the real bytes: a grey
  mould's byte 2 bit 4 is `NEVER_MOVE` and not `NEVER_BLOW`; a silver mouse
  is `MULTIPLY | RAND_50`; an adjacent mould scores above zero.
- Fixture promotion: `rs-d0006-mold-at-two-hp` no longer rests (Walk is the
  recorded retreat) and moved from the #142 gap to #120; `rust-d1330-rest-
  with-pursuer` no longer rests either, and its remaining gap is the
  staircase under a seen threat (#120). `bbc-d0006`'s exploration path
  changed from Walk(7) to Walk(9); the fixture pins the new direction.
- This is a play change: smoke, smoke-exact and full references are
  regenerated from build `3a74b98`, and the deaths at the new build
  identity are re-recorded. The runs produced 51 natural deaths, 0
  unexported, and 48 new dossiers D1424-D1471 (three repeats).
- **The full suite is 45/45 deaths, deepest depth 2, longest 89,240 game
  turns** (the longest run is 8,925 player turns). The previous baseline
  reached depth 3. This is not evidence that the fix is worse: the old
  model was resting beside moulds it could not see, and the strategy still
  has no cure/escape action. It is a play change to be measured, not a
  survival claim, and T4.2/T4.3 must judge it.

**Checks.** `cargo test -p borg-reborn-rs` (including the fixtures),
smoke and smoke-exact runs, `harness/diff` 3/3 and 3/3, the exact-boundary
check 3/3, the full suite, and 45/45 reference diffs. The references are a
new baseline, not an oracle; no survival or population claim.

**Next:** #119 (T4.2), one threat assessment. The fixtures it must satisfy
are `fear-blocks-the-attack` (#119), `bbc-d0006-deep-descent-readiness`
(#119), and the #120 group (stairs under contact, retreat at 1 HP, the
2-HP mould). #142 is closed by this task.

## 2026-11-10: T4.1 death-derived decision fixtures (#118)

Eighteen constructed `Observation` fixtures now replay deterministically in
`borg-reborn-rs/tests/decision_fixtures.rs`; the catalogue and the M3
triage are in [docs/FIXTURES.md](FIXTURES.md), the schema in
`tests/fixtures/README.md`, and the engine data they read is the trimmed
`tests/fixtures/knowledge.json` (a `BORG_KNOWLEDGE_LOG` dump at
`e473720`).

- **Four are active**: bbc D0008 (attack the adjacent 2-HP mould), bbc
  D0012 (the owner branch's one-blow kill), bbc D0013 (no level flight
  over one kobold), and `mmx-war-1` (one action per think in a breeder
  pack). They must avoid the forbidden action and meet the alternative.
- **Fourteen are gaps**, not skips: each pins the action the Borg takes
  today and names the open M4 owner. When the action or decision changes,
  the test fails and the fixture must be promoted. Owners: #119 (threat
  horizon, fear), #120 (scored alternatives, stairs under contact), #122
  (quaff/eat/read/wield), #142 (flag numbering).
- **New finding, filed as #142**: `danger.rs` numbers `MULTIPLY`,
  `NEVER_BLOW` and `NEVER_MOVE` one higher than the engine enum. A grey
  mould's `NEVER_MOVE` is read as `NEVER_BLOW`, so it scores zero danger
  and the Borg rests beside it; breeders lose the multiplier. The evidence
  is in the issue; the fix needs a new reference baseline and belongs with
  #119, not this PR.
- **M3 triage** (D1326-D1420, 95 dossiers): 59 last actions were retreat
  walks, 21 rests, 12 attacks, 2 explore walks, 1 recover walk; 4
  starvation, 4 poison, 5 grey mould, 2 Fang. Four representative fixtures
  cover the families; the rest repeat them (28 causes, one independent
  death per encounter family).
- The fixtures are constructed, not recorded: the committed dossiers lack
  the original snapshots. Each file's `construction` says what was
  invented and `limits` says what it cannot claim. The owner-manual
  branches are diagnostics, not lives; their `verified` text quotes the
  turn range and end state.
- Doc state from the M3 review is corrected: PLAN says M4 is in progress
  (T4.1), and #136 no longer carries `in progress`.

**Checks.** `cargo fmt --all --check`, `cargo clippy --workspace
--all-targets -- -D warnings`, `cargo test --workspace` and the script
tests. No play code changed, so no engine build, smoke run or reference
regeneration was needed; the fixtures are pure crate tests.

**Re-recording.** `harness/identity.sh` hashes the whole `borg-reborn-rs/`
tree, tests included, so adding the fixture test changed the build identity
even though play is byte-identical. CI's smoke deaths therefore arrived as
a new encounter family; they are re-recorded as D1421-D1423 from the CI
registry (seed 102/103/101, the same deaths as D1373-D1375). This is the
existing process cost of a test-only change. Whether test files belong in
the build-input hash is a repo-level question for the owner; it does not
block M4.

**Next:** #119 (T4.2), one threat assessment. It must account for #142
first (the fixture `rs-d0006-mold-at-two-hp` pins the wrong decision), then
add status (fear), the group/breeder multiplier and the multi-step horizon
that `rust-d1330-rest-with-pursuer` pins. #118's fixtures are the check.

## 2026-11-10: M4 source refresh

The owner directed M4 to start while #135 remains open for re-review.
Pins moved for all five active sources; copied references and the death
index are refreshed, with the new reading in
[docs/research/KNOWLEDGE.md](research/KNOWLEDGE.md).

- better-borg-c `a4cc2d4`: no new death dossiers. M11 #471 has 18 carry
  units but no cohort/per-mode evidence and no v11 tag yet. Read the five
  `borg-danger.c` corrections (partial-hit armour/chance, CON drain,
  Crush's ranged value, rubble) and Shoot'N'Scoot's index fix.
- borg-runner `1440f70`: edition 4, including the fear/shrieker death,
  Fang AC36 versus Magic Missile, sparse sleeping monster placement,
  the whole-level M map and `mmx-war-1`'s batched-key breeder death.
  These become M4 fixture inputs.
- borg-zoo `415183c`: use explicit `normal` for controls; `default`
  changed meaning at the mode cutoff. reserve410's small pilot is mixed,
  not support for a buying claim.
- borg-rs `399c907`: the family now has an explicit buying-strategy seam;
  buy-to-identify remains open. Angband's 12 new commits are noted for
  M4's rest/executor work; our engine remains at `e473720` as specified.

**Next:** #118 (T4.1), death-derived decision fixtures. Build the
observation/scenario fixtures before changing the threat model, including
the PLAN seed cases, the new fear/shrieker and Fang cases, and reproducible
M3 deaths. No survival or tuning claim comes from the M11 package.

## 2026-11-10: M3 review blocker, bounded rerolls (#136)

PR #135's review rejected the first G0 baseline: T3.7's up/down reroll
could generate levels forever at one game turn, reset both no-time and
goal budgets, and select fresh levels (stair-scumming in substance).
The dedicated fix replaces those references and rewrites
[docs/G0.md](G0.md).

- Level generations are unique; travel goals die on generation change,
  so fresh levels Explore. Only an exhausted frontier or cooling Explore
  may reroll, once per generation and at most two level changes at one
  game turn. Stairs no longer reset the ordinary no-time budget.
- Unsafe staircases retreat instead of holding; remembered danger uses
  `Track::confidence` and decays out of view.
- `tools/check-no-time.py` checks same-turn actions and level caps in CI;
  final G0 runs use a real 10-second supervisor stall timeout.
- Review findings deferred to M4: #137 Engage cooldown, #138 real
  decision candidates, #139 small strategy/executor cleanups. Closed
  M3 task labels are cleaned; PLAN points at the review blocker.

**Corrected evidence:** 54/54 processes succeed with the 10-second
supervisor; smoke is deterministic and exact passes; 54 natural deaths,
0 unexported, with 48 valid new dossiers D1373–D1420; 51 traces have no
same-turn budget violation; perception is 0 differences over 50,047
thinks; tests are 81 crate + 5 repository + 56 tools. Full-suite stair
actions fall from the rejected baseline's 2,040 to **83**, with no turn
above two.

**Not hidden:** the new full suite has 546 Holds and four starvation
deaths, all 45 runs die, deepest is 2. One unreachable-exit case owns
400 Holds and belongs to M4/#123. The old “6 Holds / 0 starvation”
claim is withdrawn. G0 passes on smoke only; there is no full-suite
no-stall or survival claim.

**Next:** return PR #135 to the owner/reviewer for re-review. Do not
start #118 until the owner merges the milestone review.

## 2026-11-10: T3.8, M4–M7 are task issues (#24)

Created the dependency-ordered future work, with PLAN.md's death fixtures
and owner-manual alternatives attached to the tasks that must satisfy
them. No play code or reference change.

- **M4, danger and survival:** #118 death-derived fixtures; #119 one
  threat assessment; #120 tactical alternatives on one scale; #121 escape
  ladder and landing risk; #122 consumables/status/reserves; #123
  recovery and oscillation; #124 calibration report and M4 gate.
- **M5, readiness and progression:** #125 depth readiness; #126 equipment
  and pack valuation; #127 class item use/spells/ranged; #128 G1 on
  `classes-1800` (owner-run zoo, D and margin fixed in advance).
- **M6, the family:** #129 evidence-led class profiles; #130 screening
  and racing; #131 G2 against better-borg-c on fresh seeds.
- **M7+:** #132 deep suites and G3; #133 long games and G4.

Every issue has the `task` label, its milestone and a **Needs** chain;
T4.7 opens the M4 review rather than allowing an automatic M5 start.

**M3 is complete.** Next and last in this session: open the M3 review PR
(`reviewed` ← `main`) with the G0 evidence and review prompt, then stop.
The reviewer is the owner's choice; do not start #118.

## 2026-11-10: T3.7, G0 (#23)

**G0 passes; the report is [docs/G0.md](G0.md).** The first attempt did
not pass honestly: seven long runs held in Explore/cooldown loops until
starvation, 53,497 Holds in the full suite. T3.7 fixes the causes:
frontier-underfoot is finished, empty Explore falls to Descend, the
up/down reroll actually Ascends on reaching the up staircase, and checked
level changes do not spend the refused-action budget. Regression tests
cover the exhausted level and its only exit.

Final committed-tree evidence (`6318cd2`, references in `84a30fd`):

- 54/54 smoke, smoke-exact and full processes succeed with no stall,
  timeout, signal or harness failure; two smoke suites are byte-identical
  and exact boundaries pass. Extra-load passes at turn 1408.
- 51 natural deaths in run outputs, 0 unexported; the 47 new dossiers
  D1326–D1372 all have valid trace tails and hashes.
- Full suite: **6 Holds and 0 violations** (was 53,497 and 2), deepest
  **3** (was 2), **0 starvation deaths** (was 7). All 45 still die, so
  this is a machinery pass, not a survival claim.
- Perception: 0 differences over 24,079 thinks. Tests: 76 crate + 5
  repository + 54 tools; fmt and clippy clean. Smoke's warrior survives
  to CL7 at the turn cap.

**Next:** T3.8 (#24) — break M4–M7 into task issues, then stop for the
M3 review PR. Do not start M4.

## 2026-11-10: T3.6, traces in death dossiers (#22)

**Already implemented by T2.4 (`651c822`), now verified as a task; no
play-code or reference change.** Every Rust death export keeps
`trace-tail.jsonl` (the last 200 thinks), a `decision_trace` record with
`lines` and `of`, its SHA-256 in `files_sha256`, and a report link.
Untraced C runs record `decision_trace: null`. The exporter's module
documentation now states this contract in the present tense.

Validation: all **54 tool tests** pass, including the 250-line synthetic
trace that retains exactly thinks 51–250. An audit of the **48 T3.5
dossiers D1278–D1325** parsed every retained think and checked every
line count and trace SHA-256; examples range from all 51 thinks of
D1278 to the last 200 of D1325's 8,876.

**Next:** T3.7 (#23) — the G0 smoke/stall/determinism/death-trace report
for the M3 review.

## 2026-11-10: T3.5, town reserves (#21)

The first strategy now has a **Town** mode, checked after recovery and
contact but before any live travel goal:

- **Reserves are explicit parameters**: 3 rations, 2 spare torches, 3
  Cure Light Wounds, 4 Phase Door and 1 Word of Recall (c15 §12.4 and
  the owner's verified D0015 loadout). Only identified consumables
  count. Food/light, cures, escapes and the way home are bought in that
  order, one ware per observation so sold-out rows cannot leave stale
  letters. A full pack still permits a purchase into an existing stack;
  unavailable stock, unaffordable wares or no room for a new stack let
  the Borg leave town instead of looping.
- **Identification sales** follow the reserves: one unknown scroll or
  potion at a time goes to the Alchemist, which learns its flavour and
  runes before the hand-over (`store.c:1947-1956`). Selling earns 0 with
  `birth_no_selling`; this is identification, not income. There are no
  discretionary purchases yet.
- **Store mechanics fixed along the way:** stacked buys/sales check
  carried item counts, not only pack slots; free store actions no longer
  consume the refused-action budget; and paths can leave a shop entrance
  after shopping as well as arrive at it.

**Validation:** 74 crate tests and 5 repository tests, clippy and fmt;
smoke 3/3, smoke-exact 3/3, full 45/45 and extra-load from committed
`155c25a`; **0 perception differences over 72,623 thinks**. Seed 101
buys the intended kit, reaches **depth 2 / CL3** and dies to a soldier at
3696 with zero violations; its keys are byte-identical on rerun. All 45
full-suite runs shop. References regenerated; deaths D1278 onward.

**Honest state:** every full-suite run still dies and deepest remains 2;
seven starve (buying food is not eating it), and two unrelated refused
attacks are left for T3.7's G0 report. No final-suite run returned to
town carrying an unknown consumable, so the identification sale's live
key sequence was not exercised; its strategy, keys and stacked effect
are unit-tested.

**Next:** T3.6 (#22) — put the decision trace in every death dossier.

## 2026-11-10: T3.4, the playing behaviours (#20)

On top of T3.3's machinery, the first behaviours (references
regenerated; every game differs from T3.3's):

- **Engage answers contact**: a visible monster in reach makes the mode
  Engage. The exchange test (c15 FIGHT-01 through the danger sum): the
  monster's danger at our grid × the turns our melee per turn
  (`Character::melee_per_turn`) needs to kill it — fought when that
  costs less than every hit point (no margin constant; M4's tactical
  layer replaces the whole comparison), the *cheapest* adjacent fight
  chosen (bbc D0012's 2-HP mold). A losing exchange steps away
  (never onto a monster); a corner fights the least dangerous one
  anyway (bbc D0018: walking from a faster pursuer is death).
- **Pickup**: the object underfoot is taken (`Get`) whenever the pack
  has room and nothing is in contact — never under fire, and never
  before the threat is cleared (bbc D0019's dropped Phase Door, both
  orders of the same mistake).
- **Stairs under fire are not taken**: `Descend` requires zero danger at
  the grid; what presses is fought or escaped first.
- Rest unchanged (the strict zero-danger gate); the recovery invariant
  now also requires safety, so contact outranks resting.
- **The world follows the player's mutable memory.** `map_info` reports
  `player->cave`, and feature 0 means the player has forgotten the grid:
  nightfall in town wipes boring floors (`cave_illuminate`,
  cave-map.c:581-585; day length 10000, hence turn 50000). `World::see`
  now erases those grids instead of keeping a private map. CI caught the
  old behaviour diverging by 2.5 million grids in one town night; the
  full test suite now has **0 perception differences over 39,002
  thinks**.
- **Town is left, not mapped.** At depth 0 Descend outranks Explore
  (c15 TOWN-01): one earlier run starved after exploring town for
  50,000 turns. The final full suite's deepest run reaches depth 2.

**The honest M3 state:** the Borg fights, but remains fragile. All 45
full-suite runs still die at CL1–2; the deepest reaches depth 2, four
starve after long dungeon stays, and fruit bats, poison and ordinary
early monsters remain deadly. Final seed 101 dies of poison at turn
2490; its 252 thinks split explore 134 / engage 116 / descend 2, with
zero effect violations and byte-identical keys on rerun. The smoke
priest survives to the 20,000-turn cap at depth 1. T3.7's G0 report
should examine both the long starvation runs and any remaining Holds.

**Next:** T3.5 (#21) — town: sell, buy reserves (the owner's verified
D0015 loadout in the ledger is the target shape; the store actions and
the free identification-by-giveaway are already in the executor).

## 2026-11-10: T3.3, the first strategy (#19)

`strategy.rs` (L5) replaces the minimal Borg's placeholder policy — the
**first play change since M1**; the references are regenerated in this
PR and every difference is the strategy's.

- **Modes, goals, budgets, cooldowns** (GOALS principle 6):
  `Recover`/`Descend`/`Explore`, each goal with a budget in game turns
  and a progress measure (hit points for recover, depth for descend, the
  level's remembered grids for explore); a goal that stalls past its
  budget, or whose action the executor reports refused, fails and its
  mode cools down (`goal_cooldown_turns`, 100). Hysteresis is by rule,
  not by number: a live goal keeps its mode until it completes, becomes
  unsafe or exhausts its budget; the recovery invariant outranks all.
- **The M3 danger placeholder** (`danger.rs`): the C Borg's danger sum
  lifted with citations — `borg_danger`'s capped sum
  (`borg-danger.c:2824-2865`), `borg_danger_one_kill`'s energy model
  (`:2361-2395`), proximity model (`:2516-2593`) and wakefulness and
  condition scaling, and `borg_danger_physical`'s per-blow full-damage
  terms for the common early blows. Declared out, for M4: the spell
  term (a caster at range scores melee only), the friends multiplier,
  the fear maps, PFE and buffs. It now feeds both the flow's threat
  costs (replacing T3.2's hand-rolled shape) and the strategy's rest
  gate.
- **Rest safety** is the strict reading of c15 REST-01: rest only at
  zero danger — the C Borg's `avoidance` form rests beside a small
  monster and dies on the rest's own turns (bbc D0017), and ours did
  too in the first try (a rest-loop against a fruit bat).
- **Recovery under threat** steps to the least dangerous neighbour
  (never onto a monster's grid) — one step of L4 until M4's tactical
  layer.
- Two new parameters with declared-placeholder defaults:
  `explore_budget_turns` (2000) and `goal_cooldown_turns` (100).
- The trace's mode is the strategy's own ("explore", "descend",
  "recover"); the executor's effect failures reach the strategy as goal
  failures.

**The honest M3 state:** the Borg cannot fight (melee is T3.4), so it
retreats from everything — and a faster pursuer kills a retreating
character (bbc D0018's shape). The smoke runs die early: seed 101 to a
fruit bat at turn 2815, the mage to a grey mold at 2221, and the town
runs to dogs and cats (D1081–D1086 and the full suite's). No stall, no
crash, no harness failure, no effect violation; modes switch correctly
(explore 246 / descend 16 / recover 21 in the seed-101 trace); the same
seed twice gives byte-identical keys.

**Next:** T3.4 (#20) — melee (the exchange judged with the danger sum),
pickup, stairs and rest as real behaviours on top of the machinery.
Then T3.5 (town), T3.6 (traces in dossiers), T3.7 (G0).

## 2026-11-10: borg-runner's ACCUMULATED_KNOWLEDGE, copied (S15)

A pass over borg-runner and better-borg-c, asked for by the owner, found
one document we did not have: **`ACCUMULATED_KNOWLEDGE.md`** at
borg-runner's root — the export *written for this repository* (its header
names borg-reborn-rs as the consumer), only what no manual or data file
says: measurements, corrections of the manuals and of itself, numbers,
and the open questions. It is rewritten from time to time (edition 3
landed during this pass and **corrected edition 2's most load-bearing
claim: a level has three or four down staircases, not one** — the "one"
was the labyrinth generator; the classic/modified generators ask for
`rand_range(3, 4)`, `gen-cave.c:1291`).

This PR adds the copy rule (and a two-line amendment to
`refresh-sources.py` so a root-file rule works, with a test), moves the
borg-runner pin to `883c474` (four new commits today: a twelfth run
`minimaxflash-3`, a Warrior that died standing still for a Word of
Recall; a new INSTRUCTIONS section on constitution as the pace of
descent; edition 3; playbook folds), and records S15's new material in
SOURCES.md. The reading itself is consolidated in
`docs/research/KNOWLEDGE.md` (next PR), which also sets up the
read-marker process the owner asked for.

## 2026-11-10: the knowledge ledger (read markers for the moving sources)

`docs/research/KNOWLEDGE.md` (this PR) consolidates the pass:

- **The loop**: the refresh report's lines (new deaths, changed copies,
  issues updated since `read_on`, new tags, changed cited functions) are
  mapped to what to read; the ledger records the state. FACT lines are
  checked and cited before use (RULES 16); nothing is a claim until the
  zoo measures it (RULES 14).
- **Reading state**: borg-runner read whole at pin `883c474` (all
  manuals, every notebook and report, ACCUMULATED_KNOWLEDGE edition 3 and
  its diff); better-borg-c dossiers D0001–D0019 and the analysis issues
  (#400–#412, #420 on this pass); the zoo's catalogue.
- **Distillations by task**: T3.3 (depth is not the objective, 3–4 down
  staircases, explore-or-reroll, constitution as pace), T3.4 (the
  hit-chance formula, thrown oil, rest policy), T3.5 (the owner's
  reference loadout from D0015, the store identifying for free,
  restocking), M4 (the five analysis issues as the failure catalogue,
  breath/summon/speed facts, unknown pools by depth, escapes-bought-
  never-used), M5 (kit gates, power pricing, the mana penalty, class
  findings).
- **What borg-runner can additionally provide**: `decisions.jsonl` at the
  pin (scenario material), the replay site, and what *not* to take
  (`agents/` is model self-notes, not game knowledge).

AGENTS.md's entry reading order gains the ledger as step 7, so every
session knows the reading state before choosing work.

## 2026-11-10: T3.2, paths and exploration (#18)

`borg-reborn-rs/src/path.rs` is L4's paths, substrate for T3.3's strategy
(nothing calls it in play yet; T3.3 wires it into decisions):

- **Distance maps, not gate-kept floods.** One Dijkstra flood per goal
  over the remembered level, from the goal grids outward (the C Borg's
  shape, `borg-flow.c:borg_flow_spread`), but threat is a **cost** rather
  than the C Borg's fear refusal (borg-flow.c:442-474): a dangerous grid
  is paid for, and a route through it stays available when it is the only
  one. Goals: `Stairs{up}` (the known `<`/`>` grids), `Frontier` (known,
  enterable grids with an unknown neighbour — the C Borg enqueues the
  unknown itself, borg-flow-dark.c:46-58; a closed door by the unknown is
  frontier too, borg-flow-dark.c:61), and `Grids` (explicit targets).
- **Costs.** Floor 1; a closed door 2 (walked open, then entered);
  a web or a known trap +2; unknown grids are never transit (the C Borg's
  `avoid`); shops are goals but not roads (PASSABLE terrain — `,` opens
  the store — yet the C Borg's spread skips a shop that is not its
  target, borg-flow.c:371-374). Threat placeholder until T3.3 lifts the
  C Borg's danger sum: an awake tracked monster costs its grid 12 and its
  neighbours 4, weighted by its actions per player action
  (`Knowledge::turns_per_action`), by `Track::confidence` and by a
  quarter when asleep.
- **Stale by construction.** `Paths` records the world snapshot
  (`turn`, `thinks`) and refuses any other; it is built per think and
  dies with it, so the stale-flow reuse of the mold deaths (bbc D0008,
  rs D0006) cannot happen. Within a think each goal's map is computed
  once.
- **Determinism:** DIRS order breaks step ties; no unordered containers.

**Evidence:** 9 unit tests (maps built from ASCII worlds: corridors,
doors, webs, walls, the unknown, shops, threat paid as a cost, faster and
sleeping monsters, the stale-world refusal); and, on a real game, the
recorded run's **302 thinks replays with every reachable goal stepping
strictly downhill onto a passable grid** (`recorded_walks_are_strictly_
downhill`, the same `BORG_*_LOG_CHECK` wiring CI already records). Play
is unchanged — no live-loop change at all: smoke 3/3, smoke-exact 3/3
and full 45/45 identical to the references. The new source hash
re-records the suites' deaths: D1035–D1080 (46 cases).

**Not checked:** the maps' *quality* (do these costs make the Borg choose
well?) is T3.3's, on play; the threat shape is a placeholder with no
claim (RULES 14). `remaining()` at the player subtracts the stood-on
grid's own cost; if a future caller finds that surprising, rename then.

**Next:** T3.3 (#19), the first strategy: modes, goals, budgets, with the
C Borg's danger sum lifted as the M3 placeholder and the executor (#17)
and paths (#18) beneath it.

## 2026-11-10: T3.1, the executor and prompts (#17)

M3 started (PLAN's row moved to *in progress*; the source refresh PR #108
landed first, see below). `borg-reborn-rs/src/executor.rs` is L6:

- **Actions to keys.** The `Action` vocabulary M3's strategy needs
  (T3.4/T3.5): `Rest(n)`, `Descend`, `Ascend`, `Walk(d)`, `Attack(d)`
  (the same key as a walk, named for its intent), `Get`, `Hold`,
  `Alter(d)`, `Buy{ware}`, `Sell{item, qty}`. Every action is one
  complete key sequence. Item letters come from the engine's own lists,
  checked at the pin: the pack's `all_letters_nohjkl`
  (`ui-menu.c:44`, labels at `ui-object.c:292`), the store's selections
  with the original keyset (`ui-store.c:805`). The store sequences are
  the C Borg's, lifted with citations
  (`borg-store-buy.c:borg_think_shop_buy:1206-1244`,
  `borg-store-sell.c:borg_think_shop_sell:1101-1146`): the whole store
  visit runs inside one `,` (`do_cmd_hold`, cmd-cave.c), costs no game
  time, and ends with the C Borg's `ESCAPE`s — one purchase per visit,
  which also sidesteps the store list's renumbering on a sale-out
  (borg-runner's minimaxflash-2#91).
- **The prompt catcher** runs in `next_key` before the key queue, as the
  C Borg's does (`borg.c:304-308`): `borg_react_prompted`'s table and
  the `-more-` test (`borg-messages-react.c:95-324`,
  `borg-messages.c:183-294`), answers unchanged, `"Die?"` excluded
  (GOALS D5). Two declared departures, both in the module docs and
  `docs/ENGINE.md`: we do not press a space to clear a normal message
  (the C Borg does, `borg-messages.c:288-290`; it would change every
  recorded key stream), and `auto_more` stays on (the catcher answers a
  `-more-` if one ever appears; the cost — a message batch that scrolls
  between thinks is not parsed from the screen — is declared, and T3.3
  revisits it if the strategy needs the full message stream). A third,
  cosmetic: the C Borg compares sub-screen lines with a trailing space;
  our screen read trims, so the trimmed form is used.
- **Expected effects.** Each action carries one (`Step`, `Time`,
  `Depth`, `PackGrows`, `PackShrinks`), checked by the next think
  against the new observation; a mismatch is a trace violation naming
  the action, and prompts answered between thinks are a new `prompts`
  field of each trace line (additive; old lines parse). The engine reads
  the catcher needs joined the D3 list: `inkey_flag`, the message line's
  first-cell attribute and the terminal cursor (`docs/ENGINE-READS.md`;
  the bindings were regenerated).

**Evidence:**
- Play is byte-identical to the references: smoke 3/3, smoke-exact 3/3,
  **full 45/45** (keys, messages, states). The catcher answered **0**
  prompts in the full suite's 72,096 thinks (the placeholder meets no
  prompts, as expected), and the effect check recorded exactly 5
  violations — refused steps the same-turn guard already caught, now
  named in the trace.
- Unit tests cover every key sequence, the whole prompt table (and its
  negatives: messages, blanks, `"Type"`, `"Die?"`, queue-not-empty
  guards) and each effect's hold and fail. `cargo fmt`, `clippy
  -D warnings` and the tools' 54 tests pass.
- The suites' deaths are recorded at this source as D0989–D1034 (46
  cases: the full suite's 45 and the smoke mage; the smoke-exact mage
  dedupes into the smoke case). The plays equal the references', but the
  Rust source hash is new, so they are new encounter families (RULES 12).

**Found on the way:**
- `birth_no_selling` defaults **true** in 4.2.6 (`list-options.h`):
  stores pay nothing, the sale prompt says "Give", and one `Enter` still
  confirms the hand-over. Sell's effect is therefore pack-based, not
  gold-based.
- A store visit charges no energy (`do_cmd_hold`: "turn will be taken
  exiting the shop" sets `energy_use = 0`), so no store action can
  expect game time to pass.

**Not exercised:** no suite run enters a store, picks anything up or
meets a y/n prompt — the placeholder never does — so `Buy`, `Sell`,
`Get` and the catcher's answers are unit-tested only until T3.4/T3.5
play them for real. The perception checks (CI's test build) were not
run locally; CI runs them on this PR.

**Next:** T3.2 (#18), paths and exploration. T3.3 (#19) replaces the
placeholder and will be the first play change since M1; new references
come with it.

## 2026-11-10: M3 source refresh

The M3 `--update` (RULES 21) moved five pins:
- borg-rs `286215c` → `8a7579d`: docs only (the milestone 9 adoption,
  its 0098 NULL guard);
- better-borg-c `eb9a78e` → `06f5789`: new deaths D0017–D0019,
  REDESIGN-LESSONS updated to cover them;
- borg-zoo `e56d5fa` → `a092d1e`;
- borg-agent `77e48c9` → `57a480c`;
- borg-runner `e829a01` → `956171b`: the knowledge page grew by 155
  lines, and three minimaxflash runs were added (notebooks and write-ups).

angband and neo-angband-tracker are at their pins.

**New deaths** (better-borg-c, read first; all original-C mechanisms,
input for M3's strategy and M4's threat model):
- **D0017**: a CL25 Ranger on DL21 escaped Mirkwood spiders (speed +10,
  poison) by staff at 7 HP, then rested for Recall while poisoned, with
  9 CCW and the SP for Herbal Curing carried. While `goal.fleeing` is
  set, caution never calls healing. A mode chosen for the fight blocked
  recovery from an ongoing status (REDESIGN-LESSONS L2's new
  requirement: re-evaluate a mode when the situation changes).
- **D0018**: a CL2 Druid, gold spent on armour and jewellery, one Phase
  Door bought and used at low danger, no CLW, Recall at home, walked
  away from a speed +8 wild cat for eight steps while it clawed the
  character down. The walking retreat judges each step by the pursuer's
  current square; a faster pursuer that keeps attacking is not modelled.
  The owner's verified retry died too.
- **D0019**: a CL3 Necromancer, no cure bought, Recall at home, fled a
  whole level over one apprentice (the CL ≤ 8 name list), spent its last
  Phase Door seven squares from the stairs, then meleed cutpurses for
  ~190 turns next to a dropped Phase Door it never picked up: attack
  runs before pickup. The owner's verified takeover survived.

D0018 and D0019 carry `owner-manual-branch.json`; D0019's branch
survived, so it joins the scenario-test list (PLAN.md) with D0010,
D0012–D0014.

**borg-runner's page** (`reference/borg-runner/manuals/INSTRUCTIONS.md`),
the M3 reading, gained: key-handling facts for the executor (never a
real newline — the frontend drops a trailing one; leave look/targeting
mode with ESC; never two commands in one ask after a movement command;
the store list renumbers when an item sells out, so buy the last row
first; the rest prompt is pre-filled and one backspace clears it); a
"depth is not the objective" section (town-and-depth-1 shuttle, town
restock daily, Word of Recall heals on arrival); where you stand (the
game begins on the town's down staircase; the stair under you is never
drawn, only the status line names it; one down staircase per level,
uniformly placed); and the experience arithmetic (a kill pays
`mexp × mlev / clev`; the early ladder; first casts and first uses
pay). FACT lines are checked against `engine/angband/src/` and cited
before use (RULES 16).

**New better-borg-c tags** `4.2.6-impact-v9` and `v9.1` (v8 was already
known): the zoo's catalogue at the new pin already lists the specimens
`4.2.6-bbc-m8` and `4.2.6-bbc-m9`, and a development suite
`classes-1800-200k` (200k turns). Nothing for this repository to do:
comparisons are the zoo's (SPEC §6).

**Next:** T3.1 (#17), the executor and prompts. PLAN's M3 row goes to
"in progress" in that PR.

## 2026-11-10: M2 reviewed and tagged

Kimi K3.1 Pro's fourth pass on #99, at `7cec235`: **M2 done**, with no
findings left. The owner had #99 merged (`reviewed` at `4ea0bb0`); `m2`
is tagged there, and the GitHub milestone is closed. The four passes'
findings were all fixed in #100–#104, so none carry over to M3. #103
(borg-runner as source S15, below) was held while the review ran and was
merged after it.

**Next session: start M3** (GitHub milestone 3, tasks #17–#24; SPEC §10
keeps M3 as PLAN.md describes it).
1. **Refresh the sources first**, in its own PR (RULES 21):
   `python3 scripts/refresh-sources.py`, then `--update`. On 2026-11-10,
   borg-runner was already 13 commits past its pin (`e829a01`), and its
   knowledge page is what M3's strategy reads.
2. **Then T3.1, the executor and prompts (#17):** the lowest-numbered open
   task, its Needs (#13, #15) closed. The M3 row of PLAN.md goes to
   "in progress" when it starts.
3. **Game knowledge for M3:** S14 (`reference/c-manual/`, chapter 15 for
   an automatic player) and S15 (`reference/borg-runner/manuals/
   INSTRUCTIONS.md`). Lines from either are checked against
   `engine/angband/src/` and cited (RULES 16), and none is a claim until
   the zoo measures it (RULES 14).
4. **The perception check stays on:** CI runs smoke and full on the
   `test-hallucinate,test-perception` build, and `tools/check-perception.py`
   must report 0 differences. A new engine read needs a row in
   `docs/ENGINE-READS.md` (`test_reads.py` fails otherwise).

**Known limits, declared and not fixed:** no recorded game has an
overfull pack, mixed cursed and uncursed stacks of one kind, the
unseen-monster-1 hallucination case, or a shop or web on an unknown grid,
so those fixes are covered only by the per-think checks. Floor objects and
store stock have no second route to compare against. Few monsters are met
(3,533 monster-thinks in smoke and full) until M3's Borg fights.

## 2026-11-10: a new source, borg-runner (S15)

The owner added [borg-runner](https://github.com/diegoami/borg-runner) as a
source of game knowledge: the accumulated knowledge of agent players.
Language-model agents play upstream 4.2.6 (`e473720`, our engine) from the
screen and record every key; what they learn goes into one page,
`manuals/INSTRUCTIONS.md`.
- **Pin:** `borg-runner` at `e829a01`. At that commit LEARNED.md and
  MANUAL-GAPS.md had just been folded into the page.
- **Copies** (`reference/borg-runner/`):
  - `manuals/`: the page, volume 2 (depth 5–20, untested) and the jev
    manual;
  - `knowledge/`: each run's notebook;
  - the `.md` write-ups of `records/`.

  `refresh-sources.py` has a new `suffix` selection for the last one.
- **Recorded in:** SOURCES (S15, with what we take and what we leave),
  GOALS D11 (borg-agent and borg-runner, same terms), README, AGENTS,
  reference/README.
- **How to use it:** each FACT line is checked against `engine/angband/src/`
  and cited (RULES 16). An OBSERVED line rests on a few runs, some of which
  inherited each other's notebooks. Nothing is a claim until the zoo has
  measured it (RULES 14).
- **First use:** the strategies and profiles of M3 onward, alongside S14.

## 2026-11-10: M2's third review: not done; findings fixed

Kimi K3.1 Pro's third review of #99, at `fb069d8`: **not done**. R2 and R4
are resolved. One blocking finding and three should-fix ones were new, all
fixed in the PR after #102:
1. **Blocking:** perception read `pack_size` inventory entries, but the
   pack has `pack_size + 1` and may be overfull by one. It now reads them
   all.
2. **Slots and levels:** the equipment's slot count and types are checked
   against the race's static body, and the occupied slots against the
   engine's; `Item.level` is compared.
3. **Pairing:** each engine object is paired with the observed item at its
   place and position. Each pair is compared field by field from the object
   directly, and whole against `analyze()` of that very object: exact
   weight, flags, modifiers, elements, description, awareness, runes. The
   description, awareness and runes come from the same engine functions as
   perception, so that half checks pairing and completeness, not those
   functions.
4. **The stat floor:** `wearing` takes the bonus nearest the known one
   (race, class, the gear's known modifiers). That is exact unless
   something unseen changes the stat at the floor. The reviewer's
   counterexamples are tests.

**Evidence:**
- Smoke and full on the test build: 76,175 thinks, 0 differences.
- The reviewer's faults now fail: slot types off by one, 48,948
  differences; zeroed item levels, 20,395.
- The normal build plays the references' games (3/3, 3/3, 45/45). Its
  deaths are D0943–D0988.
- **Not exercised:** no suite run has an overfull pack or mixed cursed and
  uncursed stacks.

## 2026-11-10: M2's re-review: done with changes; R1–R4 fixed

Kimi K3.1 Pro re-reviewed #99 at `eef7a8c`: **done with changes**. Both
blocking findings and the first round are resolved. Four remained, fixed
in the PR after #101:
- **R1:** the gear check is now place-aware. The engine says where each
  object is (`equipped_item_slot`, `object_is_in_quiver`), and each item
  is compared on its identity and known combat values, and known flags
  against bounds (superseded by the third review's pairing, below).
- **R2:** the pack count is compared with a fresh `pack_slots_used` call.
- **R3:** at the stat floor of 3, `wearing` takes the lowest result of all
  the bonuses that fit, so no stat is invented.
- **R4:** monsters are compared as multisets, so duplicates count.

**Evidence:**
- Smoke and full on the test build: 76,175 thinks, 0 differences.
- The reviewer's faults now fail: weapon and launcher swapped, 8,158
  differences; every monster duplicated, 48.
- The normal build plays the references' games (3/3, 3/3, 45/45). Its
  deaths are D0897–D0942.

**Next:** the owner merges #99 (or asks for another pass). Then tag `m2`,
the remaining findings (none open) become M3 issues, and M3 starts.

## 2026-11-10: M2's review (#99): not done; findings fixed

Kimi K3.1 Pro reviewed M2 at `70cb9b5`: **not done**, with two blocking and
six should-fix findings. They are fixed in #100 (findings 7, 8, doc nits)
and the PR after it (findings 1–6):
- **Finding 1 (blocking):**
  - **Fix:** the shop number and the web flag are read only where the
    player knows a shop or a floor, as the C Borg guards them.
  - **Check:** the truth check fails on any other shop or web.
- **Finding 2 (blocking):**
  - **Unknown grids:** a grid the player knows on the panel but the world
    doesn't is a difference.
  - **Panel:** the observed panel is checked against the engine's own
    `panel_contains`.
- **Finding 3:** monsters are compared field by field, and the gear item
  by item against the engine's gear list.
- **Finding 4:** every run needs one truth line per Borg decision, and a
  lost truth log fails the test build.
- **Finding 5:** `term_win` and `random` are in the types table, and a
  field-level audit fails on fields of unlisted types.
- **Finding 6:**
  - **To-dam:** the weapon and the launcher are left out of the gear's
    to-hit and to-dam; melee adds the weapon's own.
  - **Stats:** `wearing` recomputes the stats in use (`modify_stat_value`).
- **Findings 7–8 (#100):** `--runs` matches deaths by (family, keys);
  `make-reference.sh` refuses before writing anything.

**Evidence:**
- Smoke and full on the test build: 76,175 thinks, 0 differences.
- Kimi's three faults, which passed before, now fail: 219,103, 223,230 and
  96 differences.
- The normal build still plays the references' games (3/3, 3/3, 45/45),
  so the references stay. Its deaths are D0851–D0896.

**Next:** a re-review of #99 is the owner's call (RULES 6). #99's prompt is
updated to the new head.

## 2026-11-10: T2.5, perception against the engine; M2 complete

The owner closed E's review without a re-review. borg-zoo#48 now asks to
move `4.2.6-reborn-rust` to `f0eeee2`; the zoo check there gives the same
smoke play hashes as M1, and its death is D0804 (#97).

**T2.5 (#16):**
- **The check:** a test feature, `test-perception`, compares every think
  with the engine through other routes (`engine.rs`, below `ENGINE TRUTH`):
  - the world's remembered map against `player->cave`;
  - the engine's visible monsters on the panel against the monsters seen;
  - the gear list against the observed equipment, quiver and pack;
  - our pack count against `pack_slots_used`.
  `tools/check-perception.py` fails on any difference. It also checks the
  status against the frontend's `state.log`.
- **Results:** smoke and full, 76,175 thinks, have 0 differences.
  - Declared, not failed: 220,350 grid-thinks the player knows but the
    world never saw (218,978 in full, 1,372 in smoke), and 96 visible
    monsters off the panel. The C Borg
    reads only the panel too (`borg-update.c:264`).
  - An injected fault gives 8,205 differences, so the check can fail.
- **One defect found:** while hallucinating, `map_info` puts `m_idx = 1`
  on an empty grid. When monster 1 stood there unseen, the Borg took it as
  seen; 11 of 45 full runs failed the hallucination assertion, and CI had
  run only smoke. Perception now also needs `monster_is_visible`
  (`map_info`'s own test, a B read). Our suites play the same games
  (references unchanged: 3/3, 3/3, 45/45), since none of their runs met
  that case.
- **D3:** reason T for the check's reads, used only below the marker
  (`test_reads.py`). The types read field by field are listed (E review,
  finding 10).
- **CI:** one test build with both features runs smoke and full; its play
  equals the references, and `check-perception.py` passes.
- **Deaths:** D0805–D0850.

**M2 is complete:** T2.1–T2.5. Next is the review PR (`main` → `reviewed`)
with its prompt; the owner chooses the reviewer (RULES 6). M3 doesn't start
before that.

## 2026-11-10: E's review is in; fixes in three PRs

Kimi's review of E (on #92) has ten findings. They are fixed in three PRs
into `main`:
1. **The frontend (#95, RULES 22):** `loaded` and `exact_turns` are in
   every summary (finding 5).
   - Play doesn't change: keys, messages and states equal the references
     (smoke 3/3, smoke-exact 3/3, full 45/45).
   - `harness/diff` compares the two fields when both sides have them, as
     it does `game_options`.
   - Its deaths are recorded at its source (D0712–D0757).
2. **The code (#96):** suites, perception while hallucinating, the death and
   reference tools.
   - References are regenerated: the full suite's `menu` fix changes
     exactly the 20 runs whose character changes.
   - Smoke, smoke-exact and the other 25 full runs are unchanged.
   - Its deaths are D0758–D0803.
3. **Documents, rules and CI (#94):**
   - findings 6, 7, 9 and 10's document nits;
   - RULES 12 narrowed by the owner: measured runs only, not watched games;
   - CI fetches upstream `e473720`, so the tree check no longer skips.

**Left:**
- ENGINE-READS' types table (`term`, `monster`, `player_class`, `store`):
  T2.5's D3 recheck (#16).
- borg-zoo#48: `4.2.6-reborn-rust`'s `ref` moves to the commit that merges
  #94, not to `e`, since #96 changes the full suite's play (the zoo's own
  cells pass `-race`/`-class` with `-menu-letters`; whether its play
  changed is for the zoo's check).
- Whether the fixes get a re-review is the owner's call (RULES 6).
- Then T2.5 (#16) and M3.

## 2026-11-09: E merged before its review; a review after the fact

The owner had the session merge #92 (`main` → `reviewed`). Tag `e`
(`0a92da0`) was pushed, milestone E closed and borg-zoo#48 filed (move
`4.2.6-reborn-rust` to `e`), all **before any review**. The owner then
asked for the review after the fact:
- the reviewer, the owner's choice, reviews `55d7476` (the tree of `e`) and
  posts on #92;
- findings are fixed in PRs into `main`;
- tags are not moved (RULES 5);
- borg-zoo#48's first request is on hold until the review is clean.

Other work (T2.5, #16) may go on meanwhile.

## 2026-11-09: TE.9 and milestone E complete; E's review

- **TE.9:** `engine/tests/test_independence.py` (CI, rust job) checks that
  no build, test, check or CI step uses borg-rs. That means no checkout,
  crate, staticlib or former `angband/` tree, and no clone or download;
  provenance comments are allowed. It also checks that the workspace holds
  only our crate. CI now proves every point of SPEC §10's exit:
  - the tree is upstream's, the patches apply, and the source list matches;
  - the frontend contract, the zoo contract and the D3 list hold;
  - the bindings regenerate unchanged;
  - runs are deterministic, match the references, and keep the boundary
    contract;
  - the death reconciliation passes, and a CI death reruns with the same
    keys;
  - nothing reads borg-rs.
- **Milestone E is complete** (#81–#91). The review PR `main` →
  `reviewed` is open. The owner chooses the reviewer (RULES 6); the prompt
  is in the review PR's description.
- **After the owner merges it:**
  - tag `e`;
  - ask borg-zoo (an issue, RULES 7) to move `4.2.6-reborn-rust`'s `ref`
    to `e^{}` and to decide on the two C specimens;
  - continue with the rest of M2 (T2.5, #16), then M3.

## 2026-11-09: S12 retired (pin PR, #78)

The owner retired S12, the draft strategy manual, in favour of S14, the
complete one. This pin PR (RULES 21) removes the `strategy-manual` pin, its
copy rule and `reference/strategy/`, and updates SOURCES, reference/README
and AGENTS.md. Its local clone, `vendor/neo-angband-tracker-workbench-c-borg`,
is no longer used.

## 2026-11-09: TE.8, removals, documents and rules (#78)

- **Removed:**
  - the C arms' references (`harness/reference/smoke-c`, `smoke-pristine`,
    `full-pristine`, `smoke-c-bbc-m7`);
  - `pins.json`'s `controls` and borg-rs's oracle role, `borg_dir` and
    `tag_of_interest`;
  - `docs/ENGINE-TREE.md` (replaced by `docs/ENGINE.md`);
  - `suite.sh`'s dead control-arm branch.
- **Rewritten for docs/SPEC.md:**
  - RULES 9 (the tree is upstream's bytes plus our patch series), RULES 16
    (copies are recorded in PROVENANCE and owned) and RULES 22 (comparisons
    are the zoo's; a change to the integration reproduces the references
    or explains every difference);
  - GOALS principle 4, D2, D4 and D8, plus a superseded note on the "same
    seam, same harness as borg-rs" principle;
  - ARCHITECTURE L0, *What comes from where*, and the settled open
    question;
  - SOURCES S5 (borg-rs is a reference), with a historical note on its
    section;
  - README (intro, status, SPEC first in the table);
  - AGENTS.md (intro, start commands, the `engine/` and `harness/` rows,
    the `borg-rs/` row gone);
  - `harness/README.md` (rewritten for our harness);
  - the refresh report's wording (a new better-borg-c tag is a specimen for
    the zoo to consider).
- History stays history: PLAN's M1 row, the handover's earlier entries,
  `docs/ZOO-CHANGE-REQUEST.md`, and the M1-era rows of SOURCES.
- **S12's retirement** is its own pin PR (RULES 21), next.

## 2026-11-09: TE.7, the zoo contract (#77)

- **`docs/ZOO-INTERFACE.md`:** the contract the zoo's borg-rs adapter
  (`zoo/borgrs.py` at our pin `e56d5fa`) calls. It covers the build
  (`harness/build.sh`, `build/rust/game/`, `BUILD_ID`'s `binary_sha256` and
  `rustc`, `harness/borg.txt`), the run (`-mborgh` and its flags,
  `TZ`/locale), and what the zoo reads (`summary.json`'s 10 fields,
  `keys.log`, `state.log`). One mismatch is on the zoo's side: its `STOP`
  map knows `clev`, while our frontend writes `clevel`. It's harmless,
  since the zoo never passes `-clev`.
- **`engine/tests/test_zoo_contract.py`:** runs the binary with exactly the
  zoo's command line and checks every field and log format, plus that
  `BUILD_ID` matches the binary and another `BORG_IMPL` fails loudly. CI's
  engine job runs it.
- **`harness/zoo-check.py`:** checks only the Rust arm. End to end with the
  zoo's adapter: 3/3 cells with no harness failure or config error, and
  **play hashes identical to M1's**. The seed-29 death is recorded as D0711.
- **Retired:** `tools/endpoint.py` and its test (borg-zoo computes every
  statistic). GOALS D7 is amended.
- **After E is merged:** ask borg-zoo (an issue, RULES 7) to move
  `4.2.6-reborn-rust`'s `ref` to E's tag and to decide on the two C
  specimens.

## 2026-11-09: TE.6, our death tools; the copied tooling and borg-rs's tree gone (#76)

- **Decision (owner):** the death tooling to keep imported borg-rs's whole
  recording toolkit (GUI, replay viewer, runtime: about 2,000 lines), so
  all copied tools are dropped and replaced by lean tools of ours.
- **New:**
  - `tools/rerun-death.py` reruns a dossier's game from its build and seed
    with the observation, world and knowledge logs and checkpoint saves,
    and checks that the keys are the recorded ones. D0664 reproduces, and
    CI reruns one of its own deaths.
  - `tools/death_registry.py` regenerates the registry index, taken from
    `borg_deaths.py` with the same output (byte for byte).
  - `watch-borg.sh` builds our integration with X11 and opens the game.
- **Removed:**
  - the copied tools and their tests (25 Python files, 2 C files, 2
    headers) and 7 root launchers;
  - borg-rs's `angband/` tree, the `borg-rs/` shim crate and
    `harness/measurement/`;
  - `pins.json`'s `engine_tree` and the re-sync report;
  - the repository test `engine_tree_matches_its_record`;
  - `run.sh`'s `frames` flag.

  No file of borg-rs's engine tree is left, and no build reads one.
- **Docs:** README's *Playing*, AGENTS.md, `tools/README.md`,
  `docs/PROVENANCE.md` and SPEC §8 and §11 are updated.
- **Deaths:** the Rust deaths are recorded again, as D0665–D0710
  (`Cargo.toml` is a build input and lost the shim member). Play still
  matches the references: full 45/45, smoke 3/3.

## 2026-11-09: TE.5, the harness and run contract on our build (#75)

- **`run.sh`:** the C Borgs' flags (`cheat`, `cheatlive`, `munchkin`,
  `lunal`, `selflunal`, borg.txt rewrites) and the dropped frontend options
  (`frames`, `rune-pattern`) are gone. `BORG_IMPL` defaults to `rust`, and
  the run's `user/borg.txt` is written as before (the zoo copies the file;
  its hash is in a death's family). The environment is fixed (`TZ=UTC`,
  locale).
- **Option set:** the frontend writes every game option in force into
  `summary.json` (`game_options`), and `docs/ENGINE.md` declares the Rust
  Borg's options against the C Borg's.
  - **Found:** 4.2.6 has no `easy_alter`. The Rust Borg had been setting it,
    and the engine ignored it silently. It's removed, and an unknown option
    now stops the Borg (`GAME_OPTIONS` in `borg.rs`). Play can't change,
    since the call never did anything.
- **Reproduction:** the Rust Borg on our build reproduces the borg-rs-tree
  outputs: full 45/45, smoke 3/3, smoke-exact 3/3, hallucination smoke 3/3.
  `harness/diff` compares `game_options` only when both sides have it.
- **References:** `harness/reference/smoke-rust`, `full-rust` and
  `smoke-exact-rust`, made by `harness/make-reference.sh` with a
  `REFERENCE` file (source, binary, profile, borg.txt hashes,
  environment). CI compares smoke and smoke-exact with them. The C arms'
  references go in TE.8.
- **Deaths:** the Rust deaths are recorded again at the new source, as
  D0619–D0664.

## 2026-11-09: TE.4, our build (#74)

- **The build:** `engine/CMakeLists.txt` builds the Rust Borg from the
  pristine tree, our patches, our frontend and the crate's own staticlib.
  `harness/build.sh` (`BORG_IMPL=rust`) uses it and is rewritten; the build
  takes about 10 s.
- **Reproduction:** the Rust Borg plays **byte-identical** keys, messages
  and states to its build on borg-rs's tree (smoke 3/3, full 45/45,
  smoke-exact 3/3, hallucination smoke 3/3). The boundary contract holds.
  So the patch series and our frontend are a complete harness.
- **Gone:** the C arms (`c`, `c-pristine`, `c-bbc-m7`) from the build and
  CI, and `harness/control-overlay.py`. `harness/identity.sh` knows only
  `rust`, and its inputs are `engine/`, the crate, `Cargo.toml` and
  `Cargo.lock`.
- **Kept until TE.6:** borg-rs's `angband/`, the `borg-rs/` shim crate and
  `harness/measurement/`. The copied death tooling still builds its own
  binary from them (#76 updated).
- **Tests:** `engine/tests/test_build.py` checks the source list against
  upstream's and the input list, and runs the binary through `run.sh` for
  the frontend contract. CI's engine job runs it.
- **Deaths:** the Rust deaths are recorded again at the new source, as
  D0573–D0618 (the build inputs changed).

## 2026-11-09: TE.3, generated bindings and the D3 list (#73)

- **Generated bindings:** `borg-reborn-rs/src/bindings.rs` is generated by
  bindgen 0.73.2 from the pristine headers (`engine/bindings/`), limited to
  the symbols the Borg uses. It has 66 compile-time layout assertions
  against the C structs. The c2rust `headers/` copied from borg-rs are
  gone.
- **`engine.rs`:** re-based on the bindings. Its hand-written declarations
  of engine symbols are gone, except `inkey_hack`, which no upstream header
  declares.
- **The D3 list:** `docs/ENGINE-READS.md` lists every symbol with its
  reason (screen, direct read, static data, declared extra, write, seam).
  `engine/tests/test_reads.py` checks it both ways, and that only
  `engine.rs` touches the bindings.
- **CI:** installs libclang and bindgen and runs `generate.sh --check`.
  Locally this needed `clang`, `libclang-dev` (owner, apt) and `cargo
  install bindgen-cli`.
- **Play:** unchanged (Rust smoke 3/3, full 45/45). The Rust deaths are
  recorded again at the new source, as D0527–D0572.

## 2026-11-09: TE.2, our headless frontend (#72)

- **The frontend:** `engine/frontend/main-borgh.c` and `.h` are copied
  from borg-rs's borgh and trimmed to the contract of SPEC §3.5 (1,247 →
  931 lines). Dropped:
  - `-states` (native observer);
  - `-frames` and its diagnostics;
  - fixture building (`-make-fixture`, `-near-morgoth`);
  - `-stop-rune-pattern`;
  - `-cheat-live` (D5).

  Kept: every contract flag, the stall and key limits, `-clev`,
  `-random-birth`, the save options and checkpoints (the death tooling may
  use them).
- **The test:** `engine/tests/test_frontend.py` checks the flags, the
  `summary.json` fields, the mode registration, and a compile against the
  patched tree.
- **Not yet:** no binary until TE.4. `harness/run.sh` flags that used the
  dropped options (`frames`, `cheatlive`, `rune-pattern`) are cleaned up
  in TE.5.

## 2026-11-09: TE.1, the upstream tree and our patch series (#71)

- **The tree:** `engine/angband/` is upstream `e473720`, byte for byte
  (tree `99a0549`, `engine/UPSTREAM`), extracted from upstream, not from
  borg-rs.
- **The patches:** `engine/patches/` holds 7 patches for our harness hooks,
  every changed line guarded by `BORG_HARNESS` or `USE_BORGH`, plus one
  declared split of a declaration in `mon-make.c`.
- **Docs and tests:** `docs/ENGINE.md` and `docs/PROVENANCE.md` are new.
  `engine/tests/test_engine.py` checks the tree, that the series applies,
  and that with the guards off the code is upstream's. It runs in CI.
- **What stays:** borg-rs's `angband/` stays until TE.4 switches the build;
  nothing builds from `engine/` yet.

## 2026-11-09: milestone E planned

PLAN.md follows docs/SPEC.md §10. **E. Our own integration** is in
progress: the GitHub milestone E, tasks #71 (TE.1) to #79 (TE.9), linked by
their Needs. M2 is paused (T2.1–T2.4 done; T2.5, #16, needs E). Order:
- TE.1 tree and patches, then TE.2 frontend and TE.3 bindings;
- TE.4 build, then TE.5 harness and the reproduction check;
- TE.6 death tooling, TE.7 zoo contract and TE.8 removals and documents;
- TE.9 CI and E's review (the owner chooses the reviewer).

## 2026-11-09: the spec is in effect; only our Borg

The owner merged #66 (`main` → `reviewed`, `37bcb97`), so `docs/SPEC.md`
is in effect. There is no tag: it is not the end of a milestone. The owner
then decided:
- **borg-zoo does every comparison.** This repository delivers only our
  Borg, through the zoo contract. The C arms (`c`, `c-pristine`,
  `c-bbc-m7`), their overlays and references, `tools/endpoint.py` and #56
  (`c-bbc-m8`, now borg-zoo#36) are dropped in milestone E. GOALS D7 and D8
  change accordingly.
- **§11:** keep the death tooling and drop the GUI, replay and takeover
  tools; keep the zoo's current interface, written down; the engine-change
  question goes with the controls.

The `borg_self_scum = TRUE` finding concerns the zoo's
`4.2.6-reborn-c-*` specimens, which were built from this repository. It
is reported on borg-zoo#30.

Next: plan milestone E as issues (PLAN.md re-planned), then build it.

## 2026-11-09: the owner chooses the reviewer (RULES 6)

The review of #66 was started by the session: MiniMax-M3, picked through
the quota service. The owner then ruled: "next time give me the prompt and
I will chose the reviewer". RULES 6 and AGENTS.md (*Models*) now say so.
For every review the session writes the prompt and the owner chooses and
runs the reviewer. A second pass on #66 (for #67 and #68) therefore waits for
the owner's choice.

## 2026-11-09: review of #66 (MiniMax-M3): spec findings

The spec findings S1–S10 and the missed risks are answered in
`docs/SPEC.md`:
- **S1:** the zoo contract is written down. It covers the build entry point
  and layout, `harness/borg.txt`, and the frontend's mode `borgh`, flags
  and `summary.json` fields. The harness may change everywhere else.
- **S2:** the frontend registration (`main.c`, `main.h`) joins the patch
  series, and the line counts are corrected.
- **S3:** our CMake carries a copy of upstream's source list, checked
  against it; upstream's file can't be used as it stands.
- **S4, C2:** the configuration (the C Borgs' `borg.txt` with every D5
  exploit off, the game option set) and the environment (`TZ`, locale)
  become part of the run contract.
- **S5:** E's exit requires the new integration to reproduce today's
  `c-pristine` and Rust smoke and full keys before any old reference is
  replaced.
- **S6:** `docs/ENGINE-READS.md` is normative; channel A gains the
  message line's attribute and cursor; the four extra reads are declared.
- **S7, S9:** the inventory and the document lists are completed.
- **S8:** `c-pristine` is a control, not an oracle.
- **S10 and the risks:** configuration hashes and what CI must prove are
  in E's exit; the death registry's two generations are described.
- The reviewer's answers to §11 are recorded as recommendations; the owner
  decides.

**A finding for the owner (S4):** `harness/borg.txt` has
`borg_self_scum = TRUE`, upstream's default too. So in every control run so
far, a C Borg could money-scum for a ware from character level 10
(`borg-store-buy.c:141-153`), which D5 excludes. Its effect on M1's
numbers (c-pristine 1/45 and c-bbc-m7 2/45 at the full-exact cap) has not
been measured. The configuration is corrected in milestone E.
## 2026-11-09: review of #66 (MiniMax-M3): code findings

MiniMax-M3 reviewed the spec and the work since `m1` on #66: the spec is
"yes, with changes", and the code is sound. Code findings:
- **C1, fixed here:** `Character::blows` now follows the engine for a
  weapon too heavy to wield (one blow, `calc_bonuses:2296-2318`, with
  `adj_str_hold` lifted and checked against the engine's copy), and adds
  bloodlust's `timed / 20`. No recorded run heavy-wields yet, so the rule
  is unit-tested only.
- **C5, fixed here:** `with_borg_rng` restores the game's RNG through a
  drop guard, on every path.
- **C2** (the Rust Borg's game options differ from the C Borg's) goes to
  the spec's run contract.
- **C3** (a key aborts the Borg in a shop, which the C Borg does not) is for
  M3, when the Rust Borg first enters shops.
- **C4** (flushing the trace each think) is kept: a dossier needs the trace
  on disk when the game ends, and one write costs microseconds against
  borgh's stall limit of seconds.

The Rust deaths are recorded again as D0480–D0525.

## 2026-11-09: the spec (docs/SPEC.md), and a stop

The owner decided to move completely away from borg-rs. The game stays
upstream C Angband. Our own integration (hooks, frontend, bindings, build,
harness) is built first, and borg-rs is only a reference to copy from.
Before anything more is implemented, `docs/SPEC.md` states that project:
- the parts it keeps, replaces or drops (§8);
- the documents and rules that change (§9);
- a re-plan with a new milestone E, our own integration, first (§10);
- four open questions (§11).

The owner accepted it and sent it to review with the work since `m1`. It
takes effect when the owner merges that review PR. Until then: **no
implementation**, and the M2 task list after T2.4 (T2.5, then M3+) is on
hold for the re-plan.

## 2026-11-09: owner decisions

Asked again and answered on 2026-11-09:
1. **c-bbc-m8** (#56) runs with `borg_munchkin_by_class = FALSE`, so every
   arm keeps D5. Its label must say so. GOALS D8 is amended.
2. **The `smoke-c-bbc-m7` reference** is accepted as a self-produced
   regression reference (harness/README.md).
3. **S12 is retired** in favour of S14. The pin, its copy rule and
   `reference/strategy/` go in their own pin PR (RULES 21).
4. **T2.5: the engine is the truth, not borg-rs** (option D). borg-rs's
   perception can't be run on its own: it lives inside its decision loop.
   Perception is checked against engine facts reached by other routes
   than the Observation's reads, and every engine read is listed (the D3
   audit). GOALS D8, ARCHITECTURE L1, AGENTS.md and #16 are amended.

## 2026-11-09: T2.4, decision trace and parameter table (#15)

- `params.rs`: the parameter table has name, unit, range, default and
  the evidence for the default. It holds the minimal Borg's three numbers:
  `max_tries_per_turn` 2, `rest_below_hp_pct` 70 (integer percent, so
  `hp * 100 < max * 70` is the old test exactly) and `rest_turns` 75
  (`Action::Rest(n)`, whose keys are still `R75` Enter).
- Profiles are `name = value` files in borg.txt's syntax, read at start
  (`BORG_PROFILE`; run.sh defaults to
  `harness/profiles/reborn-default.txt`). An unknown name, a malformed
  line or an out-of-range value stops the game.
- The profile's SHA-256 is in `config.txt` (run.sh), `params.json` and
  every trace line.
- `trace.rs` plus `decide_with`: the decision now carries its goal,
  budget, scored candidates and violations; the guard of principle 6 is
  recorded as one. The Borg writes `params.json` and `trace.jsonl` to
  the run directory (`BORG_RUN_DIR`, from run.sh, Rust arm only). The trace
  is written after the decision from what the decision made, so play is
  unchanged (full 45/45, smoke 3/3, hallucination smoke 3/3 identical).
- Dossiers keep `params.json` and `trace-tail.jsonl` (the last 200
  thinks), and `record.json` gains `identity.profile_sha256` and
  `decision_trace`. CI checks the trace and parameters in run outputs and
  in Rust dossiers.
- The Rust deaths are recorded again as D0434–D0479, now with traces. The
  C arms' outputs and dossiers are unchanged.

## 2026-11-09: T2.3, knowledge tables (#14)

`borg-reborn-rs/src/knowledge.rs` holds the L3 `Knowledge`. The Borg reads
it once from the engine's parsed data at its first think
(`engine::knowledge()`):
- monster races (spoiler data, GOALS D3: level, speed, HP, AC, sleep,
  frequencies, race and spell flags, and blows with method, effect, power
  and dice);
- object kinds, egos and artifacts;
- classes with their blows parameters and spell books;
- player races;
- what each store always and normally stocks;
- the engine's `extract_energy` table and `move_energy`.

Lifted, with citations at the pinned engine:
- `stat_index` (`calc_bonuses:2065-2073`);
- `blows` (`calc_blows:1703-1735`, with `adj_str_blow`, `adj_dex_blow` and
  `blows_table`). better-borg-c milestone 8's fix to the C Borg's copy
  (`668ac32`, weigh one weapon, not the stack) agrees with the engine, so
  the engine's formula is used.

Uses:
- `Knowledge::turns_per_action` replaces the placeholder speed rule in
  `Track::confidence`;
- `Character::blows` and `melee_per_turn` give expected damage per turn for
  any weapon (blows times mean dice plus known to-dam, before hit chance,
  criticals and slays, which belong to M4's threat model).

The Observation gained the engine's own `stat_ind`, `num_blows` and the
O-combat option, as checks. Tests:
- unit tests use the C values worked by hand;
- `BORG_KNOWLEDGE_LOG=FILE` dumps the knowledge, and CI checks it against
  monster.txt and class.txt values and against the engine's `adj_str_blow`;
- on every recorded think our stat indices and blows equal the engine's:
  locally 595 thinks over all 9 classes, and in CI the Warrior run.

Play is unchanged (smoke 3/3, full 45/45, hallucination smoke 3/3). The
Rust deaths are recorded again as D0388–D0433.

## 2026-11-09: T2.2, the world model (#13)

`borg-reborn-rs/src/world.rs` holds the L2 `World`. It changes only in
`World::update(&Observation)`, once per think. It has:
- the current level's map memory: the feature last shown, when the grid
  was last in view, the remembered and the real top object, trap and web.
  An unknown grid never erases what was known. Leaving the level (a depth
  or level-size change) forgets it, as the C Borg does;
- monster tracks keyed by the game's monster index: race, position, first
  and last seen, HP, timers and speed. An index reused for another race is
  a new monster. A track out of view is kept, and `Track::confidence`
  falls with the time since it was seen and with the monster's speed
  (asleep, a quarter as fast). Its shape is a placeholder for M4's
  calibration;
- the town's shop entrances (kept across levels) with the last visit (the
  store header "Stor"/"Home" while standing on the entrance), and every
  store's stock and the home's, as last read;
- the character model: the latest status and gear, plus `GearBonuses`
  (to-hit, to-dam, AC, known flags and modifiers, and resistances combined
  as `calc_bonuses` does). `Character::wearing` gives a changed copy for
  "try this on", and `pack_free` uses the engine's own count.

The Observation gained the engine's pack count (`pack_slots_used`,
`z_info->pack_size`) and every store's stock (`borg_notice_store`: each
ware through the item analysis, with the shop's `price_item`).
`BORG_WORLD_LOG=FILE` writes the world's digest after every think. CI
records a 3,000-turn run with both logs, and checks that replaying the
observations without the game rebuilds the live world think by think (302
of 302 locally). Play is unchanged (Rust smoke 3/3, full 45/45 and
hallucination smoke identical), but the store reads and the update make the
suites about 25% slower. The Rust deaths are recorded again as
D0342–D0387.

Left for later tasks: expected damage per turn needs blows and the other
formulas of L3 (T2.3). Item curses and activations are not observed yet.
The strategy doesn't read the world until M3.

## 2026-11-09: T2.1, the Observation (#12)

`borg-reborn-rs/src/observation.rs` holds the L1 `Observation`. It is
plain, serde-serialisable data, one per think, with what the C Borg's two
channels give it (spec §1.1, GOALS D3):
- from the screen: the message line (row 0, split into messages by
  `parse_messages`) and the store header (row 3, columns 1–4, raw: the
  C Borg compares it with "Stor"/"Home");
- from direct reads: status and timed effects, every equipment slot, the
  quiver and the pack through the C Borg's item analysis (`ODESC_FULL`
  descriptions, the known part unless the item is fully known), the
  class's spells (`spell_flags`, base fail), the map panel through
  `map_info()` (clipped to the level, with known trap, shop number and
  web), the monsters `map_info()` shows with their real race, hit points,
  timers and speed, and the real kind of the top object that is not
  ignored on each grid with a seen object.

`engine::perceive()` does the reads, so `engine.rs` is still the only
`unsafe` module. The Borg perceives on every think, after the minimal
Borg's own reads. The placeholder strategy does not use the observation
yet, and play is unchanged (Rust smoke 3/3, full 45/45 and hallucination
smoke identical). `BORG_OBSERVATION_LOG=FILE` writes one JSON line per
think, about 270 KB each. CI records a 500-turn run and checks that every
line round-trips canonically. New dependencies: `serde` and `serde_json`.
The Rust deaths are recorded again as D0296–D0341.

Not in it yet, for T2.2–T2.5: the store's own inventory (when the Borg is
in a shop), item curses and activations, monster messages beyond the
line, and the oracle comparison with borg-rs's `BorgState` (T2.5).

## 2026-11-09: engine tree re-synced from borg-rs 286215c

`angband/` is now borg-rs `286215c:angband` (tree `3fc0c9f`, split
`6581b4f`), merged as a subtree (docs/ENGINE-TREE.md). Only
`angband/src/borg/` changed: borg-rs carried better-borg-c milestones 7
and 8. So the tree's own C Borg (`BORG_IMPL=c`, never a control) goes
from impact v6 to v8. It includes `borg_munchkin_by_class`, TRUE by
default: Warriors, Druids and Rangers take the munchkin start. The engine
and borg-rs's generated headers are unchanged.
- `harness/reference/smoke-c/` is replaced by a smoke run of borg-rs's own
  C build at `286215c`. Ours reproduces it byte for byte (3/3), the same
  import check as T1.2.
- A re-sync starts a new measurement baseline (RULES 22). The source hash
  of every arm includes `angband/`, so deaths from runs after this PR
  are new encounter families.
- `harness/borg.txt` is unchanged. Whether a control should run with
  `borg_munchkin_by_class = FALSE` (GOALS D5) is asked on #56: better-borg-c
  v8 has the option too.

## 2026-11-09: M2 source refresh

The M2 `--update` (RULES 21) moved five pins:
- borg-rs `e5cb263` → `286215c`: two docs changed, and the engine tree
  re-sync is available (next PR);
- better-borg-c `b1588cc` → `eb9a78e`: new deaths D0015 and D0016;
- strategy-manual `e67fd7d` → `86349ca`;
- borg-zoo `3f6b76e` → `e56d5fa`;
- borg-agent `9c58125` → `77e48c9`.

angband and neo-angband-tracker are at their pins.
- **S12 repointed**: the pin `strategy-manual` now follows
  neo-angband-tracker `workbench/c-borg` (its own clone
  `vendor/neo-angband-tracker-workbench-c-borg`), since `c-borg-reference`
  is now pristine upstream. The manual is complete there: chapter 3
  (`03-magic.md`) is new. S12 stays the draft of S14 until the owner
  retires it.
- **New deaths** (better-borg-c, both CL1 Necromancers, original-C
  mechanisms):
  - D0015: Fang, in a dark dead end seven squares from an up staircase.
    The C Borg forbids the stair dash from Grip or Fang, while another rule
    lists Fang as a reason to leave the level.
  - D0016: a fast crow after a teleport trap. The starting gold went on
    armour and unidentified rings, Recall stayed at home, and two known
    Cure Light Wounds were never offered while fleeing.

  Both are input for M4 (escapes, the tactical layer: retreat to stairs,
  healing while fleeing) and M5 (readiness: a survival stock before
  descending).
- **borg-zoo** at `e56d5fa` has borg-zoo#30 done (borg-zoo#32): the
  specimens `4.2.6-reborn-rust`, `-c-pristine` and `-c-bbc-m7` use
  `ref` `2dd11fd`, with `exact_turns`, the depth stream and kept death
  evidence. A comment on borg-zoo#30 gives them the `m1` commit
  (`fab7155`), and says what changed since `2dd11fd`: no keys change in
  our suites, but the Rust source hash does.

## 2026-11-09: a new control arm, c-bbc-m8 (#56)

better-borg-c tagged milestone 8 (`4.2.6-impact-v8`, `017e038`) before
our better-borg-c pin moved past it. The refresh report listed only tags
after the pin, so it never showed the tag. It now lists tags after the
baseline in use (`baseline.commit`, else the pin), which flags v8. Under
RULES 22, v8 is a new arm, `c-bbc-m8`, filed as #56 in M4, the first
milestone that measures against the controls. `c-bbc-m7` stays as it is,
and the GOALS wording "`c-bbc-m7` today" changes when #56 lands.

## 2026-11-09: M1 done; M2 starts

The owner merged the M1 review PR #39 (`main` → `reviewed`, merge
`fab7155`) after four review passes by Kimi K3.1 Pro. Tag `m1` is
annotated and points to that commit. Its message says "merged 2026-11-10",
following the dates of the entries below, but the merge was on
2026-11-09; tags are not moved (RULES 5). Those entries, marked
2026-11-10, are from 2026-11-09 too. The GitHub milestone M1 is closed,
and PLAN.md marks M2 in progress.

After the review, as the owner decided:
- RULES 7 now lets a session open issues, and keep its own issues up to
  date, on borg-zoo, borg-rs, better-borg-c, neo-angband-tracker and
  borg-agent, when the owner asks or when this repository needs a change
  there. Nothing is pushed or merged there, and angband/angband stays
  off limits.
- The zoo change request is
  [diegoami/borg-zoo#30](https://github.com/diegoami/borg-zoo/issues/30).
  docs/ZOO-CHANGE-REQUEST.md names the `m1` commit `fab71554fd32`.
  The issue's prompt is updated to that exact commit after this PR is
  merged, because its old fallback (`4236081`, unpeeled tag) is obsolete.

Found while re-reading RULES: commits and PR bodies since M0 carried
`Co-Authored-By` and "Generated with" lines, which RULES 4 forbids. They
stay, since published commits are not amended (RULES 5), and no new
commit has them.

Next, in M2 (each its own PR):
1. The source refresh: `--update` for better-borg-c (new deaths D0015 and
   D0016) and the other moved pins. Repoint `strategy-manual` (S12) to
   neo-angband-tracker `workbench/c-borg` so the copy keeps a source; the
   owner may still retire S12 in favour of S14.
2. The borg-rs impact-v7 engine re-sync (docs/ENGINE-TREE.md).
3. Then #12 (T2.1, Observation) and the rest of M2 in order.

Still open for the owner: whether the self-produced `smoke-c-bbc-m7`
reference is acceptable, and S12's retirement.

## 2026-11-10: M1 fourth review pass, findings S1–S2 (#39)

The fourth pass (Kimi K3.1 Pro, at `eff27c9`) resolved R1, R2, R4, R5 and
R6, and partly resolved R3. It added two should-fix findings in R3's new
extra-option load path; nothing blocks. Both are fixed in #54. run.sh now
handles `-- -load FILE` like `save=NAME`: it copies the file to
`OUT/save/input.save`, passes that absolute path to borgh, and drops `-n`.
Before, borgh received `-n`, so it started a new character that the summary
called loaded (S1). A relative path also failed after run.sh's `cd` (S2).
`harness/check-extra-load.sh`, now in CI, checks that a relative extra
load resumes the saved game and plays the same keys as `save=NAME`.

## 2026-11-10: M1 third review pass, findings R1–R6 (#39)

The third pass (Kimi K3.1 Pro, at `42ee347`) resolved N1, N3, N4, N5 and
N8, and resolved N6 and N10 for their cases. It partly resolved N2, N7, N9
and N11, and added six should-fix findings, R1–R6. Nothing blocks. The fixes
go in three PRs into `main`:
- #51, R2: a step off a shop can be refused without time (an afraid player
  walking into a visible monster), so on a shop `stand_still()` now alters
  a neighbour (`+` and a direction), which always spends a move's energy.
  Rust play is unchanged (full 45/45, smoke 3/3, smoke-exact 3/3
  identical); the deaths are recorded again as D0188–D0233.
- #52, evidence: R1, R3 and R4. check-boundary compares `depths.json`'s
  `stop_turn` with the summary. The death exporter takes a loaded save
  from borgh's `loaded`, so `run.sh ... -- -load FILE` counts, and run.sh
  records that save's hash and an extra `-keep-load-rng` in `config.txt`.
  zoo-check imports the zoo from a `git archive` of the pin, not the
  checkout, so an ignored `.pyc` cannot replace pinned code.
- sources (this PR): R5 and R6. The update snapshot backs up into a fresh
  directory of its own and never touches an existing `.refresh-backup-*`.
  The zoo prompt peels annotated tag `m1` to its commit (`m1^{}`). The
  copy on borg-zoo#30 needs the same fix, with N11's, when its link is
  recorded.

## 2026-11-10: M1 re-review findings N1–N11 (#39)

The re-review (Kimi K3.1 Pro, at `2dd11fd` and `16af1ad`) resolved 12 of
the 19 findings, partly resolved 7, and added N1–N11. N1 blocks. The fixes
go in PRs into `main`:
- measurement: N1, N3, N5 and N6. `suite.sh` writes each run's
  `run_exit`. `tools/endpoint.py` scores no run of a suite whose binary
  changed (`binary_changed`), and in a `FAILED` suite only a run whose
  run.sh succeeded. A `depths.json` that contradicts `summary.json` is an
  `evidence_conflict`. It refuses outputs with no DONE or FAILED marker,
  and never guesses the class of a run when its index played two classes.
  `harness/check-boundary.py` checks the suite hash and fails survivors
  that stopped before the cap.
- #48, N2: on a shop entrance `,` enters the store and takes no time, so
  `stand_still()` steps off instead of holding. Rust play is unchanged
  (full 45/45, smoke 3/3 identical), but the source moved, so the Rust deaths
  are recorded again as D0142–D0187.
- #49, death exporter: N4, N7 and N8. `Interrupting` and empty causes are not
  natural; `keep-load-rng` drops the seed only with `save=`; a random
  birth's terminal record names the played indices.
- sources (this PR): N9, N10 and N11. zoo-check refuses a modified zoo at
  its pin; `--update` rolls back a failed phase two; the zoo prompt needs
  tag `m1` or an exact commit from the owner. The prompt already on
  borg-zoo#30 still has the old fallback, so update it when the link is
  recorded.

**Source moved (owner, 2026-11-10):** neo-angband-tracker's
`c-borg-reference` branch is now pristine upstream (`e473720`), with no
`docs/strategy/`. The manuals live on `workbench/c-borg` (`842dc5d`):
`docs/user-manual.md` and `docs/strategy/`. The S12 pin `e67fd7d` is still
reachable from `workbench/c-borg` (and from `docs/strategy-manual`, which
the owner may delete). Do not run `--update` on `strategy-manual` before
the pin is moved: it follows `c-borg-reference`, which no longer has the
directory. In the post-review refresh PR, repoint the pin to branch
`workbench/c-borg`, or retire S12 in favour of S14 (owner's call, still
open).

## 2026-11-10: source S14, the complete Angband Strategy Manual

The owner added neo-angband-tracker `docs/c-manual/` (default branch) as a
source. It is copied to `reference/c-manual/` by a new copy rule on the
`neo-angband-tracker` pin, moved to `777eea1` with `--update` (the C Borg
spec and plan copies are unchanged there). It is the finished version of
S12's draft: 17 chapters, appendices A1–A10, and the official manual. Chapter
15, *Notes for the borg*, is direct input for M2–M6 strategy and parameters.
S12 stays pinned until the owner retires it.

## 2026-11-10: M1 review findings (#39)

The M1 review (Kimi K3.1 Pro, on #39) had 4 blocking, 13 should-fix and 2
nit findings. They are answered in four PRs into `main`, each with a reply
on its comments:
- #42: RNG isolation of perception (`with_borg_rng`, hallucination
  regression in CI), and no loop without game time.
- #43: endpoint scoring (exact boundary, process receipts, per-class
  attempts, victory, pairing, zoo boundary by kind) and suite manifests.
- #44: death records (natural-death checks, family identity, index repair,
  attempt dedupe, registration on publication refusal, zoo deaths
  repatriated, CI death reconciliation).
- the sources PR: an atomic `--update`, and the zoo request (death
  evidence, boundary for every adapter, no stale fallback ref).

Rust play is unchanged by the fixes (full suite 45/45 identical), but #42
changed the Rust build inputs, so the Rust deaths are re-recorded at the new
source. The zoo change request was filed as borg-zoo#30 at the owner's
request; the issue needs the corrections of this PR (owner's call).

**After the re-review and the owner's merge of #39:** tag `m1`; amend
RULES 7 (owner decision 2026-11-10: sessions may open issues, not push,
on the family's repositories) and record borg-zoo#30 in this file and in
ZOO-CHANGE-REQUEST.md; then start M2 with a refresh PR (better-borg-c
D0015–D0016, borg-rs impact v7 engine re-sync as its own PR).

## 2026-11-09: two sources added (owner request)

- **S12, the Angband Strategy Manual:** neo-angband-tracker `docs/strategy/`
  on branch `c-borg-reference` (not the default branch), copied in
  `reference/strategy/` at `e67fd7d`. A new pin `strategy-manual` with two new
  pin fields: `branch` (the branch the refresh report and `--update` follow)
  and `vendor` (its own clone, `vendor/neo-angband-tracker-c-borg-reference`,
  so it never fights the default-branch clone over a checkout).
  `scripts/refresh-sources.py` and `scripts/setup.sh` support both; a new test
  covers a branch pin. The manual is a work in progress (chapter 3 missing)
  and moved twice on 2026-11-09: expect the refresh report to show it.
- **S13, the official command descriptions**
  (angband.readthedocs.io `command.html`): built from upstream
  `docs/command.rst`. Upstream `master` is at our engine pin `e473720`, so the
  page is `angband/docs/command.rst` in our tree; nothing copied.
- Not done: no strategy is taken from either yet. That is M3+ work; lifted
  knowledge cites the engine source, not the manual (docs/SOURCES.md S12).

## 2026-11-09, end of session: M1 under review

**State**
- M0 is reviewed (tag `m0`). **M1 is complete and in review:** PR #39
  (`main` → `reviewed`). All M1 issues (#3–#11) are closed. Do **not** start
  M2 (#12–#16) before the owner merges #39 (RULES 6).
- CI on `main` is green: the `rust` and `engine` jobs, with all four arms,
  controls against references, Rust determinism and exact boundary, and the
  tool tests.

**Waiting on the owner**
1. The M1 review on PR #39. Its body has the reviewer prompt.
2. Tag `m1` on the merge of #39. The zoo specimens should point at it.
3. Routing the zoo change request. The owner asked for a prompt for a
   borg-zoo session; it is the appendix of
   [ZOO-CHANGE-REQUEST.md](ZOO-CHANGE-REQUEST.md). The zoo work happens in
   borg-zoo, not here.
4. Whether the self-produced `harness/reference/smoke-c-bbc-m7` is enough as a
   regression reference.

**Next session, in order**
1. `git pull`, `bash scripts/setup.sh`, and `python3 scripts/refresh-sources.py`.
   Read new deaths first; better-borg-c moves daily (RULES 21).
2. If #39 has review findings: one fix PR into `main` per finding set, a
   reply on each comment, and the re-review. If it is merged: tag `m1` (if the
   owner has not), then start M2 with an `--update` refresh PR (PLAN: *How
   every milestone starts*), then #12 (T2.1 Observation).
3. Check whether borg-zoo has the reborn specimens and exact-boundary cells.
   If it does, the next comparison is G1, which comes after M5 (GOALS), so
   nothing to run yet.

**Local state (not in Git)**
- `build/` (all four arms, the GUI build `build/play-rust`, the death-workflow
  caches), `harness/out/` (all suite outputs, including `t18/`), `vendor/`
  (pinned clones) and the scratch play data. All of it can be regenerated
  with the commands in harness/README.md and AGENTS.md.


## 2026-11-09: T1.8 every-arm check, zoo compatibility and change request (#11) — M1 complete

- **Every arm** (`c`, `c-pristine`, `c-bbc-m7`, `rust`) × smoke and full:
  - no harness failure;
  - the exact-boundary contract holds (`harness/check-boundary.py`,
    192/192 runs);
  - a rerun gives byte-identical keys (192/192);
  - the oracles still hold (`c` = `smoke-c`, `c-pristine` = `full-pristine`
    45/45).
- **Zoo:** `harness/zoo-check.py` runs the pinned zoo's own borg-rs adapter
  (in-memory specimens, no zoo change). All three arms run on the zoo's
  smoke with no harness failure or config error. The gaps are survivors
  stopping after the cap and an empty `first_turn_at_depth`.
  [ZOO-CHANGE-REQUEST.md](ZOO-CHANGE-REQUEST.md) is **for the owner to
  route**: three catalogue specimens; the adapter's `-exact-turns`,
  boundary field and depth stream; optionally the endpoint in stats.
- Controls now build from a fresh checkout: the pinned sources are cloned on
  demand and missing blobs fetched in one batch (`c-bbc-m7` in 16 s). CI
  builds both controls, checks them against `smoke-pristine` and the new
  self-produced `smoke-c-bbc-m7`, and checks the Rust arm's exact boundary.
- The Rust arm's first full-suite deaths are recorded.

**Next:** the **M1 review PR** (`main` → `reviewed`). Do not start M2 before
the owner merges it (RULES 6).

## 2026-11-09: T1.7 endpoint calculator (#10)

- `tools/endpoint.py` computes the GOALS primary endpoint per arm and class:
  alive at T with max depth ≥ D, or a victory before T, over **every
  attempted run**. Operational outcomes are listed separately, with Wilson
  95% CIs and paired exact McNemar tests on shared (seed, class).
- Inputs are local suite outputs (runs listed in the suite but missing count
  as harness failures) and borg-zoo cells (filtered by specimen and profile).
  Survivors not observed at the exact boundary are refused unless
  `--allow-after-key`; zoo borg-rs-adapter cells are after-key until T1.8's
  request is done. Seven unit tests.
- **Demonstrations (descriptive, not claims):**
  - our full-exact suite at D=5, T=100k: `c` 2/45, `c-bbc-m7` 2/45,
    `c-pristine` 1/45;
  - zoo `classes-milestones` (observer, 1800 per specimen): pristine 3.0%
    [2.3, 3.9], bbc-m4 4.7% [3.8, 5.7]; Warrior 15.5% against 20.0%,
    Druid 0.5% against 0.0%. That is post hoc on existing data.

**Next:** #11 (T1.8: every-arm check including `rust`, zoo compatibility,
change request). Then the M1 review PR.

## 2026-11-09: T1.6 launchers, recording, replay, death tooling (#9)

- borg-rs's launchers and `tools/` are copied from `e5cb263`, with the same
  commands. The adaptations are listed at the top of `tools/README.md`:
  `.angband-reborn/`, build-input lists naming our crate, two tests on `HEAD`
  instead of `m5`, and the aggregator accepting our basic dossiers.
- `run.sh` calls the depth stream and the basic exporter only when `tools/`
  is present. The recording launchers run a frozen copy without it and record
  deaths in borg-rs's full format, so there is no double record.
- The Rust Borg now honours **any key stops it** (`Term_inkey`, as the C
  Borg's `user_abort`). Harness keys are unchanged (3/3 identical).
- **Checked:**
  - a headless recording (`run-borg-headless.py`) replays **VERIFIED**;
  - a GUI session under Xvfb (`run-rust-borg.sh`, Ctrl-Z, z, Escape stops,
    Ctrl-X saves and quits) records completely and replays **VERIFIED**;
  - borg-rs's 12 GUI-recording tests pass with that fixture;
  - the tool suite passes 87 tests (26 skip for lack of fixtures) and now
    runs in CI's engine job.
- `docs/deaths/` now has borg-rs's aggregated `index.json` and `README.md`,
  regenerated after every export.
- Observed during the GUI test: the placeholder Borg dies quickly (an
  apprentice on level 2), as expected until M3.

**Next:** #10 (T1.7, endpoint calculator), then #11 (T1.8).

## 2026-11-09: T1.5 engine boundary and minimal Borg (#8)

- `borg-reborn-rs/src/headers/` holds borg-rs's c2rust engine bindings,
  copied unchanged with attribution. `engine.rs` is our own small boundary:
  the player view, `map_info` terrain, the `^Z` command key, options, the
  `inkey_hack` hook, `do_cmd_borg`, and `borgh_decision` behind the `borgh`
  feature. It is the only module with `unsafe`.
- `borg.rs` is the control loop of ARCHITECTURE.md. Queued keys go first;
  rest and repeat get `EVT_NONE`; every action starts with `ESCAPE`; the
  Borg's own RNG is used and the game's never. On top of it sits a
  **placeholder** policy: take a down stair underfoot, rest when below 70%
  HP (never twice without time passing), else walk to a random neighbour
  not known to be a wall or shop. Six unit tests cover the policy.
- `^Z z` starts it, so borgh's start script and the GUI work unchanged. It
  sets `auto_more` and `easy_alter`.
- **Checked:** the Rust smoke suite runs with no harness failure, and two
  runs give byte-identical keys (3/3). The Elf Mage of seed 102 dies to a
  rock lizard at turn 743, and its dossier is recorded. CI now runs Rust
  smoke twice and compares.
- Not here: launchers (T1.6), and the Rust arm's exact-boundary check (T1.8).

## 2026-11-09: T1.4 run contract: exact boundary and depth stream (#7)

- **No engine-tree edit was needed.** borg-rs's borgh already stops at engine
  turn T from its per-turn hook (`-exact-turns`) and logs depth on every key.
  `run.sh` gains the flag `exact`. `tools/depth-stream.py` writes
  `depths.json` after every run (boundary rule, alive at stop, maximum depth
  at stop, first turn at each depth), and dossiers include it. There are new
  suites `smoke-exact` and `full-exact`; the existing suites keep their
  semantics and references.
- **Checked on every C arm** (`c`, `c-pristine`, `c-bbc-m7`), smoke and full
  (144 runs) with `harness/check-boundary.py`. Keys before the boundary are
  byte-identical to the normal runs, survivors stop exactly at the cap, and
  deaths before the cap are the same deaths. The `rust` arm gets the same
  check in T1.8.
- The `c` arm's first full-suite deaths are recorded, from a clean build.

**Next:** #10 (T1.7, endpoint calculator; needs only T1.4) and #8 (T1.5,
the minimal Borg).

## 2026-11-09: T1.3 control C Borgs (#6)

- `BORG_IMPL=c-pristine` and `c-bbc-m7` build our engine tree with a pinned C
  Borg overlaid in `build/<impl>.src/` (`harness/control-overlay.py`,
  `controls` in `pins.json`). The overlay is checked blob for blob, and the
  pinned source is part of the build-input hash.
- **`c-pristine` is byte-identical to borg-rs's pristine-C references** (the
  zoo's `4.2.6-rs-m0-c`): smoke 3/3 and full 45/45, in keys, messages and
  state. Our engine's only gameplay correction (#474) was not reached.
- **`c-bbc-m7`:** tag `4.2.6-impact-v7` changes nothing in the engine outside
  `src/borg/` (audited). It ran smoke and full with no harness failure. No
  byte oracle exists under borgh.
- **Deaths recorded (RULES 12):** 15 `c-pristine` and 16 `c-bbc-m7` deaths on
  the full suite are in `docs/deaths/`, unassessed. This is one run each, so
  it is descriptive, not a comparison (RULES 14).
- **Note for T1.8 (#11):** without the `menu` flag, borg-rs's full suite plays
  other classes for index 8; the zoo passes `-menu-letters`.
- Controls are not in CI yet: they need `vendor/` (scripts/setup.sh). The
  local commands are in harness/README.md.

**Next:** #7 (T1.4, run contract: exact boundary and depth stream; borgh
already has `opt_exact_turns`).

## 2026-11-09: REDESIGN-LESSONS.md read into the design

- better-borg-c wrote `docs/deaths/REDESIGN-LESSONS.md` for this repository:
  thirteen lessons from D0001–D0014 and four owner manual takeovers. It came
  in with refresh #31, together with death D0014.
- [LESSONS-TRACE.md](LESSONS-TRACE.md) maps each lesson to our principle,
  our architecture section and the task that builds it.
- GOALS principles 4–8 are extended, and principle 14 is new (equipment by
  what it does). D3 now states that the threat model uses full race data
  (spoiler knowledge), as the C Borg does.
- ARCHITECTURE adds monster-track confidence, one assessment and tolerance
  for every consumer, cache validity, the escape ladder, consumable value
  and destination-aware level changes.
- PLAN adds D0014 to the fixtures, and owner manual branches as scenario
  tests.
- The refresh report now names new and changed registry documents.

**Next:** #6 (T1.3, control C Borgs).

## 2026-11-09: T1.2 harness, build identity, death export (#5)

- `harness/` is borg-rs's at `e5cb263`, with three adaptations listed in
  [harness/README.md](../harness/README.md):
  - `identity.sh` names our crates;
  - `run.sh` ends with `tools/export-death.py`;
  - `measurement/rust-overlay.py` copies both workspace members.
- `BORG_IMPL=rust` links through the seam package `borg-rs/` (the name the
  imported CMake builds), which re-exports `borg-reborn-rs`. The engine
  boundary module `engine` has a no-op `do_cmd_borg` until T1.5.
- **Import proved:** the `c` smoke suite is byte-identical (keys, messages,
  state) to borg-rs's own C build at `e5cb263`. That output is
  `harness/reference/smoke-c`. borg-rs's committed smoke reference is the
  pristine-C one (unchanged since `aa4ad6c`) and is kept as
  `harness/reference/smoke-pristine` for T1.3.
- **First death:** smoke-mage-2 (seed 102, Elf Mage, `c` arm) dies to an
  acolyte at turn 7195. It is the same as in borg-rs. Its dossier is our first
  case in `docs/deaths/`.
- CI gains an `engine` job: build `c`, run smoke, diff against `smoke-c`,
  build `rust`, check that the binary reads as `rust`.

**Next:** #6 (T1.3, control C Borgs) and #7 (T1.4, which needs T1.3).

## 2026-11-09: T1.1 engine tree imported (#4)

- `angband/` is borg-rs `e5cb263` (impact v6) `angband/`, split with
  `git subtree --ignore-joins` into 29 commits and added unsquashed. The tree id
  `5cc1b1c…` is recorded in `pins.json` (`engine_tree`), and a repository test
  pins it.
- [ENGINE-TREE.md](ENGINE-TREE.md) lists what differs from upstream
  `e473720`:
  - 8 added files (borgh, recording);
  - 14 engine files, all behind `BORG_HARNESS`/`BORG_RECORDING` except
    CMake, `main.h` declarations and the #474 cure-all correction;
  - 26 C Borg files (borg-rs's adopted milestones 2–6).
- It also gives the re-sync procedure.
- borg-rs's `FORK-CORRECTIONS.md` and `adoption/status.json` are copied to
  `reference/borg-rs/` by a new copy rule. The refresh report flags when
  borg-rs's `angband/` tree moves.
- Nothing is built yet: building and the harness are T1.2 (#5).

**Next:** #5 (T1.2: harness, build identity, death export before the first
game).

## 2026-11-09: new deaths read into the evidence

better-borg-c D0011–D0013 (from the first refresh) are folded in. SOURCES
has three new death patterns (fear by list, unknown items left unknown,
wrong kit valuation) and extended rows. GOALS principles 4–8 cite them, and
PLAN's death fixtures include them. The research digest
(`docs/research/DEATHS-AND-ISSUES.md`) is a dated snapshot of 2026-11-08;
`reference/deaths/INDEX.md` is the live list.

**Next:** #4 (T1.1, import the engine tree from borg-rs at `e5cb263`).

## 2026-11-09: T1.0 source refresh routine (#3)

- `scripts/refresh-sources.py` reports, per pin: commits since the pin, new
  and changed death cases, new tags (new control arms), changed C Borg
  functions we cite, and issues updated since the pin was read. `--update`
  moves the pins, refreshes `reference/` (copy rules in `pins.json`) and
  regenerates `reference/deaths/INDEX.md`. Tests on synthetic repositories
  cover the report, update, dirty vendor and offline cases; CI runs them.
- RULES 21–23: every session starts with the report, every milestone with an
  `--update` PR; baselines are exact; lifted knowledge is watched.
- **First `--update` (PR closing #3):** borg-rs `6b58a46`→`e5cb263` (impact v6; its C Borg now differs from upstream in 26 files, issue #4 updated), better-borg-c `63fbc1f`→`8b26042` (deaths D0011–D0013 copied), borg-zoo `ffe5a05`→`9cd7423`, borg-agent `697abde`→`8f1dd7f`. The new deaths are read into SOURCES/GOALS/PLAN in a separate PR (RULES 21).
- **First real report (2026-11-09):** better-borg-c has three new deaths
  (D0011–D0013), 25 commits and 75 updated issues; borg-rs moved 7 commits,
  borg-zoo 3, borg-agent 9. The `--update` PR that brings them in is the
  last step of #3.

## 2026-11-09: M0 reviewed, M1 started, backlog moved to issues

- The owner merged the M0 review (#1); tag `m0` is on the merge
  (`ae64f12`). D1–D11 are accepted (GOALS.md).
- The backlog is now GitHub issues (owner request): GitHub milestones
  M1–M7+, labels `task`, `process`, `in progress` and `owner`, and issues
  #3–#24 for T1.0–T3.8, each with a **Needs** line linking its
  dependencies. PLAN.md keeps only milestones, exits and method.
- **Next:** #3 (T1.0, the source-refresh routine over every pin, death
  reports included; owner request), then #4 (T1.1, import the engine tree
  at refreshed pins).

## 2026-11-08: M0 review findings addressed (T0.4)

The owner's review of PR #1 had six findings, all verified and addressed
in the T0.4 PR:

| Finding | Change |
| --- | --- |
| Blocking: the primary endpoint has no tooling that computes it. borgh overshoots the cap, the adapter leaves `first_turn_at_depth` empty, and zoo stats exclude operational outcomes | GOALS *Observation and scoring rule*; PLAN T1.4 (exact boundary, depth stream), T1.7 (`tools/endpoint.py`), T1.8 (zoo change request); G1 prerequisites stated |
| Blocking: the imported `BORG_IMPL=c` is borg-rs's adopted C Borg (22 files differ from upstream), and observer-harness zoo cells cannot be paired with borgh | PLAN T1.1 corrected; new T1.3: `c-pristine` and `c-bbc-m7` controls built here on the same engine and `borgh`, `c-pristine` checked against the zoo's `4.2.6-rs-m0-c`; GOALS D2/D8/G1/G2 |
| setup `--strict` did not check that checkouts were at the pins | `scripts/setup.sh` checks out pins, reports WRONG REVISION, never moves dirty checkouts, `--checkout` moves clean ones (all three paths tested) |
| Death copies missed `overall-assessment.md` and other small evidence files | All root-level `.md`/`.json` of each case copied; external links listed in `reference/README.md`; a test fails on dangling internal links (checked that it fails when an assessment is removed) |
| Death export came after the first playing run | Minimal death export is part of T1.2, before its first smoke run; RULES 12 applies from the first game. Follow-up review: T1.4 checks only the C arms, and the every-arm check (`rust` included) is in T1.8 and the M1 exit |
| Nit: `BORG_IMPL=reborn` | `rust` everywhere, as borg-rs |

Follow-up review on #2: borg-rs is reclassified everywhere as engine and
tooling source and perception oracle, not a control. Its zoo results are
context (GOALS recommendation, non-goals and D8; AGENTS).

Still open for the owner: D1–D11, and routing the zoo change request once
T1.8 writes it.

## 2026-11-08: milestone 0 (foundation), ready for review

**Done**
- T0.1: sources read and evaluated ([SOURCES.md](SOURCES.md)). Specification,
  plan, curated death dossiers and zoo catalogue copied to `reference/`
  with provenance. Prior art outside Angband surveyed (AutoAscend, BotHack,
  TAEB/Saiph, qw, Rog-O-Matic, NetPlay). A digest of deaths and issues
  by subsystem is in [research/DEATHS-AND-ISSUES.md](research/DEATHS-AND-ISSUES.md).
- T0.2: goals, principles, staged targets and eleven owner decisions,
  all *proposed* ([GOALS.md](GOALS.md)). The design direction is in
  [ARCHITECTURE.md](ARCHITECTURE.md).
- T0.3: [PLAN.md](../PLAN.md) (M0–M7, tasks for M1–M3),
  [RULES.md](../RULES.md), [AGENTS.md](../AGENTS.md) (model-agnostic entry;
  `CLAUDE.md` only includes it), `pins.json`, `scripts/setup.sh`, the
  empty crate `borg-reborn-rs/` (`staticlib` + `rlib`, as borg-rs),
  repository consistency tests, CI.

**For the owner to decide (milestone 0 review)**
- D1–D11 in [GOALS.md](GOALS.md#owner-decisions). The ones that shape M1
  most: D2 (engine tree from borg-rs), D3 (information level), D4 (reuse
  of borg-rs code), D9 (workflow).
- Whether the `reviewed` branch starts at the root commit (so the M0 review
  PR shows everything), as set up here.

**Next**
- Done on 2026-11-09: see the newest entry above.

**Known limits**
- Nothing is built against the engine yet. The CI checks only the empty
  crate and the documents.
- The death digest was produced with a model's help from the issue bodies
  and dossiers at the pinned commits. Its classifications are leads, not
  findings. SOURCES.md's table of death patterns was checked by hand
  against the dossiers' `assessment.json`.
