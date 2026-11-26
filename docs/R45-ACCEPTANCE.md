# Owner acceptance of R4.5

On November 26, 2026, the owner accepted the immutable R4.5 snapshot
`e898abb5586135375434ad016e240686c022c53b`. Its tree is
`72bd5c98a16c158f3c8c47a683d8470fe2778591`. Review #1286 merged into
`review/track-r-baseline` at `ea30ebc8c26ced5e3aed32bc29de0b174a57184d`
with the same tree. The Track R integration branch reached the accepted
snapshot through task PR #1095. [Task PR](https://github.com/diegoami/borg-reborn-rs/pull/1095) ·
[Milestone review](https://github.com/diegoami/borg-reborn-rs/pull/1286) ·
[Ledger merge](https://github.com/diegoami/borg-reborn-rs/commit/ea30ebc8c26ced5e3aed32bc29de0b174a57184d)

The Zoo gate used native Rust `24be98732748230062882fe2b7d9601dba5e2cc0`,
candidate `83e28cd4c3fb9a3a0eb4cb75e0ff8d635f25a9d2`, and checker
`610aabe058f34ab155859e6f3802c8ebeb218e71` on engine `574b618c`.
The `check` and `switch` arms each matched all 45 native Rust starts and
recorded zero shadow mismatches. Both distinct natural deaths are retained
in Cemetery. All five checks passed at the reviewed snapshot.
[Full Zoo gate](https://github.com/diegoami/borg-zoo/actions/runs/45685728667/job/139508652763) ·
[Cemetery #158](https://github.com/diegoami/borg-cemetery/pull/158) ·
[Snapshot CI](https://github.com/diegoami/borg-reborn-rs/actions/runs/45768674807)

The reviewed Rust setting `escape_teleport_landing_either=0` requires both
teleport kinds. The C Borg at the pinned engine accepts either kind. The
decision about a later shipped setting has its own measurement request.
[Review addendum](https://github.com/diegoami/borg-reborn-rs/pull/1286#issuecomment-7189689604) ·
[Settings question #733](https://github.com/diegoami/borg-reborn-rs/issues/733) ·
[Zoo #495](https://github.com/diegoami/borg-zoo/issues/495)

The milestone review identified #852's resistance-aware caution behavior as
work for the next main carry. Source-reference maintenance and the corrected
R5 sizing are recorded in the [R5 preflight](handover/2026-11-26-r5-preflight.md).
[Review findings](https://github.com/diegoami/borg-reborn-rs/pull/1286#issuecomment-7188954247)
