## 2026-11-23: Phase Door into never-seen grids, and the newcomer at range (#1066)

Every check below ran on the **default core, borg-rs** (`borg-core/`). No
check ran on the reborn core: this change does not touch it. Base: track-p
`21c0d9c1`. Branch: `claude/affectionate-babbage-4hl2h5` (the session's
assigned branch, not a `task/` name).

### What was done

Two switches, each with an HP floor, all four compiled **0 / 50**, so today's
play is kept. `harness/borg.txt` does not set them. The registry now holds
2418 parameters (2414 before).

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
  borg-core` and `cargo test --release -p borg-core` (712 passed) all pass,
  and so does `scripts/check-param-citations.py`. `--fix` re-pointed the
  citations that my line shifts moved.
- Three new unit tests: the HP floor, plus source pins showing that only the
  Phase Door check counts unknown grids and that the wait follows a lost
  verdict and races only the adjacent monsters.
- Smoke: 3/3 runs identical to `harness/reference/smoke-borg-rs`, so it is
  not re-recorded.

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

### Left, and for the owner

- **To decide:** the unknown count's strictness. It is literal now (every
  unknown sample is scary). A softer variant would count an unknown grid as
  scary only when it borders a known floor grid, or give it a weight.
- **To decide:** whether the newcomer wait should need the newcomer to be
  outside melee reach for more than one turn. Today it needs only a distance
  above 1. With the wait, the Priest's apprentice blinded it a turn later
  (the race does not price a newcomer's spells).
- The zoo comparison (RULES 20) for the two switches, alone and together,
  and the Cemetery counterfactual package: #1073.
