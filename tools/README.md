<!-- Cemetery ownership: normal death dossiers now live in diegoami/borg-cemetery. -->

# tools/

*Ours. The tools copied from borg-rs and better-borg-c in T1.6 (GUI recording,
replay viewer, human takeover, native recording, the death workflow and the
root launchers) were dropped in TE.6 (#76; docs/SPEC.md §8, §11): they were
one toolkit built on borg-rs's recording format and its own binary. What they
gave is covered by determinism and the Borg's own logs.*

| Tool | What |
| --- | --- |
| `export-death.py` | Run by `harness/run.sh` after every run: a natural death gets a dossier in borg-cemetery (RULES 12; explicit temporary destinations supported), with the trace tail and the parameters |
| `death_registry.py` | Forwards registry maintenance to cemetery ownership; explicit historical/temporary registries remain supported (`python3 tools/death_registry.py`) |
| `rerun-death.py` | Replays a birth dossier with its recorded controller and parameter provenance; verifies executable, configuration, endpoint and all key/state/message streams in `OUT/reproduction.json`. Runs `build/rust`, clears ambient policy and refuses loaded-save reconstruction from a birth dossier alone; supports `--checkpoint-every N` |
| `reconcile-deaths.py` | CI: every death of a CI run is in the registry; with `--runs DIR …`, every natural death in those run outputs has a dossier of the same encounter family and key stream (E review finding 3, M2 review finding 7). The comparison baseline is `--registry` (default the retained `docs/deaths/`) |
| `check-perception.py` | T2.5: over runs of a `test-perception` build, fails on any think whose observation or world disagrees with the engine (map memory, visible monsters, gear list, pack count) or whose status is in no `state.log` line of its turn; counts what the Borg does not see by design (off the map panel). Every run of a suite directory needs a truth log with one line per Borg decision (M2 review, finding 4) |
| `depth-stream.py` | Run by `harness/run.sh`: `depths.json`, the first turn at each depth |
| `import-deaths.py` | Qualifies immutable native evidence packages for historical/CI integration; preserves original records and all payload bytes; `--check` verifies the retained archive (docs/DOSSIER-IMPORT.md) |
| `cemetery_tools.py` | The shared forwarder of the four death-tool pointers above: checks `death-tools-pin.txt` and runs borg-cemetery's implementation (below) |
| `check-no-time.py` | Fails a run's decision trace on loops hidden at one game turn (level changes or actions past the profile's budget; the reborn core's `trace.jsonl`) |
| `calibrate.py` | T4.7 (#124): the reborn core's predicted damage against damage taken, per think window (docs/M4-CALIBRATION.md) |
| `progress.py` | How much a suite's runs played, not only whether they lived: frozen windows and progress per run (#201) |
| `shopping-audit.py` | Read only: what a run's Borg bought, used, stored, sold and left unspent, from its `messages.log` and `state.log` |
| `try-errands-plan.py` | Runs an errands plan outside the measured harness, for bounded diagnostics (docs/GAME-PLAN-LANGUAGE.md) |
| `tests/` | Unit tests of the above (`python3 -m unittest discover -s tools/tests`) |

To watch the Borg play: `./watch-borg.sh` at the repository's root.

A historical harness dossier without `borg_core` explicitly selects reborn.
Reborn writes observation/world/knowledge logs; borg-rs instead writes native
parameter and decision-trace logs. Ambient `BORG_*`, `ANGBAND_*` and `BORGH_*`
settings are cleared except the explicit toolkit lookup `BORG_CEMETERY_DIR`.
Only environment-origin parameters are restored as environment overrides;
file-origin settings must reproduce the original borg.txt bytes. The receipt
distinguishes `reproduced`, `mismatch`, `failed` and `diagnostic`; `--any-build`
never sets `reproduction_verified` true. Missing evidence fails verification.

## Investigating a death

1. Check out the dossier's `git_rev` (in `record.json`) if the current
   build is another source, and run `BORG_IMPL=rust harness/build.sh`.
2. `tools/rerun-death.py /path/to/borg-cemetery/deaths/DXXXX-... OUT --checkpoint-every 500`.
   Use the actual dossier directory: new exports default to
   [borg-cemetery](https://github.com/diegoami/borg-cemetery)'s `deaths/`,
   or the explicit `BORG_DEATH_RECORDS_DIR` override. Historical
   `docs/deaths/DXXXX-...` dossiers remain usable at their existing paths.
3. Read the rerun's trace and observations up to the death, or load a
   checkpoint before it.
4. Write findings in the cemetery-owned dossier and regenerate its index.
   Qualified historical packages are immutable evidence snapshots; keep new
   analysis in the cemetery rather than editing the imported package.

## Cemetery-owned death tools

The export, registry, source-checked rerun and reconciliation implementations now
live in [borg-cemetery](https://github.com/diegoami/borg-cemetery/tree/main/scripts/native),
with their provenance and original tests. The four old command paths are thin
API-1 pointers; there is no second implementation to maintain here.
Before executing any native tool, the forwarder verifies every file in
`scripts/native` against the immutable commit in `death-tools-pin.txt`. This
applies to sibling and `BORG_CEMETERY_DIR` checkouts as well as CI. Missing,
changed, symlinked or additional source files fail before tool code can run;
Python bytecode caches are ignored. The selected checkout must contain the
pinned Git object (fetch it if necessary). Later documentation/evidence-only
heads are allowed when their native subtree bytes match the pin. Merely
reporting API 1 is insufficient: an old API-1 exporter can lack the synthetic
guard. An explicit checkout mismatch is an error, never a fallback to executing
unsafe code or silently exporting a synthetic death.

Which checkout runs (#902): `BORG_CEMETERY_DIR` when set; a mismatch there is
an error, never a fallback. Without it, the first of these whose native tools
match the pin: the borg-cemetery beside the primary checkout (linked worktrees
find the sibling of their primary checkout), then `vendor/borg-cemetery` (the
worktree's, then the primary checkout's). `scripts/setup.sh` keeps the primary
checkout's `vendor/borg-cemetery` at `death-tools-pin.txt`, a full checkout of
about 1.2 GB shared by every worktree, so a sibling that tracks the Cemetery's
`main` past the pin no longer stops a run: the tools print one line naming the
checkout they use and why. With none matching, the error names `setup.sh`.

New normal exports go to the selected checkout's `deaths/`, and its own
registration hook builds readable pages/catalogue without overwriting historical
indexes. An export into `vendor/borg-cemetery` is a working copy: to keep a
death, add it to a real Cemetery branch (or point `BORG_DEATH_RECORDS_DIR`, the
explicit destination override, at one).

This repository still owns the harness, build/source identity and observations.
Its historical `docs/deaths/` is preserved for existing evidence and CI baseline
checks. CI uses the pinned external tools and its existing temporary-registry
override/reconciliation guard; no assertion is weakened. To deliberately maintain
the historical registry, supply its path explicitly rather than making it the
default output again. The reborn strategy remains model-free; the diagnosis
pilot #153 and tactical counterfactual plan #212 are not implemented by this move.
