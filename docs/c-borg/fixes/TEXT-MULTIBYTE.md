# Multibyte screen-text carry: verified product scenario, ready for adoption

Consumer [93](https://github.com/diegoami/better-borg-c/issues/93), canonical
source [287](https://github.com/diegoami/neo-angband-tracker/issues/287)
(text-287). Fix applied as `c6c51d4d09e591da9d5df313666fe95b31f05e03`
(import of source PR335 `3e7fe8a8399352c4fe67f5de2d8d48b5e0bd04c1`, patch
`patches/0025-text-multibyte-termination.patch`).

## Defect

`borg_what_text()` bounded the UTF-8 conversion by one byte per wide
character. A row of eighty characters containing exactly one two-byte
character needs 81 bytes plus its terminator, so conversion truncated at
the 80-byte wide-count limit without terminating the destination. The
caller then measured the uninitialized buffer as a string; with the buffer
deliberately junked, the unchanged fixture reads all 1024 stale bytes. This
is the stale-text/phantom-monster path, not merely cosmetic truncation.

The correction bounds conversion by the widest encoding for the same
character count and falls back to an empty string on conversion error.

## Pins and provenance

Before build: twenty-four-carry `915bba5262ebf855285b147380fc2b8144480437`
(retained tree `../blind-frighten`, normal and ASan cores, gate at
`713d9ad4`, source-identical to `915bba526` over src/lib). `borg-io.c` had
no earlier consumer patch. The verifier proves that the source diff applied
to the before module equals the product module. The source evidence fixture
at `8d844e379` is used unchanged.

## Verified results (`artifacts/t287/scenarios`, 2026-11-06)

- normal-before `multibyte`: 0/1, `terminated=0`, `strnlen=1024` — the
  unterminated conversion reads the whole stale buffer.
- normal-after `multibyte`: 1/1, `terminated=1`, `strnlen=81` — the full
  multibyte row is present and terminated.
- `control` (pure ASCII): `terminated=1`, `strnlen=80`, 1/1 on every phase,
  byte-identical before/after.
- asan-before/after mirror the normal phases; no sanitizer report.
- `result.json` records `verified: true` with all commands and hashes.

Native gate on this core: **965/965 unit + 5/5 frontend**. Regression matrix
(`artifacts/t287-carry/regression.py`): all **95 prior native fixture cases**
pass (93 identical plus the two documented losdispel interactions asserted),
and the two new text cases pass, **97 total**. Tooling 29/29.

## Scope and limits

Bounded native screen-conversion scenario with a deliberately unterminated
caller buffer. It proves termination and exact converted length under the
recorded UTF-8 locale; it does not quantify how often phantom monsters occur
in ordinary play or claim a survival change.

## Exact next work

1. PR from `task/carry-text-multibyte`, agent evidence review plus required
   hosted checks, then normal merge.
2. On merged main: frozen v1 measurement seeds 11/29/47, cap 20000, wall
   120, stall 30.
3. Mark 93 carried/open+patch-active; receipts on issue 93, source 287 and
   checkpoint 1; evidence archive under `artifacts/t287-carry/`.
4. Then calcreload-325 and opblock-324 (demonstrated low-priority native
   leaks), followed by brand-240; recheck source207 for newer demonstrations.
