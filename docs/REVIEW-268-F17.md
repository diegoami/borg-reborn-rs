# Review #268: M1 review F17 — the recorder adapter as a configure dependency

Reviewer: an independent session, a different model family from the
implementation family.

* Branch: `fix/m1-recorder-deps`, head `81387c822202`
* Base: `main` at `f441296f`
* Finding addressed: **M1-F17** (Low) from the M1 independent review of
  `angband-recorder` (issue #2 in `diegoami/angband-recorder`)

The finding asked for exactly two things:

> **Correction direction:** add the adapter to configure dependencies and
> cover it with a generated-tree dependency test.

Both are done, and the second is a real functional test rather than a
text assertion.

## Verdict

**Accept.** The finding is correctly diagnosed, the fix is minimal and
correct, and the test reproduces the bug on unfixed `main` and passes on
the branch. One procedural follow-up (the handover entry the repository
rules require) and one observation outside the finding's scope.

## What was reproduced

| Check | Result |
| --- | --- |
| The bug on unfixed `main` | **reproduced**: configure with `SUPPORT_BORG_RECORDING=ON`, `touch engine/recorder-hooks.patch`, `make cmake_check_build_system` → the generated tree's mtime was **unchanged** (stale hooks) |
| The fix on the branch | **verified**: the same probe → "Configuring done", and the generated tree's mtime **changed** (the hooks were re-applied) |
| The new tests | `test_a_changed_adapter_reapplies_during_an_incremental_build` and `test_the_adapter_is_declared_as_a_configure_dependency` both pass; the first ran **3×** without flaking |
| The test fails without the fix | confirmed by the main-side probe above (the same probe the test performs) |
| Smoke parity | **3/3 runs identical** against `harness/reference/smoke-rust` (keys, messages, states) — the fix changes no play |
| Generated tree content | unchanged: `engine/patches/`, `engine/frontend/`, `engine/sources.cmake`, `engine/apply-patches.sh` and `engine/recorder-hooks.patch` are all byte-identical to `main` |
| Recorder-enabled build | configures (`recorder pin OK: v0.1.0 c4986b26b9fd`), all eight hook files patched, builds, and `harness/check-recorder.py` passes: "Consumer recorder round trip passed, including final native save bytes" |
| Test suites | 42 engine tests (5 build-gated skips), 17 scripts, 70 tools — all pass |
| CI | `rust`, `engine` and `recorder` all green on `43706177195` |
| Death dossiers | D2359, D2360, D2361 retained; every file hash in `files_sha256` verifies |

## Findings

### F1 — the diagnosis is correct and the fix is exactly the right shape

`engine/CMakeLists.txt` applies the recorder adapter at **configure**
time, inside the `SUPPORT_BORG_RECORDING` block:

```cmake
execute_process(COMMAND patch --batch --fuzz=0 -p1 --input "${ENGINE}/recorder-hooks.patch"
    WORKING_DIRECTORY "${TREE}" RESULT_VARIABLE RECORDER_PATCH)
```

`CMAKE_CONFIGURE_DEPENDS` listed `engine/patches/*.patch` and
`engine/UPSTREAM` but not `engine/recorder-hooks.patch`, so CMake had no
reason to re-run configuration when the adapter changed — and the
generated tree kept the old hooks. The fix adds exactly that one entry
(`engine/CMakeLists.txt:29`).

Adding the adapter to the *configure* dependencies rather than to a build
rule is the right remedy: the patch is applied by `execute_process`
during configuration, so the only way to re-apply it is to re-configure.

### F2 — the test is functional, specific, and not vacuous

The test does not merely assert that the string `recorder-hooks.patch`
appears in `CMakeLists.txt` (that is the second, weaker test). It:

1. configures a throwaway build with `SUPPORT_BORG_RECORDING=ON`;
2. asserts the patched tree was generated (`tree/src/z-rand.c` exists),
   so an empty or failed build cannot pass;
3. records the tree file's mtime;
4. touches **only** `engine/recorder-hooks.patch` and sleeps past
   CMake's whole-second mtime comparison;
5. runs `make cmake_check_build_system`;
6. asserts the tree's mtime **changed**.

`z-rand.c` is a well-chosen sentinel: it is one of the eight files
`engine/recorder-hooks.patch` actually modifies, so its mtime is a valid
proxy for "the tree was re-applied".

Three properties make this a real regression test rather than a
tautology:

- **It fails without the fix.** I ran the identical probe on unfixed
  `main`: the tree's mtime was unchanged, i.e. the stale hooks the
  finding describes. On the branch the same probe re-ran CMake.
- **It is specific.** The test touches only the adapter patch, so the
  only thing that could trigger a re-configure is the adapter's own
  dependency declaration. A re-configure caused by anything else would
  not be triggered by this test.
- **It leaves no trace.** The mtime is restored in a `finally` block
  (`engine/tests/test_build.py:79-80`), and the working tree was clean
  after every run.

It is also stable: three consecutive runs passed, and the `time.sleep(1.2)`
matches CMake's whole-second comparison rather than racing it.

The one thing the test does **not** cover is the reverse direction — that
an *unchanged* adapter does not re-apply the tree. That is a performance
property, not a correctness one, and CMake's own behaviour provides it.

### F3 — the earlier CI failures were the RULES 12 gate working, not a defect

Two earlier CI runs on this branch failed in the `engine` job. The cause
was not the fix: the reconcile gate reported deaths that the branch's new
build identity had produced and that were not yet in the registry:

```
UNRECORDED: D0001-rust-seed102: Fang, Farmer Maggot's Dog at turn 3640 (keys 50491fa13924)
UNRECORDED: D0002-rust-seed1020: a giant white mouse at turn 8922 (keys c542b7d80373)
```

That is the rule doing its job (RULES 12: a measured run's natural death
must be recorded, or CI fails). The two "retain the … death" commits
added D2359–D2361, and the head is green. Worth noting explicitly,
because a reviewer skimming the run history would otherwise see two
red runs and wonder.

The three retained dossiers are correct: each keeps its trace tail (200
rows), its `encounter_family`, and every file hash in `files_sha256`
verifies.

### F4 — the handover entry is missing (procedural, follow-up)

AGENTS.md requires that every PR updates `docs/HANDOVER.md`:

> 2. docs/HANDOVER.md says what was done, what was found, what is left,
>    and anything the owner has to decide;

This PR touches only `engine/CMakeLists.txt` and `engine/tests/test_build.py`
plus the death dossiers. The handover has no entry for the F17
correction, and its recorder section (2026-11-14) still describes the
integration as it was before this fix.

This does not affect the fix's correctness and should not block the
merge, but the rule is explicit and this repository has been holding to
it. The entry needs to say: what F17 was, that the adapter is now a
configure dependency, that the tree is regenerated when it changes, and
that the generated tree's content — and therefore the game — is
unchanged.

### F5 — observation outside this finding's scope: the upstream tree's content is not a configure dependency

While auditing the configure-time inputs I checked the other one.

`engine/UPSTREAM` is declared in `CMAKE_CONFIGURE_DEPENDS`, but it is a
**marker file** (repository, commit, tree id, path) — not the content of
`engine/angband/`. So CMake does not re-configure when a file *inside*
the upstream tree changes in place. I confirmed this: touching
`engine/angband/src/main.c` twice, the second touch did **not** re-apply
the generated tree.

This is **not** a defect in this PR and **not** within F17's scope (F17
names the recorder adapter, a different input). It is also benign in
practice, for two reasons:

- a legitimate re-pin replaces the whole tree **and** updates
  `engine/UPSTREAM`, which *is* a configure dependency, so the flow that
  matters is covered; and
- RULES 9 forbids editing `engine/angband/` at all, and
  `engine/tests/test_engine.py:34-38` fails if the tree differs from
  upstream's recorded tree id or has local changes.

The residual hole is narrow: someone edits a file in `engine/angband/` in
place, and the build keeps the old tree until a test catches it. It is
worth a separate issue rather than a change here, and the natural
remedy would be to make the tree's content — not just `UPSTREAM` — a
configure dependency.

### F6 — the recorder pin and the vendored crate are untouched

The PR body says the vendored `vendor/angband-recorder` v0.1.0 pin is
unchanged, and it is: `recorder-pin.json` and `vendor/angband-recorder/`
are identical to `main`, and the recorder-enabled build accepts the pin
("recorder pin OK: v0.1.0 c4986b26b9fd"). Re-vendoring the recorder's
corrections is correctly left to a separate step.

## Not checked

* The other findings of the M1 review (F1–F16, F18, F19) are outside
  this PR's scope and are not dispositioned by it.
* The reviewer's own reproduction receipts on the recorder side; this
  review reproduced F17 in *this* repository, which is where the finding
  was filed.
* No population experiment, and no sealed suite was opened.

## Follow-up

1. Add the handover entry for the F17 correction (F4).
2. File the upstream tree's content as a separate issue (F5): make
   `engine/angband/`'s content, not only `engine/UPSTREAM`, a configure
   dependency, so an in-place edit cannot leave a stale generated tree.
