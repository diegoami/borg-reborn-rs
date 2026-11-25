# Validation command receipts (#374)

A validation receipt records how a result was obtained, including a failed
attempt. New gameplay evidence and its receipts belong in Borg Cemetery;
this source repository keeps the command contract and reconstruction recipes.
Do not edit an old report, its hashes or a qualified package to add information
that was not captured at the time.

For future captures, retain:

- Exact repository revisions and clean/dirty source/build identities for each
  role, executable hash, compiler versions and any source diff.
- The actual command as an argv array, checkout role and relative working
  directory, explicit policy/configuration environment and selected profile.
  Record execution-affecting ambient inputs or say which were not captured;
  do not dump credentials or the whole host environment into a receipt.
- The exact suite bytes, their SHA-256, revision/path and resolved per-start
  flags. A suite name alone is insufficient across repositories or revisions.
- Exit status, DONE/FAILED state, process receipts and stdout/stderr for every
  attempt. Keep failed setup/export attempts separate from completed games.
- Effective configuration, summaries and per-side stream hashes. Keep strict
  whole-summary comparison results separate from selected-stream equality.
- Canonical evidence links, input/output hashes and attempt lineage. A repeat,
  replay or paired control is not another independent life.

Capture argv/environment before execution and exit status afterwards. Label
inferred details as reconstruction, never as contemporaneous capture. Use
portable roles such as `receiving`, `donor`, `cemetery_tools` and `output`;
keep original absolute paths in the original evidence rather than rewriting it.

## Reconstructed M13 full-suite commands

[p341-full.json](../harness/recipes/p341-full.json) is a **verified historical
reconstruction**, not recovered shell history. It complements the immutable
[P341 receipt](validation/P341-FRESH-DONOR-VALIDATION.json) at reviewed source
`cf687bb24de8e0715854ca1b7a0f827b7e71be82`. That receipt already contains all
45 pairs of key/message/state hashes; no missing-hash repair is needed.

The original raw checkout paths are not available in this task's workspace.
The reconstruction uses pinned Git objects, retained build identities and all
45 per-run configs/summaries. Original argv spelling, ambient environment and
parallelism were not captured there. `JOBS=2` and the output locations below
are explicit reconstruction choices. The historical compiler versions and
binary hashes are references, not a promise of a bit-identical new build.

Use separate clean checkouts and a new, private output root. These are
copyable reproduction instructions; this task does not execute them or start
new games. Set the four paths for your machine:

```bash
set -euo pipefail
RECEIVING=/absolute/path/to/receiving-checkout
DONOR=/absolute/path/to/donor-checkout
CEMETERY=/absolute/path/to/historical-cemetery-tool-checkout
OUTPUT=/absolute/path/to/new-private-output

test "$(git -C "$RECEIVING" rev-parse HEAD)" = 2f74e1c5d57ce2b41ded1b5b694174b91b51f265
test "$(git -C "$DONOR" rev-parse HEAD)" = 1443c2d1b97d110ffbf5ae420d6c6b97ac15f7ea
test "$(git -C "$CEMETERY" rev-parse HEAD)" = 33c375527a8e443cdfcda19e864e4561f0c1c9ef
test -z "$(git -C "$RECEIVING" status --porcelain)"
test -z "$(git -C "$DONOR" status --porcelain)"
test -z "$(git -C "$CEMETERY" status --porcelain)"
test ! -e "$OUTPUT"
mkdir -p "$OUTPUT"
SUITE="$RECEIVING/harness/suites/full"
printf '%s  %s\n' 7c81c28c03be3e6c188cb555fa8c917402bae0819d0791a1a810e031214d2eed "$SUITE" | sha256sum -c -

# Clear inherited policy, input and verification overrides in each subshell.
# Preserve the configured host toolchain; this is not a hermetic build.
clear_validation_env() {
    local receipt_env_name
    for receipt_env_name in "${!BORG_@}" "${!ANGBAND_@}"; do
        if [ -n "$receipt_env_name" ]; then unset "$receipt_env_name"; fi
    done
    unset ALLOW_STALE_BUILD ALLOW_UNKNOWN_IMPL BORGH_DATA_ROOT SAVE_ROOT
    unset CHECKPOINT_EVERY RUN_TIMEOUT_SECONDS STALL_TIMEOUT_SECONDS
}
```

Both implementations must use that **same receiving suite file**, containing
45 menu-letter starts, nine classes and 100,000 turns per start. The donor's
own `full` has SHA-256
`949b423fc94533b52d92b66a8c72843c0108465e21fdb8a82e6d332709560806`
and lacks `menu`; substituting it changes birth selection. Donor build and
suite scripts also default to C, so both require explicit `BORG_IMPL=rust`.

```bash
(
    clear_validation_env
    cd "$DONOR"
    BORG_IMPL=rust BORG_RECORDING=0 \
        flock /tmp/angband-validation.lock harness/build.sh
)
(
    clear_validation_env
    cd "$RECEIVING"
    BORG_IMPL=rust BORG_TEST_FEATURES= BORG_X11=OFF \
        flock /tmp/angband-validation.lock harness/build.sh
)
(
    clear_validation_env
    cd "$DONOR"
    BORG_IMPL=rust JOBS=2 ANGBAND_BIN="$DONOR/build/rust/game/angband" \
        CHECKPOINT_EVERY=0 RUN_TIMEOUT_SECONDS=0 STALL_TIMEOUT_SECONDS=0 \
        flock /tmp/angband-validation.lock \
        harness/suite.sh "$SUITE" "$OUTPUT/donor-full"
)
(
    clear_validation_env
    cd "$RECEIVING"
    BORG_IMPL=rust JOBS=2 ANGBAND_BIN="$RECEIVING/build/rust/game/angband" \
        BORG_CORE=borg-rs BORG_CORE_OVERRIDES=off BORG_ERRAND=none \
        BORG_STOPSCARED=off \
        BORG_PROFILE="$RECEIVING/harness/profiles/reborn-default.txt" \
        BORG_CEMETERY_DIR="$CEMETERY" \
        BORG_DEATH_RECORDS_DIR="$OUTPUT/receiving-deaths" \
        CHECKPOINT_EVERY=0 RUN_TIMEOUT_SECONDS=0 STALL_TIMEOUT_SECONDS=0 \
        flock /tmp/angband-validation.lock \
        harness/suite.sh "$SUITE" "$OUTPUT/receiving-full"
)
```

The receiving selectors reproduce the retained config fields and select
`borg-core/borg.txt`, with its pristine donor policy. The profile SHA-256 is
`372785a3dfab6020d2e0881a0be7a8db42d70c39ad6db40f9599d3a6d1c2fc91`.
The pinned donor runner does not export deaths automatically. Any reproduction
must also follow the Cemetery-owned export/retention process for donor deaths;
that postprocessing is outside this full-suite command reconstruction. The
historical [dossier map](validation/P341-DOSSIER-MAP.json) preserves its actual 25 qualified
packages, exporter pin and original record hashes. No new death or reconciliation
is claimed by checking this recipe.

## Verification and limits

The reconstruction checked the suite blob (`ab5706ef24520359dc9d3e963704d98e1c2ac6e2`),
all 45 start identities/flags and each role's effective borg.txt/config hash
against the retained evidence. The pinned identity algorithms applied to clean
Git trees reproduce receiving source hash `87717140…` and donor `d07a8547…`,
exactly as retained. These are metadata checks, not new compiler or game runs.

Donor-side source inspection at `1443c2d1` also confirms the external suite
path branch in `harness/suite.sh`, C defaults in both scripts, and
`BORG_RECORDING` selecting the CMake recording flag in `harness/build.sh`.
The recipe records SHA-256 identities for both inspected scripts so another
reader can check those same objects. These were local source checks at the
#579 task review, whose reviewer did not inspect the donor repository. The
later [P5 milestone review](https://github.com/diegoami/borg-reborn-rs/pull/626#issuecomment-6998991184)
independently checked those donor scripts, the donor full-suite hash and the
M13 tag. Neither review started a new build or game; the reconstruction's
historical-capture limits remain unchanged.

Preserve strict comparison failure. Use the receiving checker's pinned version;
do not drop fields, permit incomplete suites or turn its failure into success.
This block is expected to exit 1 for the historical comparison; its status is
in `strict-summary.exit`. With `set -e`, a combined script stops there as intended.


```bash
(
    cd "$RECEIVING"
    set +e
    harness/diff --suite "$SUITE" --expect-impl rust \
        "$OUTPUT/receiving-full" "$OUTPUT/donor-full" \
        > "$OUTPUT/strict-summary.txt" 2> "$OUTPUT/strict-summary.stderr"
    receipt_status=$?
    printf '%s\n' "$receipt_status" > "$OUTPUT/strict-summary.exit"
    exit "$receipt_status"
)
```

The historical comparison has 45/45 equal play-stream triples and **0/45
identical complete summaries**. Differences include telemetry and schema fields;
the original failed-comparison output hash is
`e06e85e8db5ae89c8aa7409321b38fc4097334fcd2203407c0c09a87802f5e7d`.
A newly generated output can contain different paths and is not relabelled as
that historical file. Original evidence, per-side hashes, operational failures,
opt-in pairs and replay counts remain untouched. This recipe reconstructs only
the two full suites: it does not recover opt-in/probe/replay commands or claim
post-#355 donor parity, urgent-purchase coverage or population improvement.
