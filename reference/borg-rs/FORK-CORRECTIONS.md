# Narrow fork corrections and milestone transition

Owner update (2026-11-07): [M5-PLAN.md](M5-PLAN.md) freezes 28468aa for focused
review, then [C-CONSUMER-ADOPTION.md](C-CONSUMER-ADOPTION.md) governs the new
lane. After M5 acceptance, accepted better-borg-c milestones become the pinned
adoption baseline; neo source links stay provenance. This supersedes treating
the old source-tracker queue or pristine version as the automatic post-M5 target.
The owner selected C milestone2 on2026-11-08: seven fixes at2675d198e,
tracked on [Rust#6](https://github.com/diegoami/borg-rs/issues/6) with full
provenance/state in [adoption/c-m2](adoption/c-m2/README.md). No new Rust tag
or milestone acceptance is inferred. The narrow crash/play-blocker policy
remains, with explicit coordination and evidence.

The owner then directed C milestone3 (better-borg-c `702f091f6`, 27 carries),
tracked on [Rust#8](https://github.com/diegoami/borg-rs/issues/8) with state in
[adoption/c-m3](adoption/c-m3/README.md). Its patch0018 (same source #290)
bounds-checks every queued detection mark instead; it supersedes the local
equality reset in the table below, which is reverted to the pristine `>` test
in C and Rust (history and receipts below are unchanged). Local #287 and #322
remain and cover consumer patches0025/0027.

Engine correction during the C milestone4 adoption (owner-directed narrow
fix, 2026-11-08): `do_cmd_wiz_cure_all()` in `cmd-wizard.c` also frees the
known curse list. Before, the Borg's cheat-death answer left
`obj->known->curses` set after freeing `obj->curses`, and a later Remove
Curse crashed in `curse_menu` (C and Rust share this engine code).
Reported upstream as
[neo-angband-tracker#474](https://github.com/diegoami/neo-angband-tracker/issues/474)
for that repository's own process; evidence is in
[validation/2026-11-08-c-m4](validation/2026-11-08-c-m4/README.md).

The C milestone7 adoption (better-borg-c `3573d741e`,
[Rust#17](https://github.com/diegoami/borg-rs/issues/17),
[adoption/c-m7](adoption/c-m7/README.md)) supersedes two local corrections
below with consumer patches. 0068 corrects the consumer's 0025 to pass the
destination size, which replaces local #287 in C and Rust. The only
behavioural difference is the `wcstombs()` error path: it now terminates at 0
and goes on to the normal return instead of returning an error at once. 0065 replaces local #345
with the same call, so behaviour is unchanged. #322 and #295 remain. The
sections below are kept as history.

The C milestone8 adoption (better-borg-c `017e038e8`,
[Rust#19](https://github.com/diegoami/borg-rs/issues/19),
[adoption/c-m8](adoption/c-m8/README.md)) also carries the owner's
munchkin-by-class change (better-borg-c PR302). That change is a behaviour
default, not a crash correction: Warriors, Druids and Rangers take the
munchkin start unless `borg_munchkin_by_class` is set FALSE in borg.txt.

The sections below record the original M4 and post-M4 correction history.

Owner direction, 2026-11-06: inherited C behavior may be corrected when
it crashes or is a blatant play blocker. Mirror the narrow correction
in C/Rust where applicable, mark FORK, link the tracker and test it.
Other conflicts are decided case by case. Do not preserve crashes,
undefined bytes or known failures as milestone requirements. Broader
changes follow a verified milestone and an owner-created tag.

This supersedes older absolute no-C-changes/no-behavior-changes wording
for these corrections. Pinned upstream stays e473720ed. Original references
remain separate; new corrected-C references require two identical C runs.
No official Angband issue/PR/comment or automatic main/tag move is authorized.
The owner subsequently requested main integration and explicitly delegated
creation/push of tag `m4` at reviewed commit `3923a21`; both are complete.

## Validated production corrections at28b8170

| tracker | correction | evidence |
|---|---|---|
| [#290](https://github.com/diegoami/neo-angband-tracker/issues/290) | reset stale offsets at equality with cave dimensions before queued detection writes | same C/Rust guard; original Rogue4006 scenario now finishes its2M limit in C twice and Rust, matching all three logs |
| [#287](https://github.com/diegoami/neo-angband-tracker/issues/287) | use actual caller byte capacity, reserve NUL, return empty text/error on invalid conversion | real-function native before/after controls, full UTF-8/boundary/zero-one-capacity/error checks; original full baseline restored; all five endgame cases match C/C/Rust |
| [#322](https://github.com/diegoami/neo-angband-tracker/issues/322) | require thrust targets at distance exactly one | actual C body fixture rejects center/distant targets and retains all eight neighbors; source26 Warrior2000's `+5` stop at1,887,392 becomes a valid adjacent attack; all three corrected runs reach5,000,001 |

Latest receipt: [2026-11-06-melee](validation/2026-11-06-melee/README.md).
Fresh C/C and Rust/C crash-fixes6/6, extended12/12 and natural2/2 compare
keys/messages/player state, with DONE, successful process outcomes and no
overrides. Every extended scenario reaches its original limit. Both builds
also match105 original baseline cases and8 captured-Rogue references.
Thirty forked engine captures have identical metadata and direct saved
bytes across C1/C2/Rust. They restore engine state for fresh replays, not
private Borg memory. Current corrected references are crash-fixes,
extended and natural; original baseline/endgame bytes are preserved.

The prior session described an unchanged-C observer locating the detection
next-row write with q_y=1, q_x=0 and a NULL row pointer. Its raw trace was
not transferred; current receipts do not claim to recapture that trace or
independently verify the tracker findings upstream.

## Why the UTF-8 correction was refined

The transferred bd184cc byte-limited fix passed six cases, but both builds
matched only43/45 original full cases (while matching each other45/45).
Screen columns were still used as UTF-8 byte limits, truncating Sméagol
messages into phantom names; seed1003 also changed keys/state. The native
full-message regression reproduces the truncation.

Commit26a96be supplies actual C buffer capacity (`sizeof` at all five
callers); Rust uses its slice length. Full105-case parity returned. That
source's extended matrix then matched12/12 but exposed the separate melee
stop. Commit28b8170 corrects only that invalid target eligibility; the
unexpected-direction abort safeguard remains.

Historical references/receipts are explicit: crash-fixes-bd184cc,
crash-fixes-26a96be, extended-26a96be and their source-qualified validation
directories. The latest six-case logs are unchanged from the valid26
reference, but their current manifest/receipts bind fresh runs to28b8170.
Earlier interrupted cloud batches remain incomplete, not passes.

## Tracker and unresolved scope

Published #287/#290 updates:
[#287 comment](https://github.com/diegoami/neo-angband-tracker/issues/287#issuecomment-6693735109),
[#290 comment](https://github.com/diegoami/neo-angband-tracker/issues/290#issuecomment-6693741157).
#322 records the observed stop and narrow correction; its [final receipt
update](https://github.com/diegoami/neo-angband-tracker/issues/322#issuecomment-6695689265)
links the completed75d6ced checkpoint and was read back. Independent tracker verification and upstream
source resolution remain explicit, not inferred from consumer validation.

Allocator code-reading leads #323-#325 remain separate reported findings;
no leak-freedom/game-exhaustion claim is made. Persistent native checks now
cover declared calculator/map/flow/melee/glyph/lunal kernels, with seven
actual-source fault controls, C UBSan and measured body coverage.

The declared M4 corpus is verified at the unchanged28b8170 production inputs:
139 successful oracle-checked engine executions in11 groups, logic lines85.38%
and LLVM function instances91.96%. Six synthetic cases reach real self-lunal
entry/first descent and glyph creation with complete C/C/Rust engine-save
byte equality. Their wizard setup and finite stops are explicit. Existing
recognition finding#295 and reverse-flow lead#345 are recorded, without a
broader tactic correction. Full formation-building, unbounded scumming and
an unassisted win remain unestablished. See the2026-11-07 coverage/synthetic
receipts. The owner explicitly requested the main fast-forward to40194d8;
it is complete. On resume the owner explicitly delegated tag creation;
`m4` = `3923a21` is pushed and verified.

## Post-M4 glyph recognition (#295)

The agreed first post-tag correction separates remembered glyph recognition
from the harmful-player-trap predicate. Both C and Rust use map_info's known
trap GLYPH flag; unseen live Rune locations are not consulted. A Rune is
marked as a glyph rather than a harmful trap, and existing duplicate/capacity
checks remain intact. The old Rust first-live-trap FFI helper is removed.

Six real engine before/after cases cover known Rune/decoy, revealed pit,
hidden Rune, empty floor and a prelaid24-rune ring. Corrected C twice and
Rust match all three logs and full final-save bytes. Native C recognizes
24/tracks24 in the ring, sets Morgoth position and preserves all20 scrolls;
M4 had map0/tracked1 and cast another Rune. Harmful-trap and hidden-Rune
controls pass, and both separate C observers preserve all unobserved output
and save bytes. Existing M4 references stay unchanged; corrected C2 oracles
are glyph-recognition and synthetic-tactics-glyph-295. This is consumer
fork validation, not independent source-tracker verification or a claim of
autonomous construction/combat benefit. See the
[recognition receipt](validation/2026-11-07-glyph-recognition/README.md).
Both implementations also match121 existing references (105 baseline,
8 captured-Rogue,6 crash-fixes and2 natural). New recognition6 and existing
synthetic6 pass C twice and Rust, including complete final saves. Tests100/
one existing ignored, clippy, native glyph1,600 and154 engine-object/link
checks pass. Extended12 and coverage are retained M4 results, not recollected
for this correction. No later source inherits the tag's measured percentages.
The correction and receipts are published at `da39bf5`; the
[existing #295 update](https://github.com/diegoami/neo-angband-tracker/issues/295#issuecomment-6705894975)
was posted and read back. The tracker remains responsible for independent
source qualification/adoption.

## Owner-accepted M4: glyph reverse flow (#345)

After owner acceptance/closure of review#1, the agreed #345 correction sets
optimize=false only at the glyph builder's reverse-flow call in C/Rust.
The shared helper and its other callers are unchanged. The constructed
40-scroll case now autonomously builds25 actual engine Runes and returns
to a center with24 surrounding Runes by turn177; C twice and Rust match all
logs and full final saves. The reverted caller fails to complete by500.
The native C stop observer confirms map25/tracked25/Morgoth position true
without changing normal byte streams. This is controlled preparation, not
a naturally earned character, combat benefit or win. See
validation/2026-11-07-post-m4-investigation for the source/fixture scope.

Independent harness budgets/stops are opt-in: key limits bound stair-reroll
loops and Rune-pattern stops observe engine glyphs only. Living-save skips
dead characters while preserving real save errors; mandatory finalsave stays
strict. These hooks do not change ordinary GUI behavior. Self-lunal strategy
is diagnosed but not rewritten; the stock setting remains off.


## Independent M5 qualification of recognition evidence

The [M5 review](https://github.com/diegoami/borg-rs/issues/2#issuecomment-6711741153)
independently discriminates #295 through the native observer for known Rune/
decoy and knowledge/trap controls. Of the six recognition scenarios, only the
prelaid ring changes normal gameplay keys/messages/state/final-save bytes when
#295 is reverted on the M5 source. The other five matching gameplay traces do
not independently prove recognition; their discriminating evidence is the
observer's map/tracking state and its verified noninterference. This clarifies
the evidence channel without rewriting the original source-qualified receipts.
