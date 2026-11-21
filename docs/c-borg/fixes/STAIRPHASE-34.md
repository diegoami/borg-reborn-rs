# source34: take the stairs alone when a stair escape is chosen (patch 0100)

Consumer adoption of [source 34](https://github.com/diegoami/neo-angband-tracker/issues/34)
(CBUG-0016), fix `2176cf71` (source PR 573, evidence PR 572), kit
`docs/audit/c-reports/stairphase-34` at fixture commit `eb91a6756`.

## Defect

Five escape steps - `borg_escape()` danger levels 3.1, 4.1, 5.1, 6.1 and 8 - tested the up
staircase inside the Phase Door condition:

```c
if ((borg_escape_stair() || borg_caution_phase(...)) && ...) {
```

`borg_escape_stair()` queues the climb key `<` and returns true, but the rest of the condition
still ran. If a phase was possible its keys were queued after the `<`; if not, the `<` stayed
queued and the routine could go on to queue a teleport. The game then consumed a key stream the
borg had not planned, and the borg did not simply take the stairs it had chosen.

## Fix

Make the staircase its own branch, in all five steps:

```c
if (borg_escape_stair() || (borg_caution_phase(...) && ...)) {
```

Source fix `2176cf71` -> applied `293c0d71d` (`cherry-pick -x`), patch
`patches/0100-stairphase-escape.patch`. Scope: `src/borg/borg-escape.c` only.

## Verification

Before commit `e06a84ce4`, tip `293c0d71d`, kit used unchanged
(`tools/verify-stairphase34.py`). Both native gates green: 965/965 unit and 5/5 frontend at the
before commit and at the tip; byte-wise provenance (source diff applied to the before module
equals the product module); `src`/`lib`/build scope is the one file.

| Mode | Before | After |
| --- | --- | --- |
| `upstair` | **fails** 0/1: `<` queued, then a phase read on the new level (`commands=4`, `phase_read=1`, `scroll_read_on_new_level=1`) | passes 1/1: climb only (`commands=1`, `phase_read=0`, `scroll_read_on_new_level=0`) |
| `antisummon` | **fails** 0/1: `<` queued, then a teleport read (`teleport_read=1`) | passes 1/1: climb only |
| `downstair` | passes, control | passes, **byte-identical** |
| `floor` | passes, control | passes, **byte-identical** |
| `climb` | passes, control | passes, **byte-identical** |

The committed `before-*.log` / `after-*.log` files are the product runs; the source kit's own
recorded rows are in `receipt.json`. One product difference is disclosed: on this product the
`downstair` and `floor` controls read a teleport where the source kit records a phase (the
carried item/spell set differs), and the control rows are equal before and after the fix with
byte-identical logs, which is what the verifier pins.

Matrix group `stairphase-34` (5 cases) uses the `after-*.log` rows as its baseline and passes
5/5. Tooling: 144 tests (`python3 -m unittest discover -s tools -p "test_*.py"`).

## Mode attribution

The escape routine runs in every configuration, and the fix is unconditional (no mode gate).

| Configuration | Result |
| --- | --- |
| **Unrestricted (all modes)**, frozen v1 (seeds 11/29/47, 20k turns) | **neutral**: alive at cap, same hp and depth, traces **byte-identical** before and after (issue #427) |
| **Munchkin/Lunal only** | **not measured** |
| **Normal only** | **not measured** |

The unrestricted run is one configuration and does not separate the modes; the munchkin/Lunal and
normal arms are owed to the batch in `diegoami/borg-zoo` issue 54. What the fixture proves is that
the changed branch executes (upstair/antisummon) and that the controls are unchanged.

## Ledger and credit

`patches/series.json` entry `STAIRPHASE-34`, patch-active; consumer issue #427. Source defect,
fix and native fixture by Diego Amicabile (native proposal/evidence); carried unchanged.
