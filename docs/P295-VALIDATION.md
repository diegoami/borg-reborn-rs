# Strict configured fear thresholds (#295)

Reject invalid/non-UTF-8 BORG_STOPSCARED or borg_reborn_stopscared values
with source/value diagnostics and exit2. Preserve the existing parser's
valid off/false/no, flee/true/on/yes and numeric1–1000 forms. Environment
values win over the file; absent/empty/exact none retain the core default
and suppress a file override. Explicit off disables the threshold. The
original core ignores this setting. Interactive parsing is unchanged.
ERRANDS.md also folds the paragraph noted in review292 F4.

Pre-loop source8f45f46bc108d9a84e32ffe60b123d635edb2992fd908adf52e37c447b0f517d
was built ate159298f. It passed full45/45 v12 play parity, original smoke
and its fixture. After concurrent main recorder/loop integration, revalidate
at source27d7f45314b420d08992cbd0c5c92613dc3652a07e2a2420cc87ec99d11e140c,
compiled atd265e9647562fa399e8359eded6c35b27d375b0a with binary
84ba9c855a6db748497fd1cab672de049f63ee000e9fefd96eedac1648e330b1.
Actual build identities are retained, not relabelled across integrations.

Final required Rust checks pass: formatting, Clippy -D warnings,
155+4+5 original tests and143 core tests (one ignored). Script19/tool78
checks pass. All53 engine tests pass with one local binding-regeneration
environment skip, including11 actual native/pinned-zoo contract tests.
They cover invalid file/env inputs, precedence and empty/none aliases,
boolean aliases, non-UTF-8 input and original-core isolation.

Final default full45/45 keys/messages/states match pinned v12 with295,464
contiguous classified trace rows; original smoke3/3 and death fixture pass.
Three paired valid-setting cases (off/flee/75) match the synchronized base
for keys/messages/states. Those controls caused no natural death.

Pre-loop deaths were initially allocated IDs later needed upstream;
P295-DOSSIER-MAP.json records their identity-preserving rename toD2446–D2455.
Their source/attempt objects and file hashes were verified. D2456–D2465
retain ten final-build deaths. Both temporary registries reconcile all10
respective deaths, with none missing or unexported. Historical/CI exports
are explicit; no canonical repository write or migration occurs.

Disk exhaustion initially prevented temporary directory creation (11
engine-test errors) and suite/fixture launches; the source was unchanged
on retry. The resource failures are not deaths and are not hidden as
successful tests. Only regenerable Cargo caches were removed; raw runs,
linked binaries and dossiers remain. Retries used fresh output names and
passed; source hashes/seed lists/caps/assertions were not weakened.

Task CI must pass before track-p integration. This is later work excluded
from #292's authorized promotion and needs a separate owner review/promotion.
No population experiment or improvement claim.
