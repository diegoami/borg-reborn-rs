# M4 munchkin seed1045 stall investigation

Canonical original-C finding: [source403](https://github.com/diegoami/neo-angband-tracker/issues/403).
Tracking: [consumer204](https://github.com/diegoami/better-borg-c/issues/204).
Read-only origin: [borg-zoo workflow42416417481](https://github.com/diegoami/borg-zoo/actions/runs/42416417481),
results branch `results/dive-styles-early/42416417481-1/cells.jsonl`.
This is a liveness investigation, not a survival comparison or a gameplay fix.

## Initial exact reproduction

Harness `8c23ec0d4840246bbc47324a1673fa7aa714c056`, source M4
`70993393a1c946ce8cd38ee558bf1ce6464350e7`. GCC15.2.0
(`cc (Ubuntu 15.2.0-16ubuntu1) 15.2.0`), Linux x86_64, Debug,
observer binary SHA256 `dc72f57fcde361cb0fda19635ed0233370bb87125c645697f8480aa09d1bcb7c`.
The zoo used GCC13.3; local source, harness and engine stall turn match.

From a checkout containing this report (use fresh output paths):

```sh
ZOO1045_REPORT_ROOT="$PWD"
git worktree add --detach /tmp/zoo1045-harness8c 8c23ec0d4840246bbc47324a1673fa7aa714c056
cd /tmp/zoo1045-harness8c
flock /tmp/angband-validation.lock python3 tools/measure-native.py prepare \
  --ref 70993393a1c946ce8cd38ee558bf1ce6464350e7 \
  --observe --output /tmp/zoo1045-m4
flock /tmp/angband-validation.lock python3 "$ZOO1045_REPORT_ROOT/docs/measurement/zoo-stall-1045/run.py" \
  /tmp/zoo1045-m4 /tmp/zoo1045-run-m4
```

The helper calls the pinned runner's `run()` with `configuration` text
`borg_munchkin_start = TRUE`, fixed default Human Warrior birth, seed1045,
cap100000, wall900s, stall90s, no checkpoint saves. The runner appends
cheat-death FALSE and respawn-winners FALSE. The CLI itself has no configuration flag.
The helper must be supplied alongside the historical harness checkout.

Actual result: `stalled`, engine59893, maxdepth6, CL9, HP129, wall185.832s.
Final depth1, decisions20478, totalenergy2076456, exit0/unexpected_stop after
monitor termination. Final depth/decision count depend on the wall-time stop;
`unexpected_stop` and cause `(alive and well)` do not mean cap survival.

## Mechanism observed so far

At fixed engine59893, consecutive 100-decision samples gain10000 total energy.
Notes repeat Power Diving/downstairs cost0, level feeling, Power Climb/upstairs
cost0, level feeling. Native map message history confirms successful up/down
staircases. This contradicts an interpretation of decisions as rejected,
zero-energy keys. Angband's original `on_new_level()` grants minimum move energy;
`run_game_loop()` skips its world-tick/turn increment when generating a level.
The observer counts those original counters rather than causing this behavior.

The native pristine control reaches100000 alive. M3 and carry-prefix tests
localise the triggering trajectory; the loaded-state check demonstrates the
original mechanism. Curated results and exact binary identities are in
[zoo-stall-1045/results.json](zoo-stall-1045/results.json).

## Carry-prefix bisection

Same seed/profile/cap100000/wall900/stall90/no-saves throughout. The ordinary
from-birth samples were pristine,42,34,17,8,4,6,5; the adjacent boundary is:

| Prefix | Immutable source ref | Result |
|---|---|---|
|0|`e473720ed88a0f0c7ca66815cef3000eb4a38bc3`|100000 alive|
|4|`337e50eb6c732bf6c908ab0d7c22b12b3a82b7d3`|100000 alive|
|5|`3638b690525cd8a381cb106b4c5ee40652babde2`|100000 alive|
|6|`45eac63bd9f7cabe3c4ef1607703324ca8dae0fc`|stalled59893|
|8|`4e2af89f34cad98d10131727b3fa4c0e5ddef5fb`|stalled59893|
|17|`a88da7a063221bd56a6e40e5fec472b38286d5c0`|stalled59893|
|34/M3|`702f091f6ac06cae1014df2f85ac218be0189813`|stalled59893|
|42/M4|`70993393a1c946ce8cd38ee558bf1ce6464350e7`|stalled59893|

**First bad carry boundary found by bisection:0006, healing-potion timing**,
consumer31/source77, original source commit
`cedf25eb790f2ff7c7c161cd1612b18ed99fb35e`, applied
`5e9417be42cb1401708a7a37ec1b13641bc04b83`, merge45eac63.
Prefixes0/4/5 have the **same complete trace SHA256** as the zoo's pristine run:
`77f8399dc5cc41277244dc6d199c2a7aac66f7e27d2308c3f042cae98f210ade`.
Prefix6 matches every one of204 shared M4 progress records including complete
RNG. The other failing samples likewise match every shared M4 progress record.
See `bisection.json` and `observer-qualification.json`. This is a sampled
prefix bisection and adjacent5/6 confirmation, not exhaustive testing of every
possible carry subset or a universal monotonicity claim.

Carry6 changes the eligibility of the `randint1(100)` call in
`borg_quaff_crit()`: the corrected recent-drink condition includes a nonzero
last-drink timestamp. Different later Borg RNG trajectories are therefore
expected even if no early command outcome changes. First sampled divergence
against pristine occurs at decision5000 in **Borg RNG**; the engine state/RNG
still match at that sample. This source-level explanation is separate from
claiming a measured value for every call's last-drink timestamp.

The carry did not add the repeated stair branches, the poison-rest exclusion,
or level-entry energy handling. Pristine executing the loop from the reached
save proves the distinction. Do not roll back the verified timing correction
or call this a newly introduced M4 stair algorithm. Source403 owns the separate
original-C liveness defect. Original range245 is not established as a cause:
closeness is positive here (CL9/depth1-2), with known stairs underfoot.

Initial runs recorded harness8c23ec0. After the durable documentation-only
commit `b3ae91f8a25e0a06fd3b344562d9268833639f22`, later manifests record that
head instead. **Every measurement-tool file hash is identical to8c23ec0**;
no tool or runtime source was changed by that report commit. The diagnostic
export's additive instrumentation is separately declared and qualified.

## Known state and repeated decisions

The additive diagnostic plus `borg_verbose = TRUE` reproduces the same engine
stall turn. All204 shared `STATE progress` records are byte-identical to the
original M4 run, including full engine RNG and Borg RNG. M3 likewise matches
all192 shared progress records. See `observer-qualification.json`.
Instrumentation appends output only; stripping the one added decision note
recovers the original module byte for byte. No existing condition or RNG call
is rewritten. The separate observer and binary hashes remain in its manifest.

At59870, native messages parsed by the Borg say `You feel drugged!`, `You have
learned the rune of hallucination curse`, `You are poisoned!`, then the poison
curse rune. It replaces its sandals at59873. The final known inventory contains
the removed cursed sandals. Curses causing these statuses are observed; the
Borg's prior knowledge of those curses is not assumed. See `curse-onset.log`.

Representative consecutive decisions at engine59893 (coordinates x,y):

| Decision | Depth | Position and own feature | Known up stairs | Known down stairs | Choice |
|---|---|---|---|---|---|
|12726|2|(25,12), upstairs|(25,12)|none|Power Climb, cost0, `<`|
|12727|1|(13,31), downstairs|none|(13,31)|Power Diving, cost0, `>`|
|12728|2|(96,41), upstairs|(96,41)|none|Power Climb, cost0, `<`|
|12729|1|(48,12), downstairs|none|(48,12)|Power Diving, cost0, `>`|

These are **Borg tracking arrays and its own-grid feature**, checked against the
native own tile; no unseen map staircase is treated as known. Positions change
because `birth_levels_persist=0`: the stairs lead to newly generated levels.
The flow target is the arrival staircase underfoot. Its old flow is255 on new
entry, then the new stair flow commits at0; entry goal.type is0. Munchkin mode
is1; the choice sets fleeing/fleeing_munchkin1, and observed goal.less is1.
New-level handling clears fleeing flags before the next choice. Representative
known-state/key records are in `loop-and-keys.log`.

The Borg's computed safe_place is1, scary0, HP129/131, hungry0, weak0. Light2,
torch3630, fuel11, penultimate pack slot empty: this is neither missing light
nor the full-pack selling branch. Native hallucination is14 and stays14 across
the loop; Borg hallucination and poison flags are1. The log repeatedly clears
`-more-` with space and receives successful stair messages; it is not stuck in
a prompt. Native energy gains100 per stair decision. Anti-bounce is2 and level
age1 at termination: normal new-level resets defeat a within-level bounce
counter. See the selected terminal Borg state.

The unchanged recovery routine excludes poison from its general rest branch;
its cure attempts have not queued an action here. The trace shows no recovery
command during the loop. Hallucination suppresses normal object/monster intake
and clears goals in the original update code. The observed combination leaves
stair travel running while timed ailments cannot age. This explains the liveness
failure; it is not a claim that an arbitrary replacement command would be safe.

## Zoo accounting

Keep `stalled` as a failure to advance the declared engine-turn endpoint. Record
which counter stopped and retain energy, decisions, depth/position/level changes
and a short key/note tail. Distinguish an energy-spending stair cycle from
zero-energy input retries or a process that is no longer deciding. Energy alone
must not reset an engine-progress watchdog here: that would conceal the same
world-clock freeze until the wall limit. Preserve monitor reason separately
from the native `unexpected_stop` kind and `(alive and well)` cause. None is a
cap survivor or a natural death. Stall90s is an operational bound; this is not a
proof of mathematical nontermination for every future decision/calendar event.

The original zoo data/catalogues remain unchanged.

## Loaded-state controls and limits

Frozen native save from the M4 stopped run: SHA256
`d03039590e5edfd2b65dd7f6026c00820327c4cde90a723d7d861fb99f1d3c26`.
Borg memory is not serialized; these are diagnostic loads of an ordinarily
reached game state, not faithful continuation or independent cohort lives.
Use the existing named-play native activation bridge. An initial ordinary UI
key load failed to activate Borg (0 decisions); its90s stall is excluded.
`retained-attempts.json` preserves the failed attempt and the input-save
preservation receipt. Named play redirects native autosaves into its own output.

M4 with munchkin FALSE, same frozen native save,107 additional-turn target:
**turn_cap at60033,140 actual additional engine turns**,3 decisions,0.102s,
HP131. It walks two exploration steps (59893,59903), then rests to recover at
59913; poison and hallucination clear. The boundary overshoot is expected for
named play's first subsequent command boundary. This establishes progress from
this state without a game/observer replacement, not a survival benefit.

Pristine/munchkin TRUE on that frozen save: **stalled at59893 after90.243s**
with repeated successful stairs and growing energy/decisions. This directly
demonstrates the mechanism in uncarried C, separately from the passing pristine
from-birth control. Classification is **(b), original C Borg defect reached by
a carried trajectory**. Source403 owns diagnosis/correction; consumer204 owns
reproduction and subsequent adoption. No source fix has been made here.

## Secondary seed1047/no-deeper5

Only after completing1045, ran the same M4 binary with configuration
`borg_no_deeper = 5`, fixed birth/no saves, cap100000/wall900/stall90.
Actual result: **turn_cap alive at100000, wall793.508s**, depth4/max4, CL8,
HP106. It passed90120 and reached the full cap within the declared limit.
This does **not reproduce** the zoo's900s wall timeout. The trace does show
slow advancing exploration/leave/stair-flow attempts; no second defect or
portable performance conclusion is established, and no further secondary
mechanism investigation is claimed. Different runner load/compiler/platform
can affect a wall endpoint; this is not a contradiction of the retained zoo
cell. The parameter is not asserted here to be an engine-enforced depth cap.

## Validation, raw evidence and next action

All native experiments used the shared validation lock. Python sidecars compile;
source/prose whitespace checks pass. Native log excerpts retain original
trailing spaces and terminal text; the unfiltered whitespace check reports
those data bytes, which are intentionally preserved. Intermediate PR211 hosted job128560000819 passed95
tooling tests,965/965 native tests and5/5 frontend tests; final-head hosted
checks are recorded on PR211/the issue receipt. There are no product C changes
and no adoption/fix gate is being substituted by these experiments.

Full runs/saves/binaries/build manifests/source exports remain outside Git in
the persistent ignored `artifacts/zoo1045/evidence.tar.gz`; see the curated
archive receipt for its exact SHA256, size and member readback verification.
This is local durable evidence, not a claim of a hosted raw artifact. The report,
commands, machine-readable results and selected native logs are in Git.

Investigation complete; consumer204 remains open for source403 follow-up.
Next source action: a scoped liveness correction plus unchanged poisoned/
hallucinating failure and normal-munchkin/disabled-munchkin controls. Only
then make a separate verified consumer carry through its normal native gates.
No rollback of the valid timing correction, gameplay strategy, official post,
Rust work or borg-zoo mutation was performed.
