# Glow-array overflow carry: verified product scenario, ready for adoption

Consumer [102](https://github.com/diegoami/better-borg-c/issues/102), canonical
source [311](https://github.com/diegoami/neo-angband-tracker/issues/311)
(glowoverflow-311, priority high). Fix applied as
`3cb61705d1adcf813c8318285d7c29abf8c61d5a` (import of source PR346
`d4dcb983c9623757b523a7fae861b624d858e23d`, patch
`patches/0028-glow-array-bound.patch`).

## Defect

The production out-of-light flow collected every glowing grid in a fully
perma-lit arena panel into fixed 1536-entry glow arrays. The 1600-grid panel
therefore wrote 64 entries past the arrays. AddressSanitizer identifies a
global-buffer-overflow `WRITE of size 1` in `borg_flow_light` at the glow-array
store. The correction stops collection at the bound and still runs the flow
over the bounded set.

## Pins and provenance

Before build: twenty-seven-carry `f1f71a2ff989d7d08e96d29e7a358fe1cd681606`
(retained tree `../stale-target`, normal and ASan cores, gate at `0d843a41`,
source-identical to `f1f71a2ff` over src/lib). `borg-flow-misc.c` had no
earlier consumer patch. The verifier proves that the source diff applied to
the before module equals the product module. The source evidence fixture at
`1a9946c6` is used unchanged, including its terminal sizing that keeps this
scenario distinct from the separately noted oversized-terminal panel-bounds
lead.

## Verified results (`artifacts/g311/scenarios`, 2026-11-07)

- normal-before `glow`: 0/1, `glow_n=1600`, `overflow=64`.
- normal-after `glow`: 1/1, `glow_n=1536`, `overflow=0`; the flow completes.
- `control` (1000 glowing grids): `glow_n=1000`, `overflow=0`, 1/1 in both
  normal builds, byte-identical before/after.
- asan-before: exit 1 with **global-buffer-overflow, WRITE of size 1, in
  `borg_flow_light`**.
- asan-after: 1/1, `glow_n=1536`, `overflow=0`, no sanitizer report.
- `result.json` records `verified: true` with all commands and hashes.

Native gate on this core: **965/965 unit + 5/5 frontend**. Regression matrix
(`artifacts/g311-carry/regression.py`): all **101 prior native fixture cases**
pass (99 identical plus the two documented losdispel interactions asserted),
and the two new glow cases pass, **103 total**. Tooling 29/29.

## Scope and limits

Bounded native production-flow scenario with a fully glowing arena panel. It
proves the array-bound violation and its correction, not the frequency of a
fully lit panel in ordinary play or a survival improvement. The separate
latent oversized-terminal panel-bounds lead is outside this fix and is not
claimed resolved.

## Exact next work

1. PR from `task/carry-glow-overflow`, agent evidence review plus required
   hosted checks, then normal merge.
2. On merged main: frozen v1 measurement seeds 11/29/47, cap 20000, wall
   120, stall 30.
3. Mark 102 carried/open+patch-active; receipts on issue 102, source 311 and
   checkpoint 1; evidence archive under `artifacts/g311-carry/`.
4. Recheck source207; when no higher-priority demonstration is ready, resume
   opblock-324 and calfree-323, then brand-240.
