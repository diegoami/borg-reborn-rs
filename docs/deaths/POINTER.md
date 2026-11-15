# The death registry now lives in borg-cemetery

The death dossiers this repository used to hold under `docs/deaths/` are
recorded in **[borg-cemetery](https://github.com/diegoami/borg-cemetery)**,
which is the canonical home for recorded Borg runs, written death
investigations and playable saved scenarios.

- Registry and case index:
  <https://github.com/diegoami/borg-cemetery/tree/main/deaths>
- The cross-project index of every Borg death we know of (this
  repository's, better-borg-c's and borg-rs's) is still generated here by
  `scripts/refresh-sources.py --update`, in
  [`reference/deaths/INDEX.md`](../../reference/deaths/INDEX.md).

## The case ids collide — the other session must resolve this

**Our registry and the cemetery both use ids from `D0001`, and they mean
different deaths.** Merging the two directories naively would overwrite
the cemetery's cases with ours.

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

Two ways out, for the session doing the move to choose:

* **Namespace ours** (for example `reborn/D0001-c-seed102`), keeping the
  historical ids where they are; or
* **Renumber ours** into a free range, with a mapping recorded in the
  cemetery's index so an old id still resolves.

Either way, the mapping must be written down before the move, and the
`encounter_family` in each `record.json` is the durable identity that
survives a renumbering.

This pointer does not decide it: the move and the reconciliation belong
to the session doing them.

## The registry generator must not overwrite the cemetery's own index

`tools/death_registry.py` writes `index.json` and `README.md` **into the
registry root** (`update_index`, `tools/death_registry.py:118-125`). Its
root is `BORG_DEATH_RECORDS_DIR`, default `docs/deaths/`.

If that variable is pointed at a **cemetery checkout**, the generator
would overwrite the cemetery's own `deaths/index.json` and
`deaths/README.md` — which hold the cemetery's historical cases, its
aggregated actions and its own index — with a regeneration from the
local registry. That is the hazard issue #273 names ("Do not let the
native registry generator overwrite cemetery-owned historical indexes").

**This pointer does not change that behaviour**: the write path and its
defaults are untouched, and the routing decision belongs to the session
working #273. What this pointer does is make the registry's canonical
home explicit, so the routing work has one place to hang it on.

Two safe shapes, for whoever implements the routing:

* the generator writes only into a registry it owns (the local
  `docs/deaths/`, or a per-run staging directory), and the cemetery's
  index is never a target; or
* the generator is told explicitly which file it may write (an output
  path, not a registry root), and refuses to write into a root it does
  not own.

**The index formats are identical, so the generator cannot tell the two
registries apart by their contents.** Both `index.json` files carry the
schema `borg-rs/death-index@1` and the same key set (`actions`, `cases`,
`causes_by_distinct_start`, `death_process_attempts`,
`distinct_starting_conditions`, `limit`, `schema`, `versions`). The
cemetery inherited borg-rs's format and ours derives from the same one.
A content check will therefore not distinguish them, and the guard has
to be about *which directory* the generator was pointed at, not about
what it finds there.

The same applies to `scripts/refresh-sources.py`, which writes
`reference/deaths/INDEX.md` — that file stays ours, and its rows for our
cases link to the cemetery rather than to a local copy.

## Why the code stays here

The Borg, its engine integration and its harness stay in this repository
([docs/SPEC.md](../SPEC.md)); the cemetery holds the *evidence* they
produce. Writing a dossier still happens here: `tools/export-death.py`
runs at the end of every run that died and writes the dossier, and
`tools/reconcile-deaths.py` is the CI gate that fails when a measured
run's natural death has no dossier (RULES 12).

Every tool resolves the registry through one seam, so the same code
serves both homes:

| Variable | Meaning |
| --- | --- |
| `BORG_DEATH_RECORDS_DIR` | the registry directory a run writes to and the tools read from |
| default | `docs/deaths/` in this checkout |

`tools/export-death.py` (`registry_dir`), `tools/death_registry.py`
(`registry_root`), `tools/reconcile-deaths.py` (`--registry`) and
`scripts/refresh-sources.py` all honour it. CI already writes its
dossiers to a temporary registry
(`BORG_DEATH_RECORDS_DIR=$RUNNER_TEMP/deaths`) and reconciles them
against the committed one.

## What this means for a session

* **Do not assume a dossier is in this repository.** Look in
  `borg-cemetery`'s `deaths/` for the canonical copy; the case id
  (`Dxxxx-rust-seedNNN`) is the same in both.
* **A new death is written here first.** Let `tools/export-death.py`
  write it into the registry directory, then it is copied to the
  cemetery by the reconciliation step. Both copies carry the same
  `record.json`, the same `files_sha256` hashes and the same
  `encounter_family`.
* **RULES 12 still binds.** Every natural death of a measured run is
  recorded, with its actual source, build, implementation, profile and
  seed, and CI fails when one is missing. Moving the registry does not
  relax the rule; the reconcile gate is the same gate.
* **The case ids stay stable.** A dossier's id is its identity; moving
  it does not renumber it.

## Related

* [`reference/deaths/INDEX.md`](../../reference/deaths/INDEX.md) — every
  Borg death we know of, with the encounter families.
* [`docs/research/DEATHS-AND-ISSUES.md`](../research/DEATHS-AND-ISSUES.md)
  — the digests by subsystem.
* [`tools/README.md`](../../tools/README.md) — the tooling that writes and
  checks the registry.
