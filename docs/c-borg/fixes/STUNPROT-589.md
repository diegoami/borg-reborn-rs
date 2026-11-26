# source589: model protection from stun separately from sound resistance (patch 0135)

Consumer adoption of [source 589](https://github.com/diegoami/neo-angband-tracker/issues/589),
source pair [#598](https://github.com/diegoami/neo-angband-tracker/issues/598),
fix `3088b91d9` (branch `task/stunprot-589`), kit `docs/audit/c-reports/stunprot-589` from the
evidence branch `task/stunprot-589-evidence` (generators byte-identical to the fixtures run-state
pin `ba9ebcb174`; the captured artifacts were added by later evidence-branch commits).

## Change

Several attacks can stun: gravity, plasma and force breath, water balls and bolts, storms, plasma
bolts and ice bolts. In the game, only protection from stun stops their stun; sound resistance
stops only sound's stun. `borg_danger()` adds a large penalty (+500) when an already-stunned Borg
faces a stunning attack, but for all of these attacks it asked for sound resistance (`BI_RSND`)
instead of stun protection — the penalty was skipped for a sound-resistant Borg (danger 156
instead of 656, under-rating a possibly lethal position at 402 HP) and applied anyway for a
stun-protected one without sound resistance (656 instead of 156). The fix adds a
protection-from-stun trait (`BI_PSTUN`, set from player flags and worn items in
`borg-trait.c`/`borg-trait.h`), uses it for the eight non-sound stun guards in `borg_danger()`,
and makes the sound breath check `BI_RSND || BI_PSTUN`.

Source `3088b91d9` -> applied `e20d372f0` (`cherry-pick -x`), patch `patches/0135-stun-protection.patch`,
scope the three files `src/borg/borg-danger.c` (9 hunks), `src/borg/borg-trait.c` (+`BI_PSTUN`
set) and `src/borg/borg-trait.h` (+`BI_PSTUN`). Both layered modules already carry earlier
consumer corrections; all hunks apply with offsets (borg-danger.c at −1, borg-trait.c at up to
−13) and the provenance check below is byte-wise over the layered result.

## Verification

Before commit `29af41256` (the kit commit, `src/` identical to `origin/main`), tip `37ded9e8c`, kit
unchanged (`tools/verify-stunprot589.py`). Native gates green at both (965/965 unit checks and 5/5
frontend tests at the before commit and at the tip). Byte-wise multi-file provenance: the source
diff applied to the before modules equals the product modules, and the `src`/`lib`/build scope of
the fix and of before..tip is exactly the three files.

| Mode | Before | After |
| --- | --- | --- |
| `rsound` (sound resistance only, stunned) | fails: the penalty is skipped (danger 156, `stun_penalty=0`) | passes: the corrected 656 (`stun_penalty=500`) |
| `pstun` (stun protection only, stunned) | fails: the penalty is applied anyway (danger 656, `stun_penalty=500`) | passes: the corrected 156 (`stun_penalty=0`) |
| `both` (stunned) | passes: 156 | passes, same outcome, **byte-identical** |
| `rsound-unstunned` (control) | passes: 156 | passes, same outcome, **byte-identical** |

With 402 HP the corrected sound-resistance-only value (656) is above the Borg's hit points, so
the position counts as potentially lethal where the original rated it safe. The committed
`before-*.log` / `after-*.log` are the source lane's captures; product logs and STATE/OUTCOME rows
are compared against them by the verifier and recorded per case in `product-result.json`; the
source's own rows and hashes are in `receipt.json`, carried unchanged. The source notes only
gravity is exercised natively; the other seven attacks use the same one-word substitution.

## Mode attribution

Per the 2026-11-10 owner rule, each configuration is stated. The eight stun guards sit in
`borg_danger()`, which feeds fight/flee and target choice for every monster in every
configuration — unrestricted (all modes), munchkin/Lunal-only and normal-only — so the changed
branches execute in all of them; the kit exercises them through the production `borg_danger()`
with the trait combinations above. The per-mode cohort effect of correctly rating stunning
positions is **not measured**, not neutral; the batch measurement with explicit arms is owed to
borg-zoo issue 54.

Matrix group `stunprot-589` (4 cases) uses the `after-*.log` rows as its baseline and is
byte-identical in all four cases. Full matrix at this tree: 113 groups / 354 cases, `ok: true`,
zero failed cases, 290/354 byte-identical; the 24 byte differences are the same known ones as
before this carry — no new difference.
