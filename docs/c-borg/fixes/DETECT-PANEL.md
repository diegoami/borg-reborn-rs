# Detect panel carry: verified product scenario, ready for adoption

Consumer [72](https://github.com/diegoami/better-borg-c/issues/72), canonical
source [290](https://github.com/diegoami/neo-angband-tracker/issues/290)
(detect-290, source priority high). Fix applied as
`c3e8f60a3ea164b74ce76bbac7a3097263a13a9f` (import of source PR306
`02fb00af6abdb165fc1b445c42dc85d2263a3cdd`, patch
`patches/0018-detect-panel-bounds.patch`).

## Defect

The Borg updates on the second panel row of a large cave, queues the
self-reaction of a trap detection (the same `borg_react` call the spell
path uses), and the level changes to a cave exactly one panel high — the
height where the strict old-offset reset lets equal offsets through while
the panel count still yields a single row pair. The queued reaction keeps
the old panel offsets and its four-cell mark is indexed after the update
resized the arrays: the original reads its row pointer one row past the
outer array and writes through it — the native run crashes, and
AddressSanitizer names the read at `borg_handle_self` (borg-update.c:1396).
The correction guards every detection branch and the trap-creation reset
(with the same indexing) with a panel-bounds check: the stale reaction is
dropped and the run completes.

## Pins and provenance

Before build: seventeen-carry `5667d085dcdef669a587c8e8153aa6cca8c20ecd`
(retained tree `../slay-traits`, normal and ASan cores, gate at
`6bcdd28a`, source-identical to `5667d085` over src/lib).
`borg-update.c` is pristine in this product, so the imported module is
byte-identical to the reviewed source commit (module SHA256
`0eadbb2d75bf66417466164f2893603b9e44e82cd6ba03470a086e47711aa1ef`,
reproduced by the patch round trip from pristine `e473720`).
Before-ASan replaces only the update object with the pre-fix source,
compiled with the ASan build's recorded flags; no normal object changes.

## Verified results (`artifacts/d290/scenarios`, 2026-11-06)

- normal-before `stale`: **exit −11 (SIGSEGV)** — the native crash,
  matching the source evidence's recorded exit 139 and the segfaulting
  harness runs that had blocked this fix.
- normal-after `stale`: 1/1, `last_row_mark=0` — the stale reaction is
  dropped.
- asan-before `stale`: exit 1, **heap-buffer-overflow READ** in
  `borg_handle_self` (pre-fix object linked into the fully instrumented
  core).
- asan-after `stale`: 1/1 clean.
- `fresh` control (reaction queued for the current cave): `last_row_mark=1`,
  1/1 on every phase, **byte-identical logs across all four phases** —
  legitimate detection marking untouched.
- `result.json` records `verified: true` with all commands and hashes.

Native gate on this core: **965/965 unit + 5/5 frontend**. Regression
matrix (`artifacts/d290-carry/regression.py`): all **81 prior native
fixture cases whole-log identical** to their baselines (79 + 2 slay).
Tooling 29/29.

## Scope and limits

Bounded native scenario: the production `borg_react`/`borg_update` calls
with a level change to the one-panel-high cave the report describes. The
crash and the sanitizer report are the demonstrated harm; no autonomous
reachability, frequency or survival claim. ASan options
`detect_leaks=0:intercept_strcmp=0` (disclosed pristine init overread);
the tracked read/write are compiler-instrumented.

## Exact next work

1. PR from `task/carry-detect-panel`, agent evidence review plus required
   hosted checks, then normal merge.
2. On merged main: frozen v1 measurement seeds 11/29/47, cap 20000, wall
   120, stall 30.
3. Mark 72 carried/open+patch-active; receipts on issue 72, source 290 and
   checkpoint 1; evidence archive under `artifacts/d290-carry/`.
4. Then the demonstrated valuation pairs in posted order: activ-260
   (PR308/309) and reserve-258 (PR318/319), plus rechecking source207 for
   newer demonstrations (259/262/261 ranked next by the source lane).
