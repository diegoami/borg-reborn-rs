# source192: mark every rubble grid around the borg when the rubble is removed (patch 0141)

Consumer adoption of [source 192](https://github.com/diegoami/neo-angband-tracker/issues/192),
source pair [#628](https://github.com/diegoami/neo-angband-tracker/issues/628),
fix `1abc77849` (branch `task/rubblemark-192`), kit `docs/audit/c-reports/rubblemark-192` from the
evidence branch `task/rubblemark-192-evidence` (generators byte-identical to the fixtures run-state
pin `28a1553da`; the captured artifacts were added by later evidence-branch commits; the
interpreter `__pycache__` directory is not carried).

## Change

The "You have removed the ..." handler in `borg_parse_aux()` is meant to convert the rubble grids
around the borg to broken doors, so a lightless borg does not treat freshly dug ground as rubble
any more. Its loops ran a 2x2 block (y and x from c-1 to below c+1) and the loop body never used
its own x and y: it tested only the goal grid `ag`. With the goal elsewhere, nothing was
converted; with the goal on a rubble grid, only that one grid converted by accident. The
correction walks the full 3x3 block around the borg and converts each grid that is rubble. The
light check keeps its meaning: the whole handler still does nothing while the borg carries light.

Source `1abc77849` -> applied `8a51a8014` (`cherry-pick -x`), patch `patches/0142-rubble-marking.patch`,
scope `src/borg/borg-messages.c` only (6 insertions, 4 deletions; applies at a +14-line offset
over the module's carried corrections, byte-wise over the layered result).

## Verification

Before commit `0b0f63f40` (the kit commit, `src/` identical to `origin/main`), tip `3fb66d3cd`, kit
unchanged (`tools/verify-rubblemark192.py`). Native gates green at both (965/965 unit checks and
5/5 frontend tests at the before commit and at the tip). Byte-wise provenance: the source diff
applied to the before module equals the product module, and the `src`/`lib`/build scope of the fix
and of before..tip is the one file. The verifier compiles each kit phase against that phase's own
build tree (the per-phase include-roots pattern from the source589 verifier).

| Mode | Before | After |
| --- | --- | --- |
| `dark` (lightless Warrior digs five rubble grids; the game's real dig message reaches the parser) | fails: nothing converted (`broken=0`, `rubble=5`) | passes: all five converted (`broken=5`, `rubble=0`) |
| `control` (lit — the handler's light gate) | passes: nothing converted, gate holds | passes, same outcome, **byte-identical** |

The committed `before-*.log` / `after-*.log` are the source lane's captures. All four product
logs are **byte-identical** to them (both phases, both modes) and the STATE/OUTCOME rows match
exactly; the per-case comparison is recorded in `product-result.json`, and the source's own rows
and hashes are in `receipt.json`, carried unchanged.

## Mode attribution

Per the 2026-11-10 owner rule, each configuration is stated. The dig-message handler runs in
`borg_parse_aux()` in every configuration — unrestricted (all modes), munchkin/Lunal-only and
normal-only — whenever the game reports removed rubble, so the changed branch executes in all of
them; the kit exercises it through the production parser with the light gate shown unchanged in
the control. The per-mode cohort effect of correctly clearing dug ground on the borg's map is
**not measured**, not neutral; the batch measurement with explicit arms is owed to borg-zoo
issue 87.

Matrix group `rubblemark-192` (2 cases) uses the `after-*.log` rows as its baseline and is
byte-identical in both cases. Full matrix at this tree: 119 groups / 367 cases, `ok: true`, zero
failed cases, 315/367 byte-identical; the 12 byte differences are the declared intentional
transitions (dispel-248, vampire-257, dispel259, fleeheal-408, preparation) — no new difference.
