# source593: check teleport landing zones with either kind of teleport (patch 0144)

Consumer adoption of [source 593](https://github.com/diegoami/neo-angband-tracker/issues/593),
source pair [#606](https://github.com/diegoami/neo-angband-tracker/issues/606),
fix `edc4dcb3f` (branch `task/tportcheck-593`), kit `docs/audit/c-reports/tportcheck-593` from the
evidence branch `task/tportcheck-593-evidence` (generators byte-identical to the fixtures run-state
pin `bcc2c86aef`; the captured artifacts were added by later evidence-branch commits; the
interpreter `__pycache__` directory is not carried).

## Change

The teleport landing-zone check required both a teleport source (`BI_ATELEPORT`) and an escape
staff (`BI_AESCAPE`) before it would examine landing zones, while its callers in the Destruction
and Teleport Level defences ask whether the Borg has **either** kind. With teleportation scrolls
but no escape staff the check refused, and the Borg read the Scrolls of Destruction next to the
danger instead of checking for a safe landing. Examine landing zones with either kind.

Source `edc4dcb3f` -> applied `c9b018c8f` (`cherry-pick -x`), patch `patches/0144-tportcheck-either.patch`,
scope `src/borg/borg-escape.c` only (3 insertions, 2 deletions; applies at a +2-line offset over
the module's carried corrections including 0139, byte-wise over the layered result).

## Verification

Before commit `382acb7b9` (the kit commit, `src/` identical to `origin/main`), tip `6940a0f88`, kit
unchanged (`tools/verify-tportcheck593.py`). Native gates green at both (965/965 unit checks and
5/5 frontend tests at the before commit and at the tip). Byte-wise provenance: the source diff
applied to the before module equals the product module, and the `src`/`lib`/build scope of the fix
and of before..tip is the one file. The verifier compiles each kit phase against that phase's own
build tree (the per-phase include-roots pattern from the source589 verifier).

| Mode | Before | After |
| --- | --- | --- |
| `scrolls` (teleportation scrolls, no escape staff, danger 2000) | fails: the landing check refused — the Borg reads the Scrolls of Destruction next to the danger (`landing_ok=0`, `uses_destruction=1`) | passes: the landing zone is checked and no Destruction scroll is spent (`landing_ok=1`, `uses_destruction=0`) |
| `control` (has both kinds) | passes: landing checked, no Destruction read | passes, same outcome, **byte-identical** |

The committed `before-*.log` / `after-*.log` are the source lane's captures. All four product
logs are **byte-identical** to them (both phases, both modes) and the STATE/OUTCOME rows match
exactly. At the accumulated matrix, two unrelated groups gain one new diagnostic line at this
tip — losecon-61 (2 cases) and deepdescent-590 (1 case) each add
`TRACK # No Teleport. scary squares: 100`: their fixtures hold teleportation scrolls without an
escape staff, so with this fix the landing-zone check now *runs* where it previously refused at
the gate, and logs its assessment (no safe landing found). Decisions are unchanged — the
OUTCOME rows are identical and both groups pass; this is the fix's intended either-kind behaviour
showing up in other scenarios, recorded per the dangerfull-69/healamount-592 pattern (both groups
join the baseline re-point sweep). The per-case comparison is recorded in `product-result.json`,
and the source's own rows and hashes are in `receipt.json`, carried unchanged.

## Mode attribution

Per the 2026-11-10 owner rule, each configuration is stated. The landing-zone check runs in the
Destruction/Teleport-Level defence paths in every configuration — unrestricted (all modes),
munchkin/Lunal-only and normal-only — so the changed branch executes in all of them; the kit
exercises it through the production defences with the either-kind state the callers ask about, and
records the has-both control where the corrected gate is unchanged. The per-mode cohort effect of
checking landings before spending Destruction is **not measured**, not neutral; the batch
measurement with explicit arms is owed to borg-zoo issue 87.

Matrix group `tportcheck-593` (2 cases) uses the `after-*.log` rows as its baseline and is
byte-identical in both cases. Full matrix at this tree: 121 groups / 371 cases, `ok: true`, zero
failed cases, 316/371 byte-identical; the differences are the declared intentional transitions
(12) plus the new `No Teleport` TRACK lines in losecon-61 (2) and deepdescent-590 (1) documented
above — the fix's either-kind gate now running in those scenarios. No unexplained difference.
