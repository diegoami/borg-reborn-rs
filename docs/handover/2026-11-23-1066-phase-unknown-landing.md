## 2026-11-23: Phase Door into never-seen grids, and the newcomer at range (#1066)

Every check below ran on the **default core, borg-rs** (`borg-core/`). No
check ran on the reborn core: this change does not touch it. Base: track-p
`21c0d9c1`. Branch: `claude/affectionate-babbage-4hl2h5` (the session's
assigned branch, not a `task/` name).

### What was done

Two switches, each with an HP floor, compiled **0 / 50**, so today's play is
kept. The review of #1075 added two more values (below). `harness/borg.txt`
sets none of the six. The registry now holds 2425 parameters (2419 before,
at track-p `db23e6dc`).

- `escape_phase_unknown_scary` and `escape_phase_unknown_hp_percent` (50).
  In `caution_phase`, the landing check every Phase Door site asks, a
  sampled grid the Borg has never seen (`FEAT_NONE`) ends its attempt and
  counts as scary. C skips it and draws another grid. This applies only
  while HP is at least 50% of the maximum. The note adds "N of them never
  seen (#1066)". `count_scary` takes the switch, and the teleport check
  passes `false`.
- `escape_phase_newcomer_wait` and `escape_phase_newcomer_hp_percent` (50).
  When the Phase rule (`escape_phase_adjacent_rule`) judges the adjacent
  fight lost, `newcomer_wait` turns the verdict into Fight if three things
  hold:
  - HP is at least 50%;
  - an awake known monster farther than one grid has `borg_danger_one_kill`
    above 0 here;
  - #759's race against the adjacent monsters alone is winning.

  For that race, `winning_fight`'s computation is now `fight_race(st,
  adjacent_only)` in `borg_caution.rs`. `winning_fight` keeps its notes and
  actions. Every refusal of the race still stands, and an unseen attacker
  anywhere still refuses.

Engine facts are in `docs/CALIBRATION.md`. Phase Door is TELEPORT 10
(`object.txt:2571-2581`). The landing is uniform over passable grids at the
chosen distance, whether known or not
(`effect-handler-general.c:132-146, 2600-2650, 2675`).

### Checks

- `cargo fmt --check -p borg-reborn-rs`, `cargo clippy --workspace
  --all-targets -- -D warnings`, `cargo test --workspace --exclude
  borg-core` and `cargo test --release -p borg-core` (712 passed; 719 after merging track-p `db23e6dc`) all pass,
  and so does `scripts/check-param-citations.py`. `--fix` re-pointed the
  citations that my line shifts moved.
- Three new unit tests: the HP floor, plus source pins showing that only the
  Phase Door check counts unknown grids and that the wait follows a lost
  verdict and races only the adjacent monsters.
- Smoke: 3/3 runs identical to `harness/reference/smoke-borg-rs`, before
  and after the merges of `769f3fbd` and `db23e6dc`, so it is not
  re-recorded.

### Replays at the fatal moment (leads, not measurements)

Both deaths play another game on the head, so I put the patch on the zoo's
specimen, `206cf4e7` (code files only). I replayed the zoo bundles of run
45070528012 with their own `user/borg.txt`. The control reproduces both zoo
games key for key (`279610c79e006202`, `9d1b38532c53c82d`).

| arm | Priest 694417276 | Blackguard 2025089708 |
|---|---|---|
| control | dies t28,692 (cave spider) | dies t40,759 (soldier) |
| `escape_phase_newcomer_wait = 1` | **alive** to the cap, CL7; splits **at t28,617** | **alive** to the cap, CL7; splits **at t40,580** |
| `escape_phase_unknown_scary = 1` | dies t31,031 (floating eye); splits at t881 | alive to the cap, CL8 DL6; splits at t7,923 |
| both | as `unknown_scary` | as `unknown_scary` |

- **Newcomer wait, Priest.** At 28,617 the race against the soldier alone
  says 2 turns at 13.5 a turn against HP 41: winning, so no Phase Door. It
  throws oil. Then the apprentice blinds it and it drops to 19 HP. It drinks
  a CLW, then at 21 HP reads Phase Door (63 scary of 100) and again at 13,
  and lives. It is a rescue at the fatal point, but the wait cost about 20 HP
  and the outcome was partly luck.
- **Newcomer wait, Blackguard.** At 40,580, 3 steps from the stairs with the
  soldier in the way, the race says winning (5 turns, 5.5 a turn, HP 42), so
  it fights. From 40,419 to 40,532 the race was not certified (one of #759's
  refusals; the note does not say which) and it did not wait. It drinks 2 CLW and takes the
  staircase at 40,654: the owner's "stand and fight or go back to the
  stairs".
- **Unknown-scary.** This switch changes the first Phase Door check of each
  game: the Priest at t881 (83 of 100 never seen) and the Blackguard at
  t7,923 (88 of 100, where the control had 0). So the two outcomes are a
  different game, not a rescue at the fatal point. The Priest's floating-eye
  death is a paralysis lock 8,000 turns after its last #1066 refusal.
- **Over-refusal.** In that Priest game, 13 of 15 checks refused, with up to
  97 of 100 grids scary only because they were unseen. Most unknown grids
  near a corridor are rock the engine never lands on. Above half HP in an
  unexplored area, Phase Door is effectively off, which is what the issue
  asked for. Whether that is too much is for the owner and the zoo.

The replay script and outputs are in this session's scratchpad. They are
not committed (RULES 18). The canonical case for these counterfactuals
belongs in borg-cemetery, next to the study `lvl1-deaths-trackp-206cf4e7`
(borg-cemetery#138, open).

### Review of #1075 (Codex, GitHub connector, on `e588e6ea`)

The review's findings are in [#1075](https://github.com/diegoami/borg-reborn-rs/pull/1075#issuecomment-7088999549). Both are fixed in the commit after `2ef8ddf9`.

1. **[P2] A ranged enemy can hit before the Borg acts again.** The wait
   now also needs a survivable next turn (`newcomer_turn_safe`). The race's
   HP a turn from the adjacent monsters plus the danger of the monsters at
   range must stay below `escape_phase_newcomer_turn_percent` (100) of HP.
   That danger is read without fear when `escape_phase_lost_damage_only` is
   on (shipped 1), as the rule reads it.
2. **[P2] Phase Door paths that bypass the unknown policy.** These sites
   sample no landing: the critical phase (escape 1.2, which the emergency
   refusal falls through to), last ditch (1.3), the twitchy ones, and
   fast-fragile with `fast_fragile_phase_landing` 0. They now ask a
   never-seen gate in `phase_door`, after the site's own check. It draws
   100 landings as the landing check does (`never_seen_landings`) and
   refuses above `escape_phase_unknown_tolerance` (75). Shoot N Scoot's
   sampler counts never-seen grids as scary. All three read one policy
   (`phase_unknown_policy`).

   The gate decision is the pure function `unknown_gate`. Its test,
   `the_critical_phase_after_a_refused_emergency_asks_the_gate`, plays the
   fall-through:
   - the Blackguard at 43/60 with 88 never-seen landings: refused;
   - at 15/60: read, and the sampler is not run;
   - a `Landing` or `ShootScoot` site is not gated twice.

   A source pin checks the order in `phase_door` (the site's check, then the
   gate, then the cast) and Shoot N Scoot's use of the policy. The sampler
   itself needs the engine's RNG and bounds, so no unit test runs it; the
   replays below do.

Re-checked before and after merging track-p `df84b81e` (#1074, which grew
smoke to 4 runs and shipped the #926 bundle): `cargo test --release -p
borg-core` 721 passed, fmt, clippy, the workspace tests and the citations
pass, and smoke is 4/4 identical to `smoke-borg-rs`. The replays on `206cf4e7` plus the updated patch:

| arm | Priest 694417276 | Blackguard 2025089708 |
|---|---|---|
| control | same keys as the zoo | same keys as the zoo |
| `newcomer_wait = 1` | the same game as before the fix (`9783cb54a70f3570`), alive | the same game (`d9921fd672ef8ed4`), alive |
| `unknown_scary = 1` | alive to the cap, CL8 (`3fe0dd66ef85f6e6`); splits at t881 | alive to the cap, CL8 DL5 (`e29f0e8eced5f742`); splits at t7,923 |

- **The wait passes the new test at both fatal moments.** The Priest's
  apprentice is danger 8 at range (13.5 + 8 < 41). The Blackguard's two
  monsters at range are 11 (5.5 + 11 < 42). The Priest's real loss after
  waiting (41 to 19) came from the apprentice's blinding flash and the
  soldier's blows while blind. The danger without fear does not price that
  blindness, so this test cannot catch it. Reading that danger with fear
  would refuse the wait in the Priest's case, and that is the owner's call.
- **The gate refuses the critical Phase Door.** The Priest's game has 7
  gate refusals (90 to 97 never seen of 100), the Blackguard's 3. The
  Blackguard's first one is at t7,923, the same think where the emergency
  site refused. The over-refusal is now stronger: across both sites, the
  Priest's game refused 67 of 70 landing checks.

### Left, and for the owner

- **To decide:** the unknown count's strictness. It is literal now (every
  unknown sample is scary). A softer variant would count an unknown grid as
  scary only when it borders a known floor grid, or give it a weight.
- **To decide** (review finding 1): whether the next-turn test should read
  the danger with fear, which prices status attacks such as the apprentice's
  blindness. That would refuse the Priest's wait.
- **To decide:** whether the newcomer wait should need the newcomer to be
  outside melee reach for more than one turn. Today it needs only a distance
  above 1. With the wait, the Priest's apprentice blinded it a turn later
  (the race does not price a newcomer's spells).
- The zoo comparison (RULES 20) for the two switches, alone and together,
  and the Cemetery counterfactual package: #1073.
