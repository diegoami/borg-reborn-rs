## 2026-11-22: the emergency attack spell casts without mana (#980)

**Done.** I found the root cause. A parameter now stops the cast, and its
default keeps today's play: `attack_reserve_spell_overexert` (1 by default;
0 never casts beyond the current SP). There is a unit test
(`reserve_tests`) and a `docs/CALIBRATION.md` entry. The shipped
`harness/borg.txt` is unchanged.

**Root cause.** `borg_attack_aux_spell_bolt_reserve`
(`borg-core/src/borg_fight_attack.rs:3196`, the C Borg's own function at
`engine/angband/src/borg/borg-fight-attack.c:2520`) is the C Borg's deliberate
emergency cast.
- It runs only when the real-mana check fails (`:3225`, "Must not have enough
  mana right now").
- It fakes `BI_CURSP = BI_MAXSP` for its fail check (`:3292`) and for the cast
  (`:3316`).
- So it misses the engine's +5% fail per missing SP
  (`player-spell.c:404-406`) and the faint that always follows
  (`player-spell.c:540-553`, chance 100, paralysis `1d(5*oops+1)`, past free
  action).
- Its kill test uses twice the average missile damage plus 4. A 9-HP crow
  passes it, so the crow survives the cast.
- The #842 focused ranking (`focused_attack_ranked`) scores that inflated
  damage, at the faked-mana fail rate. It ignores `uses = Some(0)`. So
  `attack on sight ... attack 5` picked it.

**Checked**, on the default borg-rs core (`borg_reborn_core = borg-rs`),
shipped `harness/borg.txt` plus the zoo report's overrides:
- 592754909 on track-p 4b2c2fcf: `b13a6f0fc76ff016`, the zoo's game (death at
  3574). With the branch at default: the same hash. With `= 0`:
  `f115048321c12344`, no emergency cast, but the Borg still dies to the same
  crow at 3569, after a missed flask of oil. That is now #990.
- 1378676237 on track-p plays another game that never reaches this path
  (`ae8060ddaad2f822`, alive at the cap, identical with `= 0`). On main
  f57499d0 with the patch: at default it dies at 7431 after the emergency
  cast (`e4f0cded08e5e7f0`); I could not check that hash against the zoo
  cell. With `= 0` it reaches the 100,000-turn cap alive
  (`4aa7c9abe6b54133`).
- Smoke: 3/3 identical to `smoke-borg-rs`. fmt, clippy, the workspace tests
  and the borg-core release tests pass. `borg-core-vendor.py check` is ok.

**Review (Mavis, MiniMax M3.1, on #991 at 5517892d), processed.**
- I fixed the 32 citations that the inserted lines moved
  (`check-param-citations.py --fix`; the CI blocker).
- The cost at the gate is now normalised with `.max(1)`.
- A test pins the default to 1.
- CALIBRATION now names the `= 0` override, the registry count (2377) and the
  CON damage. `PY_EXERT_CON` is CON damage, not confusion as the review says
  (`player-util.c:825-832`).
- The branch name stays `task/980-...`, the form the owner's brief for this
  session gave.
- The PR body's footer was added by the posting tool; I removed it.

**How the `= 0` arm was played.** The shipped `harness/borg.txt`, plus the zoo
report's overrides, plus the line
`borg_param_attack_reserve_spell_overexert = 0` appended to the run's
`user/borg.txt`. The key is not in the shipped file, so `BORG_TXT_SET` cannot
reach it, and the added line changes `borg_txt_sha256`. After the review
fixes, the hashes are the same: `b13a6f0fc76ff016` at default and
`f115048321c12344` at `= 0`.

**Re-review (Mavis, at 89888283), processed.**
- I merged `track-p` `c15a4e73` (#891/#878, #988, #989, #994): `params.rs`
  took track-p's side plus this row, and the citations were fixed again
  (2389 parameters, 0 wrong).
- On the synced tree, the hashes are unchanged: 592754909 gives
  `b13a6f0fc76ff016` (default) and `f115048321c12344` (`= 0`); 1378676237 gives
  `ae8060ddaad2f822` in every arm. Smoke is 3/3 identical.
- R4, against #878 (`attack_reserve_faint_percent`): see CALIBRATION.
  - The #980 gate comes first, so `= 0` makes both #878 settings inert.
  - On 592754909, #878 at 50% or 100% refuses the same cast and plays the same
    keys as `= 0`. At 100% the refusal is marginal: 3.0 expected damage against
    3 HP.
  - #878 assumes a successful cast kills the target, so it underestimates the
    damage while the missile's kill test is doubled.

**Left.**
- #990: the crow death that remains on 592754909.
- #1013: the last-ditch escapes fake the mana the same way
  (`borg_escape.rs:1778`).
- #1014: the focused ranking ignores `uses = Some(0)`.
- The zoo measurement: borg-zoo request issue, linked from PR #991.
- The replays are not in borg-cemetery yet.

**Owner decides.** Between `attack_reserve_spell_overexert = 0` (no
emergency cast) and #878's `attack_reserve_faint_percent` (a priced one),
after the zoo. `= 0` makes #878's two reserve settings inert, so shipping it
means retiring them or recording that they are superseded.
