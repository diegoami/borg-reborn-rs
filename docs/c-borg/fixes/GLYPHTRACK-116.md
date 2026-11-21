# source116: report a rune read on a grid that is already tracked (patch 0119)

Consumer adoption of [source 116](https://github.com/diegoami/neo-angband-tracker/issues/116),
fix `d0164206a` (branch `task/glyphtrack-116`), kit `docs/audit/c-reports/glyphtrack-116` from
the evidence branch `task/glyphtrack-116-evidence`.

## Defect

When the Borg stands on the grid it chose in the Sea of Runes flow, `borg_flow_glyph()` reads a
Rune of Protection (or casts or activates the equivalent) and tracks the new rune. If the grid was
already in the glyph tracker it returned false although the read had just been queued:

```c
for (i = 0; i < track_glyph.num; i++) {
    /* Stop if we already new about this glyph */
    if ((track_glyph.x[i] == borg.c.x) && (track_glyph.y[i] == borg.c.y))
        return false;
}
```

The caller treats false as "nothing done" and goes on to its remaining flows in the same turn, so
their keypresses are queued behind the read. A complete sea makes the Borg return to its centre,
which is always tracked, so this is the ordinary way to reach it.

## Fix

`return false` becomes `return true`: the read is the action. Source `d0164206a` -> applied
`f41782f79` (`cherry-pick -x`), patch `patches/0119-glyph-read-tracked.patch`, scope
`src/borg/borg-flow-glyph.c` only (one line).

## Verification

Before commit `cc61542fb` (the kit commit, `src/` identical to `origin/main`), tip `f41782f79`, kit
unchanged (`tools/verify-glyph116.py`). Native gates green at both (965/965 unit, 5/5 frontend);
byte-wise provenance over the source diff; the `src`/`lib`/build scope is the one file.

| Mode | Before | After |
| --- | --- | --- |
| `complete-sea` (Human Warrior, seed 116, level 50, depth 99, lit arena, centre rune already tracked, potion three grids east) | **fails** 0/1: the read is queued and a step towards the potion follows (`second_rc=0`, `extra_step=1`) | passes 1/1: only the read is queued (`second_rc=1`, `extra_step=0`) |
| `new-sea` (first arrival at a new sea centre) | passes, control | passes, **byte-identical** |

The committed `before-*.log` / `after-*.log` are the source lane's captures; the product runs of
this tree (`product-result.json`) are byte-identical to them for all four logs, so the kit reproduces
unchanged. The source's own rows and hashes are in `receipt.json`, carried unchanged.

## Mode attribution

The branch runs only when the Borg reads a Rune of Protection on a Sea of Runes grid that is already
in the glyph tracker, which is a late-depth path. No mode gate is involved, so it is reachable in
unrestricted, munchkin/Lunal-only and normal configurations alike once the Borg gets there. No
cohort or zoo arm reaches it (the class-grid runs finish near depth 3), so the per-configuration
effect is recorded as **not measured**, not neutral, and belongs to the batch with the zoo's
explicit arms (borg-zoo issue 54).

Matrix group `glyphtrack-116` (2 cases) uses the `after-*.log` rows as its baseline. Full matrix
at this tree: 97 groups / 294 cases, `ok: true`, no failed group, 241/254 byte-identical; the 13
byte differences are in the same six groups as before this carry (`dispel`, `dispel259`,
`fleeheal-408`, `knockout`, `preparation`, `vampire`), so no new difference. Running it needed the
tracker's fixture commits (fetched by sha into the object store).

## Ledger and credit

`patches/series.json` `GLYPHTRACK-116`, patch-active; consumer issue #482. Source defect, fix and
native fixture by Diego Amicabile (native proposal/evidence); carried unchanged.
