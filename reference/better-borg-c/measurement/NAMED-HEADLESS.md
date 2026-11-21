# Named headless play verification (consumer111)

Owner request: run a named native save without a window for a bounded number of
additional engine turns, publish it beside visible saves, support explicit
backup restore in both launchers, and export natural-death dossiers.

## Implementation and limits

`./run-borg-headless.py NAME --turns N` exports a pinned observer build, uses the
real test frontend/native game loop, and loads an immutable copy of an existing
save or performs ordinary birth with a random race and class. Named-only activation invokes
the existing `borg_cmd_start` via a bridge in the disposable source export.
The ordinary product C sources are unchanged. Full game/Borg memory parity after
loading is not claimed; the game RNG and character/dungeon restore, while Borg
memory is rebuilt. Named mode disables continuous resurrection.

N means additional engine turns. Stop/save is at the first subsequent command
boundary at or after the target; a long action may overshoot. Native autosaves
are redirected into the run directory, protecting the frozen input. Normal
termination writes a final save using the existing forked saver and the wrapper
atomically publishes it after checking the old save did not change. Failures
leave the previous named save untouched. An explicitly requested backup restore
still takes effect before a failed run.

Both launchers retain five rolling pre-launch saves and share the same per-name
lock. `--from-backup` (slot1) and `--from-backup=2` (through5) explicitly restore a
snapshot and preserve the current save in a timestamped before-restore file.
After a headless death, slot1 holds the latest living checkpoint; new characters
also retain a birth backup. A dead final save is never silently resurrected.

## Actual native results

All following smoke/death runs used actual C source `127483bc4724` (29 carries),
with observer/binary/data/tool hashes in each result. They are diagnostics and
references, not an expanded survival cohort.

| Check | Actual result |
| --- | --- |
| New HeadlessSmoke, seed11, +200 | turn1 →201; cap survivor; native final saved |
| Load the published save, +200 | 201 →401; cap survivor |
| Restore its pre-run backup, +100 | 201 →301; pre-restore401 save retained |
| Final observer safe-boundary +2000 | 301 →2305; requested2000, actual2004; input hash stable |
| Native wall limit1s, requested+1000000 | wall_timeout; existing named save byte-identical |
| HeadlessDeathProtocol, seed29, +30000/wall120/stall30 | natural gallant death at20445; +20444; D0003 exported |
| Load actual death's living checkpoint at20073, +10 | 20073 →20083; alive; dead pre-restore save retained |
| Refresh pre-feature observer export | compiled successfully; previous binary/manifest retained |

The death export test occurred before automatic recovery-slot seeding was added.
The living checkpoint was explicitly seeded from its real recent.save to test
restore/continuation; the final automatic death-to-slot1 path is also covered
by an isolated wrapper integration test. No GUI windows were opened.

The initial stdin-pipe activation prototype failed before the first command
boundary. That failed attempt and its terminal evidence remain archived, not
counted as a death/survivor. It was replaced by named-only native activation.

## Ordinary-measurement negative controls

Re-ran the unchanged frozen v1 seeds11/29/47, cap20000/wall120/stall30, with the
extended observer in ordinary measurement mode. All3 were cap survivors and
**each complete trace was byte-identical** to the previously measured29-carry
light-modifier references. Their trace SHA256 values are recorded under
headless/measurement-controls/result.json in the archive. Ordinary measurement
retains its original keyboard activation and exact world-tick cap; named play's
actuation/command-boundary/final-publication behavior is opt-in only.

Local tooling:41 tests passed, including atomic-publication failure, changed
outside writer, rolling backup/restore, duplicate-name exclusion, different-name
locks, wrapper death/cap/timeout classification, named map export and qualified
family aggregation. Visible backup restore was verified against a disposable
executable, with no X11 instance. Hosted gate receipts are on the PR/issue.

## Death protocol

[D0003](../deaths/D0003-named-headless/report.md) retains its actual29-carry source
and binary, not a later checkout's version. All native Borg notes and all23
noncheckpoint state records (with sampled game/Borg RNG) match D0002's recorded
six-carry life. A verified assessment groups it with D0002's encounter family:
three dossiers, **two starting-condition families**, nine death process attempts.
Names/PLAY metadata/checkpoint ordering differ; full raw traces are not asserted
byte-identical. This is not a new independent life, survival improvement or
proved bug. Current-build pre-death windows and causal diagnosis remain pending
on investigation34. Automatic dossiers begin unassessed and require follow-up.

## Persistent evidence and recovery

Primary-checkout archive: `/home/diego/projects/angband/better-borg-c/artifacts/named-headless-validation/20261107T001901Z/evidence.tar.gz`

- Size: 155,925,417 bytes; 238 file members.
- SHA256: `88390a1efa41fd6f6959c4c4844010a3ee8e06c493ce3a89ed69f9317c604b84`.
- Every member was read back and matched against adjacent manifest.json.

It preserves every named attempt/input/result/map/living/final save, baseline
controls, old/new observer build archives/binaries/manifests and refresh receipts,
and the implementation snapshot. Disposable .o/.a/CMake cache trees are excluded.
Full raw data stays outside Git. Dossier logs/maps and actual identity are the
selected native evidence requested by the owner.

For historical recovery, verify receipt/manifest, extract into a fresh evidence
directory, reconstruct source from each source.tar, and overlay that build's
archived instrumented source files. Build commands/compiler/platform are in
build.json; match data/tool/option/seed/calendar identity. The archived binary
is retained for the exact original workstation ABI. Rebase raw run output paths
when reproducing; never overwrite the old evidence. Public source/committed
tools and archived source/observer bytes avoid a dependency on /tmp. A normal
new root-script invocation uses its current committed source, not the historical
D0003 build.

Fresh automated births use the native random race/class menu choices and a
random seed when --seed is omitted. Use --seed N for reproducibility and
--birth fixed for a Human Warrior control. Loaded saves keep their character.
The actual native CHARACTER, seed and input are retained in the run metadata.
Frozen ordinary measurement cohorts keep their existing birth profile.
