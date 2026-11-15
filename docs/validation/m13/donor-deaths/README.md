# Rust Borg death records

Every confirmed natural death gets an immutable source/build/input identity, original observations, an assessment and linked actions. Full raw runs/saves/builds stay outside Git. Death alone does not establish a bug.

> **The canonical registry lives in
> [borg-cemetery](https://github.com/diegoami/borg-cemetery)** — recorded
> Borg runs, written death investigations and playable saved scenarios.
> Historical copies can remain local; temporary registries are isolated
> validation evidence. See [POINTER.md](POINTER.md) for the seam
> (`BORG_DEATH_RECORDS_DIR`) and storage ownership.

Dossiers: 12; distinct starting conditions: 12; retained process attempts: 12 (including repeats). This registry is not a survival-rate estimate.

| Case | Discussion | Actual source | Seed | Raw cause | Death turn | Diagnosis |
| --- | --- | --- | --- | --- | --- | --- |
| [D0001-rust-seed1011](D0001-rust-seed1011/report.md) | unassigned | `1443c2d1b97d` | 1011 | a cave spider | 18903 | unassessed |
| [D0002-rust-seed1020](D0002-rust-seed1020/report.md) | unassigned | `1443c2d1b97d` | 1020 | a small kobold | 9547 | unassessed |
| [D0003-rust-seed1021](D0003-rust-seed1021/report.md) | unassigned | `1443c2d1b97d` | 1021 | a fruit bat | 30109 | unassessed |
| [D0004-rust-seed1022](D0004-rust-seed1022/report.md) | unassigned | `1443c2d1b97d` | 1022 | a giant yellow centipede | 14120 | unassessed |
| [D0005-rust-seed1023](D0005-rust-seed1023/report.md) | unassigned | `1443c2d1b97d` | 1023 | a metallic red centipede | 13184 | unassessed |
| [D0006-rust-seed1043](D0006-rust-seed1043/report.md) | unassigned | `1443c2d1b97d` | 1043 | a wild cat | 23423 | unassessed |
| [D0007-rust-seed1050](D0007-rust-seed1050/report.md) | unassigned | `1443c2d1b97d` | 1050 | a grey mold | 24096 | unassessed |
| [D0008-rust-seed1062](D0008-rust-seed1062/report.md) | unassigned | `1443c2d1b97d` | 1062 | a soldier | 49147 | unassessed |
| [D0009-rust-seed1072](D0009-rust-seed1072/report.md) | unassigned | `1443c2d1b97d` | 1072 | a giant white mouse | 40785 | unassessed |
| [D0010-rust-seed1073](D0010-rust-seed1073/report.md) | unassigned | `1443c2d1b97d` | 1073 | a gallant | 94857 | unassessed |
| [D0011-rust-seed1074](D0011-rust-seed1074/report.md) | unassigned | `1443c2d1b97d` | 1074 | poison | 22250 | unassessed |
| [D0012-rust-seed1084](D0012-rust-seed1084/report.md) | unassigned | `1443c2d1b97d` | 1084 | a grey mushroom patch | 22908 | unassessed |

## Aggregated actions

Each run that dies is exported by tools/export-death.py (harness/run.sh). Curate assessment.json and report.md by hand; regenerate this index with `python3 tools/death_registry.py`; rerun a case with its logs with tools/rerun-death.py. Original-C defect leads go to neo-angband-tracker under the repository policy; no automatic issues or gameplay changes are made by the logger.
