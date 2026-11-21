# Iswarded-198 carry: verified product scenario

Consumer [286](https://github.com/diegoami/better-borg-c/issues/286),
canonical source
[198](https://github.com/diegoami/neo-angband-tracker/issues/198)
(iswarded-198; source PR452 `62242ac2`, evidence PR451). Fix applied as
`ecfea5064` (`cherry-pick -x` of source
`62242ac26bab4f128793d767f0f502bf4ca4d820`, patch
`patches/0070-rune-check-current-level.patch`).

## Defect

When a monster hits the Borg while it tracks runes, or a Rune of Protection
breaks, the message handler clears the tracked runes and then cheats a look
at the rune under borg.c. Messages are parsed at the Borg's next keypress,
before borg_update() moves borg.c onto the current level: a -more- line
stored just before a level change (Word of Recall firing in the same game
turn as the monster's action) is parsed on the new level with the previous
level's position. On a smaller new level (the 66x22 town) that position lies
outside the level: `square_iswarded()` fails the `square_istrap` bounds
assertion in the normal build (SIGABRT), and with assertions off reads
outside the level's row array — an ASan heap-buffer-overflow READ of size 8
in `square_istrap` from `borg_parse_aux`. The correction guards both rune
checks with `square_in_bounds(cave, borg.c)`; the tracked runes are still
cleared. Memory-safety carry.

## Pins and provenance

Before: merged main `47be91c95b471aae5dc1de8bca9bb2942ae769d7` (snapshotted
build `build/native-before`, gate result `artifacts/native-before/result.json`,
965/965 + 5/5 at that commit). The ASan before twin was rebuilt in the
retained before tree `.worktrees/iswarded-before` (detached at the before
commit) with NDEBUG, matching the source kit's ASan variant. The verifier
proves that the source diff applied to the before `borg-messages.c` equals
the product module and that the applied commit is the only `src`/`lib`
difference from the before commit. Product gate at the tip. Fixture: kit
`563a2616` at head `5454144a`, asserted byte-identical to the run pin.

## Verified results (`artifacts/isw198/scenarios`, 2026-11-08)

- `rune-town` / `hit-town` (out-of-level cases) — fail before, pass after:
  normal-before exit -6 (SIGABRT from the `square_in_bounds` bounds
  assertion), 0/1; asan-before exit 1 with the AddressSanitizer
  `heap-buffer-overflow` READ of size 8, site `square_istrap <- borg_parse_aux
  borg-messages.c:1192` (rune) and `:629` (hit) — the source kit's sites
  `:1188`/`:625` shifted by the consumer's carried hunks in the same file;
  normal-after and asan-after 1/1, `survived=1`, no assertion, no report.
- `rune-inside`, `hit-inside`, `hit-noglyph` (controls): 1/1 in both phases,
  `survived=1`, logs byte-identical before/after in the normal build
  (`2925cac6…`, `8e732605…`, `2407dc8d…`) and in the ASan build.
- `result.json` records `verified: true` with all commands, pins and hashes.

Native gate on the product: **965/965 unit + 5/5 frontend** (before and
product); tooling **129/129**. Prior-fixture matrix not rerun (uncommitted
runner; known gap recorded on checkpoint 1 and the milestone 6 review).

## Harness corrections disclosed

Three verifier commits record the corrections, none of which loosens a
defect requirement: the ASan phases needed NDEBUG (assertions off, like the
source kit) — otherwise the bounds assertion fires before ASan sees the
overflow — and the scenario binaries must define NDEBUG too or
`flag_on_dbg` is unresolved against the NDEBUG core; the exit check scopes
the SIGABRT to the failing before cases; the frame extractor no longer pins
the `borg_parse_aux` frame index. Two aborted attempt directories were
replaced by complete reruns; the causes are described here.

## Scope and limits

Bounded message-parsing scenario across a level change. It proves the
out-of-level read is gone and the in-level behaviour is unchanged; it does
not measure how often a rune break or tracked-rune hit coincides with a
level change in ordinary play, and claims no survival change.

## Exact next work

1. PR from `task/carry-iswarded`, agent evidence review plus required
   hosted checks, then normal merge.
2. On merged main: frozen v1 cohort seeds 11/29/47 against the 0069-after
   cohort at `47be91c95` (traces `ce4759e4…`/`eb86314a…`/`123995e6…`); the
   fix is expected neutral (the crash path requires a same-turn recall);
   any divergence recorded as observed.
3. Receipts on issue 286, source 198 and checkpoint 1; evidence under
   `artifacts/isw198/`.
4. Then painmsg-197 + spasticscore-147 (PR pair), trollswap-201 +
   drophole-136 (PR pair).
