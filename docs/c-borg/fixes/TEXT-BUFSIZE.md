# Screen-text buffer-size correction carry: verified product scenario

Consumer [93](https://github.com/diegoami/better-borg-c/issues/93), canonical
source [287](https://github.com/diegoami/neo-angband-tracker/issues/287)
(text-287, revised kit). Correction applied as `60034c84c` (`cherry-pick -x` of
source PR335 head `e7f670fa4d4f0672dc346637fc7be20c814a4b62`, patch
`patches/0068-text-buffer-size-bound.patch`), stacked on the carried 0025
shape (`3e7fe8a83`, `patches/0025-text-multibyte-termination.patch`).

## Defect

The carried termination fix bounded the UTF-8 conversion by
`ABS(n) * MB_CUR_MAX` — a limit derived from the character count, not from
the destination. On a wide terminal showing multibyte text that limit exceeds
the caller's 1024-byte buffer: `wcstombs` writes past it and still leaves the
text unterminated. The revised source evidence kit (fixtures `f9706564e`,
superseding `8d844e379`; owner decision "pass the buffer size" recorded on
source 287) demonstrates the overflow on the interim shape with a 301-column
wide case: 177 guard bytes overwritten in the normal build, and under
AddressSanitizer a `stack-buffer-overflow` WRITE of size 1201 in `wcstombs`
from `borg_what_text`. The correction passes the destination size, converts
at most `size - 1` bytes and always terminates the result (empty on a
conversion error); the callers pass `sizeof` their buffers.

## Pins and provenance

Before: `origin/main` `5cd2258dd78d9fed83c11f5df6a3afe88889337a`. Normal
before build snapshotted as `build/native-before` with its gate result
(`artifacts/native-before/result.json`, 965/965 + 5/5 at that commit); the
ASan before twin was rebuilt after the fact in the retained before tree
`.worktrees/text-bufsize-before` (detached at the before commit) so its
unit-test helpers share the before identity. `borg-io.c` is byte-identical
to the source interim shape `3e7fe8a83` and `borg-io.h`/`borg-think.c` are
pristine, so the verifier proves file by file that the source correction
applied to the before module equals the product module, and that the applied
commit is the only `src`/`lib` difference from the before commit. The
product gate ran at `cb04f973a` (965/965 + 5/5; game sources identical to
the verification HEAD). The fixture is the revised kit at `f9706564e`
(scenario `t287-variant.h` selects the unsized API for before phases and the
sized API for after phases; before-phase headers are exported from the
before ref by the verifier).

## Verified results (`artifacts/t287buf/scenarios`, 2026-11-08)

- `wide` (301 columns, 4-byte text) — the fail-before/pass-after case:
  normal-before 0/1, `terminated=0 strnlen=1024 guard_overwritten=177`;
  asan-before aborts with the AddressSanitizer `stack-buffer-overflow`,
  `WRITE of size 1201`, frames `wcstombs` and `borg_what_text` (exit 1);
  normal-after 1/1, `terminated=1 strnlen=1020 guard_overwritten=0`;
  asan-after clean (exit 0, no sanitizer report).
- `multibyte` (80 characters, one two-byte) — the carried 0025 contract
  holds before and after: 1/1, `terminated=1 strnlen=81` in both, logs
  byte-identical (`8474005f…`); the sized bound terminates the same 81
  bytes.
- `control` (pure ASCII): 1/1, `terminated=1 strnlen=80` in both, log
  byte-identical before/after (`e5bf1ca0…`).
- `result.json` records `verified: true` with all commands, pins, binaries
  and per-object hashes.

Native gate on the product: **965/965 unit + 5/5 frontend**; tooling
**129/129**. Prior-fixture matrix not rerun (the accumulated-matrix runner
is uncommitted; known gap recorded on checkpoint 1 and the milestone 6
review). The only fixture that compiles against the changed API is the
text-287 kit itself, superseded by the revised kit in this carry, and the
conversion bound can only differ from the 0025 shape where
`ABS(n) * MB_CUR_MAX` exceeds the 1024-byte buffer (terminals wider than
256 columns of multibyte text).

## Harness corrections disclosed

The first verifier run aborted in the ASan phase: the moved ASan snapshot
lacked the unit-test helper objects, and rebuilding it in place would have
compiled tip sources into a "before" tree. The ASan before twin was rebuilt
in the retained before worktree instead; the partial first attempt's output
directory was replaced by the complete rerun and kept no results file. The
second run failed its asan-after assertion because the detector matched the
word AddressSanitizer anywhere — the scenario's wide case prints a `LAYOUT
standalone buffer (AddressSanitizer)` line by design — and was corrected to
match actual `AddressSanitizer:` report headers, with no before-row
assertion loosened (the wide case must still fail with 177 overwritten
guard bytes and the 1201-byte WRITE). The product-gate identity check was
scoped to ancestor-with-identical-game-sources so docs/tooling commits may
follow the gate. Two verifier commits record both corrections.

## Scope and limits

Bounded screen-conversion scenario. It proves the buffer-size bound, the
preserved 0025 termination contract and the removed overflow; it does not
quantify how often wide terminals or multibyte message rows occur in
ordinary play and claims no survival change.

## Exact next work

1. PR from `task/carry-text-bufsize`, agent evidence review plus required
   hosted checks, then normal merge.
2. On merged main: frozen v1 cohort seeds 11/29/47 (cap 20000, wall 120,
   stall 30) against a fresh before-cohort at `5cd2258dd` — expected neutral
   below 256 columns; any divergence recorded as observed.
3. Receipts on issue 93, source 287 and checkpoint 1; evidence archived
   under `artifacts/t287buf/` (scenario runs) and the retained trees.
4. Then glowtrace-213 correction (source PR411 `c316a65b2`, `square_isglow`)
   on issue 224; recheck the source board for the newer draft pairs
   (painmsg-197, spasticscore-147, trollswap-201, iswarded-198,
   drophole-136).
