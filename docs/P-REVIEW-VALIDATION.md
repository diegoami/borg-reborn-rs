# Track P retrospective review follow-up (#275)

Implementation pin: `132aeb26a54a7988ca013256f752fce19f24afc9` (stacked on
workflow/main synchronization #277). Source SHA256:
`4182670dfb294135edca35c3650ae73c5e1ce50b818aa0934560c37263c8eb70`.
Binary SHA256: `315489b386edfa5d795aa25122ed5e4112f4a31838bb41f28bf3280e2ef3d776`.
The upstream engine remains `42cd156ac`; no population experiment was run.

## Disposition of review #259

| Finding | Resolution |
| --- | --- |
| F1 selector fallback | Invalid environment/file selectors now exit 2 with source/value diagnostics. Absent selection defaults to reborn; reborn/0 and borg-rs/1 remain valid, environment first. Non-UTF-8 environment values also fail. Pure parsing and actual pinned-adapter process tests cover the contract. |
| F2 rest units | The proposed key-digit interpretation is incorrect: manual command.rst:186–194 and engine ui-command.c:191–226 confirm decimal rest turns. Keep names/defaults (500 and 100). Engine cmd-cave.c:1639–1645 reveals that 1 repeats the previous duration: restrict explicit durations to 2–9999 and test boundaries. Rest may be disturbed early. |
| F3 hit-power multiplier | It multiplies `(level + power)` inside a denominator in borg_danger.rs, so clarify the unit to denominator multiplier while retaining the positive bound. The generic zero-denominator rejection test includes this unit. No arithmetic/default changes. |
| F4 summon citations | These default-1 citations identify the original implicit `p += spell_power` branch. The nearby literal 3 is a different branch. Preserve correct locations and explain the implicit factor in metadata/catalogue. |
| F5 RNG claim | Qualify call-count/order preservation at defaults. Landing sample/trial overrides change draw budgets and subsequent RNG state. |
| F6 retained literal | Remove the stale exactly-one-monster/one-blow claim. |
| F7 casing | Document lowercase file keys and uppercase environment names. Strict rejection remains unchanged. |

The original reports are archived unchanged in REVIEW-259-P1-P3.md and
REVIEW-260-WORKFLOW.md from `ade599661355bd5556015c8cf0a50b54d661a703`.
Their recommendations do not constitute owner milestone acceptance.

## Evidence reconciliation

The reviewer reported an original-core full suite at P3.1 source
`1f5eb966caeb02bd8584f439961747e5fe5df89aed654bed46a7ea8b1c6ca0e9`
without preserving its 31 natural deaths. Reproduced that public regression
suite on the immutable original P3.1 binary
`cfdcdaf3af68f5b06be65e6c337600bec246ea4ac7c0f22198eda05f78e9bd66`.
This is a reproduction, not recovery of the reviewer's raw logs.

All 45 runs match harness/reference/full-rust for keys, messages and states.
Export used the actual old BUILD_ID, never the updated build identity.
D2359–D2388 retain 30 new cases; seed 1020 is an existing encounter/result
in D2355. Reconciliation finds 31 natural deaths, none unexported or missing
from the committed registry. Repeats do not increase independent counts.

## Updated build validation

Required Rust checks pass: package formatting, workspace Clippy with warnings
as errors, original crate tests 153 + 4 + 5, and release core tests 138
(1 ignored). Script tests 17 and tool tests 70 pass.

Actual pinned-zoo native profile contract passes all 5 tests, including
invalid numeric/bool selectors, invalid/empty environment overrides and
valid environment precedence over an invalid file selector. All 42 engine
tests pass (1 environment-dependent binding-regeneration skip); vendoring
check passes against pinned borg-rs.

Full 45/45 default runs match pinned v12 keys, messages and states byte for
byte; 295,464 trace rows have contiguous think IDs and no unclassified
checks. Trace on/off preserves all three play logs; the diagnostic run has
351 trace rows and exactly 1,734 effective default parameters. Original-core
smoke matches the reference 3/3; the separate death fixture completes.
D2389–D2398 retain ten natural deaths from this updated source (eight full,
one smoke, one fixture). Reconciliation finds none missing or unexported.
The updated example catalogue/manifest validates 384 exploratory cells and
900 fresh confirmation seeds through the actual pinned catalogue; no games
from those experiments were played.

Default play compatibility is established for those public regressions,
not for every possible configuration or game. Intentional input changes:
invalid core selectors now fail instead of falling back, and numeric rest
value 1 now fails instead of repeating a previous duration. Parameter names
and defaults are preserved; metadata/snapshot hashes change.

Required GitHub CI remains the integration gate; this report does not
constitute owner milestone acceptance.

The owner sweep prompt/examples now pin the fix commit above. That pin
must complete validation and the owner review gate before dispatch.
Historical P3.2 evidence continues to describe its original source; it
must not be relabelled as evidence for this updated build.
