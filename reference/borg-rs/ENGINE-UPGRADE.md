# Upgrading the vendored Angband engine

Prospective procedure only: **no upgrade has happened or is scheduled.** The
vendored upstream engine stays pinned at `e473720ed88a0f0c7ca66815cef3000eb4a38bc3`
(see [FORK-CORRECTIONS.md](FORK-CORRECTIONS.md), [REVIEW.md](REVIEW.md) and
[C-CONSUMER-ADOPTION.md](C-CONSUMER-ADOPTION.md)) until the owner deliberately
authorizes a new target. This document exists so that when that happens, the
work follows a known procedure instead of being improvised.

## Parity is not the acceptance bar for this upgrade

Owner decision, 2026-11-10: byte-identical keypress parity with C was
feasible while the engine was pinned to one fixed version, but is too much
of a limitation to carry forward across engine upgrades. For the engine
version this document covers, parity against a C build of that version is
**not** the acceptance bar. This does not relax parity for the currently
pinned engine (`e473720ed`) or its better-borg-c milestone adoptions, which
keep following the rule in `CLAUDE.md`.

What replaces parity as the acceptance bar is **not yet decided** — this
document records the mechanical steps an upgrade still needs regardless
(FFI resync, reconciling FORK corrections, regression coverage), but the
actual go/no-go criteria for a specific upgrade are for the owner to set
when that milestone is opened.

## Why this isn't a routine update

Even without a parity requirement, moving the pin isn't a simple dependency
bump: `docs/c-bugs.md`'s sections cite exact lines in the pinned source,
the FFI headers in `borg-rs/src/headers`/`borg_headers` are hand-mirrored
from it, and `docs/FORK-CORRECTIONS.md` records inline edits made directly
to the vendored files. All of that is tied to the exact pinned commit and
needs reconciling against whatever the new source actually contains, so
this is treated with the same weight as a better-borg-c milestone adoption
(see [C-CONSUMER-ADOPTION.md](C-CONSUMER-ADOPTION.md)), not as a dependency
bump.

## Preconditions

* The owner names an exact target commit (or tagged release) of upstream
  Angband, not "latest" or a moving branch head. A tracking issue is opened
  for it, in the style of the C-milestone issues (`borg-rs#N`).
* The current M5/C-milestone work is at a stable checkpoint; this is not
  combined with an in-flight C-milestone adoption.

## Procedure

1. **Add an upstream remote.** The original import (`git log` shows
   `Squashed 'angband/' content from commit e473720ed` merged as
   `angband`) used `git subtree add --prefix=angband <upstream-url>
   e473720ed --squash`. No remote for it is kept in this repository, so add
   one before pulling, e.g.:

   ```sh
   git remote add angband-upstream https://github.com/angband/angband.git
   git fetch angband-upstream <new-ref>
   git subtree pull --prefix=angband angband-upstream <new-ref> --squash
   ```

2. **Resolve against FORK corrections.** Every entry in
   [FORK-CORRECTIONS.md](FORK-CORRECTIONS.md) is an inline edit to a vendored
   file, so the subtree merge will surface each as a normal merge conflict
   (or a silent, unflagged divergence if upstream touched the same function
   without conflicting). For each one: confirm whether upstream already
   fixed the underlying bug, whether the correction still applies verbatim,
   and whether its linked tracker issue (neo-angband-tracker or
   `neo-angband-tracker#474`-style engine issue) still describes the current
   behaviour. Do not carry a correction forward unexamined just because it
   merges cleanly.

3. **Re-sync the FFI boundary.** Diff the new `angband/src/borg/*.h` and
   related engine headers against `borg-rs/src/headers` and
   `borg-rs/src/borg_headers` for changed struct layouts, field order,
   enums or function signatures the Rust Borg relies on. Update `engine.rs`'s
   safe wrappers to match before anything else will build.

4. **Re-baseline `docs/c-bugs.md`.** Its sections cite specific lines in the
   `e473720ed` source. Re-locate each cited bug in the new source: same
   logic at a new line (update the citation), changed/fixed upstream (record
   that explicitly, do not silently delete the section), or genuinely gone.
   Do not renumber past sections; append corrections the way milestone notes
   already do elsewhere in that file.

5. **Rebuild the harness against the new engine.** `harness/reference/*`
   holds keypress logs captured from the old pinned engine, and is no
   longer a pass/fail gate once parity stops being the goal — but a fresh
   C build at the new pin is still useful as a diagnostic baseline to tell
   "the Rust Borg changed behaviour" apart from "the engine changed
   behaviour underneath it" when something looks wrong.

6. **Rebuild Rust against the new engine and run the suites.** The goal is
   no longer byte-identical match against C; it's that the Rust Borg still
   builds, still runs without crashing or regressing obviously, and that any
   genuine behavioural divergence from the old pin is understood and
   deliberate rather than accidental fallout from the header/FORK-correction
   resync in steps 2–3.

7. **Treat it as a milestone, not a merge.** Publish a bounded
   receipt/checkpoint the way the C-milestone adoptions do (see
   `docs/adoption/c-m*/README.md` for the pattern): target commit, what
   changed in the engine, which FORK corrections were kept/dropped/updated,
   and the actual acceptance criteria the owner set for this upgrade (since
   it isn't parity — see above). Get an independent review on the tracking
   issue before the owner accepts it.

8. **Update every doc that hardcodes the old pin.** At minimum: `README.md`,
   `docs/REVIEW.md`, `docs/HANDOVER.md`, `docs/C-CONSUMER-ADOPTION.md`
   ("The current official engine base remains e473720ed...") and
   `docs/FORK-CORRECTIONS.md` ("Pinned upstream stays e473720ed"). Update
   them together with the acceptance, not piecemeal beforehand.

## What "done" looks like

* A new annotated tag marking the accepted engine base, following the same
  pattern as the `m5` / `4.2.6-impact-vN` tags.
* `docs/STATUS.md` records the acceptance the way it records each C
  milestone, including the (non-parity) criteria the owner actually
  accepted it against.
* Every doc in step 8 refers to the new pin consistently; no stale
  `e473720ed` references remain outside historical/superseded sections that
  are explicitly marked as history.
* `docs/c-bugs.md` and `CLAUDE.md`'s keypress-parity rule are updated to
  make clear they describe the *previous* pinned engine's history, not a
  live requirement for the new one.
