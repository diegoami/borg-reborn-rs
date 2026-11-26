## 2026-11-26 — R5 source refresh and ownership preflight

The owner accepted R4.5 at `e898abb5586135375434ad016e240686c022c53b`.
The R5 preflight starts from that exact snapshot on `feature/track-r`.
The standalone source refresh updates moving references under Rule 21.
The engine remains pinned to `574b618c` through the dedicated engine-change
workflow. The vendored Rust runtime and its pinned validation evidence stay
at the accepted R4.5 baseline. [R4.5 acceptance](../R45-ACCEPTANCE.md)

### Source changes

- The borg-rs reference contains the v15 carry and updated adoption notes.
  Its changes touch already-owned retreat, danger, escape and defence
  functions, which need an explicit carry if the vendor runtime advances.
- The better-borg-c reference records the v15 engine adoption and 11
  additional collected death cases, D0020 through D0030, since the old
  reference pin. The published copies already match the refreshed pin.
  Regenerating the 2,757-case reference index reproduces its existing bytes.
  The canonical Reborn evidence remains in Cemetery.
- The tracker re-pins its C fix base to `574b618c`. Current reports on
  stale descent state and dark exploration are leads for the stair and
  movement ownership audit.
- The Zoo catalogue and adapter reference advance. Historical R4.5
  controls, candidate and checker retain their original commits.
- The borg-runner source had no commits after `e55ac6f3`. Its read date
  records the refresh.
- The Angband refresh report identifies 17 later commits, including C Borg
  commits `fd411b29`, `894e9741`, and `186b37d5`. Any engine adoption has
  its own scope, source review and baseline under Rules 9 and 22.

### Ownership inventory

Count the top-level Rust files in `borg-core/src/` whose names start with
`borg_flow` or `borg_think_dungeon` using the exact accepted snapshot:

```sh
snapshot=e898abb5586135375434ad016e240686c022c53b
git ls-tree --name-only "$snapshot":borg-core/src |
  rg '^(borg_flow|borg_think_dungeon).*\.rs$' |
  while IFS= read -r file; do
    printf '%s %s\n' "$(git show "$snapshot:borg-core/src/$file" | wc -l)" "$file"
  done
```

| Native file | Lines |
| --- | ---: |
| `borg_flow.rs` | 2,026 |
| `borg_flow_dark.rs` | 908 |
| `borg_flow_glyph.rs` | 368 |
| `borg_flow_kill.rs` | 3,673 |
| `borg_flow_misc.rs` | 1,639 |
| `borg_flow_stairs.rs` | 1,426 |
| `borg_flow_take.rs` | 898 |
| `borg_think_dungeon.rs` | 1,728 |
| `borg_think_dungeon_util.rs` | 1,318 |
| **Total** | **13,984** |

The same inventory at the previous sizing baseline `2f74e1c5` is 11,736
lines. These totals include shared computation and state maintenance. The
ownership audit starts with the path and danger-cache mutations, the stair
entries, and the priority order in `borg_think_dungeon`. It records every
read, mutation, queued command and early return needed to preserve the
native decision order before moving flow policies into modes and goals.
The first runtime slices should validate the named boundaries with
constructed controls and the existing shadow replay. The 95-minute R4.5
Zoo run provides a reference for the full fixed-pin cohort.
[Measured carry](../R45-CARRY-REPORT.md) ·
[Sizing review](https://github.com/diegoami/borg-reborn-rs/pull/1286#issuecomment-7188954247)

### Review follow-ups

The next main carry needs a separate task for #852's resistance-aware
caution behavior. The exact ten-commit comparison used by the R4.5 report
contains nine changed files, including #1154's frontend save-load
acknowledgment. The effective lines in `harness/borg.txt` and
`harness/scary_monsters.txt` match across that comparison. Their byte hashes
changed, so future Zoo gates need the exact new input receipts.
[Resistance change #852](https://github.com/diegoami/borg-reborn-rs/issues/852) ·
[Post-pin comparison](https://github.com/diegoami/borg-reborn-rs/compare/24be98732748230062882fe2b7d9601dba5e2cc0...15a064f54490cd1b93c5ab6ef52091e175fce727)

The class-order references, store UI source label, healing-roll comments,
and historical C teleport wording belong in a separate source-maintenance
change. The teleport behavior question remains with the existing Zoo
experiment. A runtime carry needs an approved receiving pin and its own
parity evidence. [Milestone addendum](https://github.com/diegoami/borg-reborn-rs/pull/1286#issuecomment-7189689604) ·
[Settings question #733](https://github.com/diegoami/borg-reborn-rs/issues/733)

### Validation

The 325 copied files match their declared source blobs. All 15 profile
contract tests passed against the refreshed Zoo adapter and a binary
with the accepted build-input hash. The vendor attestation, R4.5 source
audit and all 2,721 local historical and qualified dossiers verified.
The archived validation receipts and gameplay inputs keep their original
bytes.
