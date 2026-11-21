# Archive: documents that no longer apply

Each file here was current once and stopped applying. It is kept unchanged
(moved with `git mv` on 2026-11-21, #816; relative links inside it resolve
against `docs/`, its old directory) so that older notes and reviews can
still be read.

| Document | Was | Stopped applying |
| --- | --- | --- |
| [G0.md](G0.md) | the M3 "no machinery failures" gate of our own reborn core (2026-11-10) | the repivot of 2026-11-14 froze the reborn core; the shipped borg-rs core is checked by the smoke references in CI |
| [LESSONS-TRACE.md](LESSONS-TRACE.md) | where each lesson of better-borg-c's REDESIGN-LESSONS is answered in the reborn core's design | same: the reborn core's design is now the target of track R ([reborn-layers.md](../architecture/reborn-layers.md)), and the lessons are tracked per task |
| [ZOO-CHANGE-REQUEST.md](ZOO-CHANGE-REQUEST.md) | the M1 change request to borg-zoo (2026-11-09), filed as borg-zoo#30 | done; the contract with the zoo is [ZOO-INTERFACE.md](../ZOO-INTERFACE.md) |

Also historical but left in `docs/` because code or tests read them by
path: [M4-CALIBRATION.md](../M4-CALIBRATION.md) (cited by
`borg-reborn-rs/src/danger.rs`) and [FIXTURES.md](../FIXTURES.md) (read by
`borg-reborn-rs/tests/decision_fixtures.rs`); both describe the frozen
reborn core. [HANDOVER.md](../HANDOVER.md) is the archive of handover notes
up to 2026-11-18.
