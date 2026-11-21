# Deep paired observations after the potion-timing carry

Three paired ordinary Human Warrior starts produced two deaths/one cap survivor
on the five-carry build and one death/two cap survivors on the six-carry build.
No stalls, wall timeouts, crashes, completions or harness failures occurred.
This small observation is not established population survival improvement.
Changed random consumption leads to different encounters/progression; it does
not isolate a life-saving potion decision in the original fatal encounter.

Issue33 declared this batch before outcomes: seeds11/29/47, absolute engine-turn
cap400000, wall180s, stall30s, ordinary default birth/input/options, no intervention.
The frozen v1 cap20000 cohort remains unchanged and reported separately in
[POTION-TIMING.md](POTION-TIMING.md). Its seed47 level4→2 result is retained.
Deep v2 uses the same three starting families, not additional independent lives.
No seed/cap replacement or selective enlargement was used to obtain a good result.

Actual five-carry source3638b690525cd8a381cb106b4c5ee40652babde2 reuses its
qualified native observed binary; six-carry is the rebuilt actual merge
45eac63bd9f7cabe3c4ef1607703324ca8dae0fc. Both retain the official game base
e473720ed88a0f0c7ca66815cef3000eb4a38bc3. Exact source/build/binary/data/tool/
calendar/RNG/command identity and all individual summaries are in
[machine results](deep-impact-v2-results.json); the prospective configuration is
[cohort.json](deep-impact-v2-cohort.json). Observer instrumentation is unchanged,
reads state and uses forked saves; no decisions/RNG/game rules changed by logging.
No cheat-death, hidden-information input or favorable intervention was used.

| Build | Seed | Outcome/cause | Engine turn | Max depth | Level | Borg calls | Energy spent | Wall seconds |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Five carries |11|death: stegocentipede|174862|12|9|15366|1774459|56.893|
| Six carries |11|cap survivor|400000|9|15|40303|4042637|112.971|
| Five carries |29|death: gallant|20445|3|3|2066|208234|6.635|
| Six carries |29|death: gallant|20445|3|3|2066|208234|6.640|
| Five carries |47|cap survivor|400000|6|14|40495|4032041|106.655|
| Six carries |47|cap survivor|400000|4|16|35928|4032512|95.422|

Energy, engine turns, Borg-think calls and wall time are distinct measurements.
All three starting saves match byte-for-byte across versions. Subsequent full
traces differ, including private Borg RNG and later choices. A lower wall time
across different trajectories is not a measured execution speedup. Caps are
censored observations, not wins or promises of eventual survival. Higher level
can coexist with shallower maximum depth; there is no single proven fitness win.

## Deaths, replay and diagnoses

The five-carry seed11 death has exactly the full trace/start save/final RNG of
[D0001](../deaths/D0001-stegocentipede/report.md), now appended as a versioned
reference. The original D0001 source337e50eb is immutable. The fixed seed11 full
repeat survives400000 with byte-identical trace/start save; that replay is a
control, not another independent survivor. No assertion that healing rescued
the same stegocentipede encounter is justified.

[D0002](../deaths/D0002-gallant/report.md) records the seed29 gallant death at
20445, both versions and pristine reference, exact binary/build identities,
native logs/inventory/maps, assessment and issue34 actions. The unchanged fixed
repeat matches the entire trace/start save/terminal RNG. Four original-start
windows T−20/15/10/5 match prior observations; actual HP31/22/12/3. A recent
living save20073 loaded/advanced20074 by an ordinary wait, without Borg, as a
separate diagnostic. No exact midrun Borg-memory continuation or reloadable
postdeath dungeon is claimed.

Five-carry/pristine seed29 full traces match each other; fixed notes/private
RNG differ but final game RNG, death turn/cause and gameplay state agree. This
behavior predates our carries; a defect is not established. Twelve CLW potions
remained, but existing healing guards reject them at recorded danger507/HP3.
No claim that a drink was eligible or would save it. Investigation34 examines
actual call order/fleeing and earlier Phase Door exhaustion before any policy
change. Original-C defects, if proved, route to source207 before fix/adoption.

All three fixed v1 observation prefixes were repeated by the deep runs. Fixed
seed47's complete400000-turn trajectory was not repeated; only its first20000
observations are qualified. [Replay receipts](deep-impact-v2-repeatability.json)
state that limitation. Runtime calendar/timezone is recorded for these runs;
native seasonal monster rules still limit cross-calendar reproduction.

## Reproduction and evidence

From a clone containing these pins, prepare each product independently:

```sh
flock /tmp/angband-validation.lock python3 tools/measure-native.py prepare --ref 3638b690525cd8a381cb106b4c5ee40652babde2 --observe --output artifacts/deep-five
flock /tmp/angband-validation.lock python3 tools/measure-native.py prepare --ref 45eac63bd9f7cabe3c4ef1607703324ca8dae0fc --observe --output artifacts/deep-six
flock /tmp/angband-validation.lock python3 tools/measure-native.py run --build artifacts/deep-five --output artifacts/deep-five-11 --seed 11 --cap 400000 --timeout 180 --stall-timeout 30
flock /tmp/angband-validation.lock python3 tools/measure-native.py run --build artifacts/deep-six --output artifacts/deep-six-11 --seed 11 --cap 400000 --timeout 180 --stall-timeout 30
```

Repeat29/47 into new directories without replacing failed outcomes. Match the
recorded toolchain/data/options/calendar; source pins do not promise identical
binary bytes across compilers. Qualified existing binaries were reused for the
actual batch, not silently relabelled as new builds. All jobs were serial under
the shared validation lock and watched. Raw traces/saves/maps/inputs and actual
binaries remain ignored in persistent artifacts/deep-impact-v2, with a verified
archive/manifest/recovery receipt on issue33. Curated death logs/snapshots and
per-run machine summaries are committed at the owner's request.

Next product work remains verified independent bugs before more exploratory
death runs. No new tactics, automatic tuning or dashboard was implemented.
