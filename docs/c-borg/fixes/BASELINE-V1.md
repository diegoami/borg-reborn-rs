# Frozen native baseline v1 — 2026-11-06

The validated baseline and paired first carry use independent **n=3**:
Human Warrior/default point buy, seeds11/29/47, absolute engine turn20000,
wall120seconds, no observed turn progress30seconds. Baseline repeats are
reproducibility controls, not six independent samples. The cohort was committed
at a78d21889e3826cac65f7f4f219393da1af5304b before any full-cap gameplay results.

Every accepted run survived to the cap; there were **zero deaths, successful
completions, stalls, crashes or harness failures** in this cohort. Thus there are
no observed death causes to rank. Baseline/fixed starting saves and complete
native decision/state traces are byte-identical for each seed. This shallow
cohort reaches maximum depth3; emergency swaps require prior depth50. It does
not measure a survival benefit for the correction. No speedup is inferred from
wall-time differences between sequential runs.

| Seed | Baseline wall seconds, repetitions1/2 | Fixed wall seconds | Max depth | Level | HP | Engine turn | Energy | Borg-think calls |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 11 | 15.268 /15.673 | 12.304 | 1 | 3 | 28 | 20000 | 207434 | 1994 |
| 29 | 11.929 /11.428 | 9.767 | 3 | 3 | 49 | 20000 | 203758 | 2019 |
| 47 | 8.779 /8.149 | 8.154 | 2 | 4 | 63 | 20000 | 207052 | 1860 |

Mean maximum depth2; mean character level3.33. These are descriptive early-game
statistics, without a population survival estimate or confidence claim. All
three independent starts are censored at the turn cap. World turns, spent energy,
Borg calls and wall time are different measures; birth starts at turn1, so the
elapsed world-turn interval is19999.

Raw machine-readable metrics for **every** accepted run, excluded diagnostic
attempts, source/build identities and save/trace/result hashes are published in
[measurement issue3's immutable receipt](https://github.com/diegoami/better-borg-c/issues/3#issuecomment-6682135753).
Full generated evidence remains outside Git in the persistent primary-checkout
archive described by the final checkpoint. No source fixture or recovery depends
on workstation `/tmp`. Reproduction commands are in README.md and cohort-v1.json.

## Exact native identities

Pristine source: `e473720ed88a0f0c7ca66815cef3000eb4a38bc3`.
Unmodified Git archive SHA256:
`6bdd13921470ea541f240456c5c98fc972970aa70d22478c040901bdcc589a5d`.
The pristine archive and uninstrumented binary are retained separately, read-only.
The initial pristine binary's native version helper discovered the enclosing
bootstrap checkout; its version text is not its source identity. Its build
manifest/archive hash identifies the exact unmodified C source. Later disposable
snapshot builds explicitly stamp the source commit and observer and prevent Git
searching above the export. There is no gameplay delta in the pristine build.

Accepted instrumented baseline binary SHA256:
`914289b331e3ee16f62bd0afc6e8b4e26a2c176cc1f8a7248cfc5a93afd1ac3d`;
observer tooling `7bdc4f16cf66cc5875d6daef1ffdfae96273070d`.
Accepted fixed source/build export:
`33b29f0a4af62749d3a340c2c91c0e59a642da21`, with observer tooling
`cc0f80236af861e07219afe85033a1bcfd7e0ee5`;
binary SHA256 `66f4f13cc3471ddb6df1ead95e3cf2bfeef96c6b8e3a43046b80f9a6b008a4c9`.
The only runtime source delta is the reviewed emergency-swap comparison/comment.
Compiler: GCC15.2.0, Linux x86_64, CMake Debug, Borg/X11/test frontend. Binary
hashes identify these actual builds, not promised cross-toolchain reproducibility.
Every run retains data/config/options hashes and commands in result.json.

## Failure analysis and observer qualification

The first observer lacked Borg declarations; the next snapshot version stamp
contained a newline that broke generated compiler flags. Both compile failures
remain retained. Initial autoplay attempts had two data-path harness failures
and four startup stalls, including a verbose terminal diagnostic. Native UI birth
and the unsupported-Borg warning require explicit acknowledgements; the final
recorded input starts Borg and completes unattended. Startup stalls have zero
Borg decisions and are harness startup diagnoses, not proven gameplay loops.

A2000-turn no-saves control initially matched. At the full20000-turn cap it
exposed a later difference around turn9511. Native `wr_dungeon()` compacts the
monster list when saving. Saving in the live observer can therefore change
monster order and later random assignments. All six original baseline and three
original fixed runs are retained as **observer-affected diagnostics** and excluded
from the accepted statistics. They were not silently replaced or relabelled.

Evidence saves now execute in a POSIX fork child with copy-on-write memory. The
parent does not execute saver mutations. The complete seed11 cap20000 trace
matches the no-save trace exactly after removing checkpoint-only records; final
RNG, energy, decisions and character state match. Every accepted baseline seed
also repeats with identical traces. No seed, cap, timeout or assertion changed
when the same frozen cohort was repeated after observer repair.

Birth save loading reproduced initial state and the2000-turn pilot. Ordinary UI
retirement separately tested the real native dead/end capture path (`Retiring`),
with a retained living checkpoint; that diagnostic is excluded from performance
statistics and is not a combat-death observation. Natural combat death/crash
capture was not observed in the small cohort.14 tooling controls test numeric
native summaries, missing results, signals, nonzero exits, real watchdog timeout
and stall, and preflight failure persistence. A checkpoint lacks complete Borg
memory; exact mid-run continuation is not claimed. Native post-death saves omit
dungeon records and must not be described as reloadable living encounters.
