# Strict configured errand budgets (#296)

For a configured vendored-core command line, reject unreadable, empty,
negative, overflowing and non-UTF-8 BORG_ERRAND_BUDGET values with their
source/value and exit status 2. Preserve nonnegative i64 integers, with
missing, 0 and exact none meaning unbounded. Original-core and
inactive-errand contexts ignore this setting as before. No file-budget
setting or interactive-command behavior is added.

Final combined source:
`aed27276f412db648994714eecef92c16a1fe95cca523c57aaff59f8a4a55daf`. Actual
build git revision: `ab0c0bb583620e737188dd2ec3a1e7e740b27c04`. Binary:
`1a8b398f3a967be4c088211b859edb353227d87b516235c0d615b41b88c95232`. Later
upstream dossier/report merges do not change build inputs. This build
includes main's recorder/loop-levels work and #295's strict fear validation;
their existing behavior is preserved.

Required Rust checks pass: formatting, Clippy -D warnings, 156+4+5 original
and 143 core tests (one ignored). Script 19/tool 78 tests pass. All 55
engine tests pass with one local binding-regeneration skip, including 13
actual native/pinned-zoo contract tests. They exercise invalid values/bytes,
valid zero/none/max budgets, finite stopping and ignored inactive contexts.

The initial finite-budget test used the wrong adapter outcome name. The
native runner correctly stopped at turn 11; the pinned zoo classifies an
intentional borg_stopped result as stalled. The corrected test asserts both
adapter classification and runner/native stop reason and verifies turns
below the cap. No product/adapter change or weakened bound was used. Initial
Rust compilation also hit ENOSPC; unchanged-source retries pass following
regenerable-cache cleanup. These failures remain in the record, not reported
as deaths or successful first attempts.

Final default full 45/45 keys/messages/states match pinned v12, with 295,464
contiguous classified trace rows. Original smoke 3/3 and its death fixture
pass. Six paired valid-budget cases (unset/0/none/1/12/i64::MAX) preserve
keys/messages/states against #295; their 12 control runs have no natural
death. Native tests prove a one-turn budget ends before the cap.

D2466–D2475 preserve the final ten natural deaths against the actual build.
All reconcile, with none missing or unexported. Explicit historical/CI
exports retain the CI guard; no canonical repository write or migration.
Upstream source-qualified dossiers and maps are preserved unchanged.

This task is excluded from #292's earlier owner authorization. Required CI
gates track-p integration; a separate owner review/promotion follows. No
population experiment or improvement claim.
