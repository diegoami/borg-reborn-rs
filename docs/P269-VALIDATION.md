# Strict configured errands (#269)

The core-selector portion was delivered in #280. This task closes the
remaining configured-errand fallback: an unknown command must not quietly
run the core without the requested errand.

## Behavior and compatibility

`BORG_ERRAND` takes precedence over `borg_reborn_errand` in borg.txt.
Absent, empty-string and exact `none` selections retain their established
no-errand meaning. Valid shell command lines are passed unchanged to the
existing parser/executor. Invalid commands (including an invalid suffix
following a valid command), whitespace-only/empty command lists, and
non-UTF-8 environment selectors fail with source/value diagnostics and
exit status 2 before the vendored core starts. Interactive parsing is
unchanged. Original-core runs ignore this vendored-core setting.

This intentionally changes invalid-input behavior, not valid/default
policy. The first implementation rejected an empty string; the existing
shopping test caught that compatibility regression. The final version
preserves the empty alias without modifying that test.

## Exact builds

First candidate: commit `014e64a4bfac878db35781af24bcf8bca5cf5078`, source
`77f040ffc32cd317b917064c071d07c376d006276e6664f1b8b866e502672d46`,
binary `dedace262eee600aaf3227ae1c85c99e7bcc7349ed07c3fb44eeb9d94fe6dfa0`.
Engine checks: 45 tests, one error in the unchanged shopping test's empty
errand fixture and one environment skip. This candidate is not accepted.
Its eight full-suite natural deaths are retained as D2399–D2406, with the
actual candidate BUILD_ID. It is not relabelled as the corrected build.

Corrected implementation: commit `c259f69c44f8873a21fb65e523235a13a21df4e9`,
source `cb48d94396894881d4ae001b2a49b994e86da5f675e8f2444a7c9979c85e5b90`,
binary `44676b1ae367ee41be371791f13c6db7e8fbf802292a7051dc44db228062695f`.
Upstream engine and dependency pins are unchanged from the synchronized
base; the death tools are read from the main-approved cemetery pin.
No external repository is written or migrated by this task.

## Validation status

Required local Rust checks pass (formatting, Clippy with warnings as errors,
154 + 4 + 5 original crate tests, release core 138, one ignored).
Script tests 19 and tool tests 78 pass. All 45 engine tests pass with one
local binding-regeneration environment skip. The actual pinned zoo/native
profile tests cover invalid file settings, invalid environment command
lists, valid/none/empty overrides of an invalid file value, and original
core isolation. The unchanged shopping fixtures also pass.

Corrected original-core smoke matches its reference 3/3, and its separate
death fixture completes. D2407–D2408 preserve those two natural deaths.
The mixed-source temporary registry reconciles against both actual run
manifests and the historical dossier archive: ten deaths, none missing.
The explicit historical/CI registry retains this evidence for the existing
CI reconciliation gate; canonical migration/publishing is separate.

Corrected full-suite default keys/messages/states match pinned v12 in
45/45 runs, with 295,464 contiguous, classified named trace rows. Trace
on/off preserves all three play logs (351 trace rows when enabled).
D2409–D2416 retain the corrected full suite's eight natural deaths. Final
reconciliation accounts for all eight; the earlier mixed-source registry
also reconciles all ten candidate/smoke/fixture deaths. There are 18 new
source-qualified dossiers in total, with no failed run or candidate dropped.
Vendoring check passes. Required GitHub CI remains the integration gate.

No population experiment was launched; no improvement claim is made.
This is post-#260 work: it integrates into track-p after CI and remains
subject to a separate owner milestone/promotion decision.
