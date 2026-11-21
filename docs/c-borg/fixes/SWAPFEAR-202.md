# source202: record fear protection on swap weapons and armour (patch 0132)

Consumer adoption of [source 202](https://github.com/diegoami/neo-angband-tracker/issues/202),
source pair [#582](https://github.com/diegoami/neo-angband-tracker/issues/582),
fix `c5eae07cb` (branch `task/swapfear-202`), kit `docs/audit/c-reports/swapfear-202` from the
evidence branch `task/swapfear-202-evidence` (generators byte-identical to the fixtures run-state
pin `27a567ee553`; the captured artifacts were added by later evidence-branch commits; the
interpreter `__pycache__` directory is not carried).

## Change

`borg_notice_weapon_swap()` and `borg_notice_armour_swap()` reset and set the protection flags of
the candidate swap from its object flags, but `weapon_swap_resist_fear` and
`armour_swap_resist_fear` were never reset or set. The swap valuation's fear-protection bonus
(+5500 when the Borg has no fear protection of its own) therefore never applied, and a backup
weapon that protects from fear — a *(Holy Avenger)*, an *of Fury* weapon, or one of the 26
artifacts with fear protection — was valued as if it gave nothing. Reset and set both flags from
`OF_PROT_FEAR` alongside `OF_PROT_BLIND`. The armour half changes no decision at this commit (no
armour valuation reads the flag yet).

Source `c5eae07cb` -> applied `fe4c77c38` (`cherry-pick -x`), patch `patches/0132-swap-fear-flags.patch`,
scope `src/borg/borg-trait-swap.c` only (12 insertions, 8 hunks). The module already carries
0045 (gloves-205) and the swapacid/slayswap corrections; the source hunks apply at a one-line
offset and the provenance check below is byte-wise over the layered result.

## Verification

Before commit `af1f14d1c` (the kit commit, `src/` identical to `origin/main`), tip `6ed12bbfd`, kit
unchanged (`tools/verify-swapfear202.py`). Native gates green at both (965/965 unit checks and 5/5
frontend tests at the before commit and at the tip). Byte-wise provenance: the source diff applied
to the before module equals the product module, and the `src`/`lib`/build scope of the fix and of
before..tip is the one file.

| Mode | Before | After |
| --- | --- | --- |
| `fear-plain` | fails: the fear-protecting Mace is offered for sale (`swap_rfear=0`, swap value 1299, `sells_fear=1`) | passes: the fear-protecting Mace is kept (`swap_slot=4`, `swap_rfear=1`, value 6724, `sells_plus1=1`) |
| `fear-brave` (control) | passes: level-30 BRAVERY, the bonus correctly stays off | passes, same outcome, **byte-identical** |
| `plain-plain` (control) | passes: the +1 Mace is kept | passes, same outcome, **byte-identical** |

The committed `before-*.log` / `after-*.log` are the source lane's captures. All six product logs
are **byte-identical** to the source captures (every phase and mode, including the failing
`fear-plain` before phase), and the STATE/OUTCOME rows match them exactly; the per-case comparison
is recorded in `product-result.json`, and the source's own rows and hashes are in `receipt.json`,
carried unchanged.

## Mode attribution

Per the 2026-11-10 owner rule, each configuration is stated. The swap-notice scan runs in every
configuration — unrestricted (all modes), munchkin/Lunal-only and normal-only — on inventory
changes, so the changed branches execute wherever the Borg notices gear; the kit shows the changed
branch executing (`swap_rfear` flips in the fear-plain mode) and records the level-30 BRAVERY
configuration where the bonus correctly stays off. The per-mode cohort effect of valuing
fear-protecting spares correctly is **not measured**, not neutral; the batch measurement with
explicit arms is owed to borg-zoo issue 54. The source notes this is a valuation change (which
spare weapon is kept and offered for sale); a swap weapon is only carried from deepest level 50.

Matrix group `swapfear-202` (3 cases) uses the `after-*.log` rows as its baseline and is
byte-identical in all three cases. Full matrix at this tree: 110 groups / 346 cases, `ok: true`,
zero failed cases, 283/346 byte-identical; the 23 byte differences are the 13 in the six
pre-existing groups (`dispel`, `dispel259`, `fleeheal-408`, `knockout`, `preparation`,
`vampire`), dangerfull-69 (4), teleotier-176 (4) and wandnames-38 (2) — no new difference.
