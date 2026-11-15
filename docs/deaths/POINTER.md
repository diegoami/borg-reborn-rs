# Death tooling and new reports live in borg-cemetery

[borg-cemetery](https://github.com/diegoami/borg-cemetery) owns the native death
export, registry, rerun and reconciliation implementations, human-readable
investigations, recordings and playable scenarios. The four `tools/` commands
here are compatibility pointers, not another implementation. The Borg, engine,
build, harness and instrumentation remain in borg-reborn-rs.

See [the shared toolkit guide](https://github.com/diegoami/borg-cemetery/blob/main/docs/native-death-tools.md)
and [the cemetery catalogue](https://github.com/diegoami/borg-cemetery/blob/main/deaths/CATALOG.md).

## Running the pointers

Clone/update borg-cemetery beside the primary borg-reborn-rs checkout, or set
`BORG_CEMETERY_DIR` to its checkout. Linked worktrees find that same sibling.
CI checks out the exact revision in `death-tools-pin.txt` into
`vendor/borg-cemetery`; an API-version check rejects an incompatible toolkit.
The existing command paths and arguments still work:

```bash
python3 tools/export-death.py RUN_DIRECTORY --impl rust --build-id build/rust/BUILD_ID
python3 tools/rerun-death.py CASE_DIRECTORY /tmp/rerun --checkpoint-every 100
python3 tools/reconcile-deaths.py TEMP_REGISTRY --registry docs/deaths --runs RUN_DIRECTORY
```

Normal new exports go directly into the cemetery checkout's `deaths/`.
`BORG_DEATH_RECORDS_DIR` explicitly selects another registry, including isolated
CI registries. Reconciliation verifies records; it does **not** copy them to
another repository or publish them. Committing/publishing retained evidence is
a separate delivery step. RULES 12 and the existing CI missing-death assertions
still bind; historical CI comparisons explicitly retain `docs/deaths/`.

## Historical cases have not moved

The existing `docs/deaths/` remains the historical validation archive. It has
not been copied or renumbered. The historical cross-project index at
[`reference/deaths/INDEX.md`](../../reference/deaths/INDEX.md) links to those
actual local records; it must not invent cemetery URLs for unimported cases.

**Both repositories already use the first seven IDs for different deaths.**
A future historical import needs an explicit namespace or renumbering map,
source-qualified identities and preserved encounter-family/key-stream grouping.
Never copy historical directories over the cemetery's existing cases.

| id | ours (`docs/deaths/`) | cemetery (`deaths/`) |
| --- | --- | --- |
| `D0001` | `D0001-c-seed102` — an acolyte at turn 7195 | `D0001-seed8400-warrior` — a pseudo-dragon at turn 37680 |
| `D0002` | `D0002-c-pristine-seed1052` — a wild cat at turn 5021 | `D0002-seed8402-druid` — a cutpurse at turn 5590 |
| `D0003` | `D0003-c-pristine-seed1013` — a fruit bat at turn 9782 | `D0003-seed8406-rogue` — a giant yellow centipede at turn 5600 |
| `D0004` | `D0004-c-pristine-seed1042` — Fang at turn 6416 | `D0004-seed8408-blackguard` — a wild cat at turn 6231 |
| `D0005` | `D0005-c-pristine-seed1023` — a metallic red centipede at turn 5605 | `D0005-recorded` — Fang at turn 36289 |
| `D0006` | `D0006-c-pristine-seed1011` — a small kobold at turn 14534 | `D0006-recorded` — a grey mold at turn 22841 |
| `D0007` | `D0007-c-pristine-seed1043` — a cave spider at turn 23381 | `D0007` — Fang at turn 3640 |

Our ids run `D0001`–`D2358`; the cemetery's run `D0001`–`D0007`. The
cemetery's seven are the historical borg-rs cases (they are the ones in
`reference/deaths/borg-rs/`); ours are the reborn Borg's cases. **Only
the first seven collide**, but those seven are exactly the ones a
directory copy would clobber.

For new exports, allocation uses the shared registry lock and the next free
number in the target registry. Historical local IDs do not imply a cross-repo
identity. Repeats and saved-position branches remain non-independent attempts.

## The cemetery owns its catalogue

The old native formatter rewrites `README.md` and `index.json`. Its formats
match the cemetery's historical files, so a schema check cannot identify safe
ownership. The moved formatter explicitly refuses the canonical cemetery root;
its command delegates to the cemetery registrar, which preserves those original
files and generates `CATALOG.md` and human landing pages under the registry lock.
Temporary registries retain native-format aggregation.

The canonical-home notice introduced by #276 remains in that temporary-registry
template and is covered by the original four regression tests, now also retained
with the owned implementation. No bulk migration or change to gameplay is part
of #273.
