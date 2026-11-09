# Handover

*Where the work stands, for the next session. Update it in every PR.*

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
