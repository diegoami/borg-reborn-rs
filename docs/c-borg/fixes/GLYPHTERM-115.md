# source115: drop the always-zero glyph count from the outlying rune score (patch 0126)

Consumer adoption of [source 115](https://github.com/diegoami/neo-angband-tracker/issues/115),
fix `87323a76f` (branch `task/glyphterm-115`), kit `docs/audit/c-reports/glyphterm-115` from the evidence branch `task/glyphterm-115-evidence`.

## Change

Drop the always-zero glyph count from the outlying rune score.

When borg_flow_glyph() extends an existing Sea of Runes it scores each
unglyphed grid of the 5x5 block around the stored centre as
500 + (tmp_glyph * 500) - cost.  tmp_glyph is only counted in the
new-sea branch, so in this branch it is always 0 and the score is
500 - cost: the borg glyphs the nearest unglyphed grid next.

Write the score as 500 - cost.  No behaviour change; it only makes the
nearest-first choice explicit.

Source `87323a76f` -> applied `ec7f388fd` (`cherry-pick -x`), patch `patches/0126-glyph-score-term.patch`, scope `src/borg/borg-flow-glyph.c` only.

## Verification

Before commit `236780604` (the kit commit, `src/` identical to `origin/main`), tip `ec7f388fd`, kit
unchanged (`tools/verify-glyphterm115.py`). Native gates green at both; byte-wise provenance over the source diff; the
`src`/`lib`/build scope is the one file. This is a behaviour-preserving change, so the kit has no
fail-before mode: every mode passes in both phases with the recorded outcome.

| Mode | Before | After |
| --- | --- | --- |
| `offcenter` | passes, `{"progress": 1}` | passes, same outcome, **byte-identical** |
| `ring` | passes, `{"progress": 1}` | passes, same outcome, **byte-identical** |

The committed `before-*.log` / `after-*.log` are the source lane's captures. All product logs are byte-identical to the source lane's captures. The
source's own rows and hashes are in `receipt.json`, carried unchanged.

## Mode attribution

Per the 2026-11-10 owner rule, each configuration is stated. This is a behaviour-preserving cleanup
(removes code the source shows is unreachable or a no-op), so no configuration (unrestricted, munchkin/Lunal-only, normal-only) can observe a
difference. That rests on the source's reachability argument above and on the kit's identical
before/after outcomes; it is **not** a cohort measurement.

Matrix group `glyphterm-115` (2 cases) uses the `after-*.log` rows as its baseline.
