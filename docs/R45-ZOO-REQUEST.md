# Zoo request for the Track R4.5 parity pilot

Track R4.5 needs paired native, check and switch results against the owner’s
fixed receiving baseline. The local carry passed all three shipped smoke
starts in both shadow modes. The receiving task requires the exact
45-start suite, with zero check mismatches and 45/45 switch parity.
[borg-reborn-rs #856](https://github.com/diegoami/borg-reborn-rs/issues/856) ·
[Zoo request #792](https://github.com/diegoami/borg-zoo/issues/792)

## Builds and arms

| Arm | borg-reborn-rs commit | Shadow mode |
| --- | --- | --- |
| Native | `24be98732748230062882fe2b7d9601dba5e2cc0` | `off` |
| Check | `83e28cd4c3fb9a3a0eb4cb75e0ff8d635f25a9d2` | `check` |
| Switch | `83e28cd4c3fb9a3a0eb4cb75e0ff8d635f25a9d2` | `switch` |

The candidate commit is reachable from the published rebuild branch.
Build the candidate once for both shadow arms, using the same GCC right-to-left
argument evaluation as the native control. Use the shipped
`harness/borg.txt` and class and scary-list inputs in all three arms.
The base `borg.txt` SHA-256 is
`dd21cc3e95f5ac0dc504d1f821f148befe356df74d4eb69b17d3b549edc4e046`.
[Rebuild branch](https://github.com/diegoami/borg-reborn-rs/tree/task/T856-r45-rebuild)

## Adapter and evidence

- Add an explicit per-arm selector for `off`, `check` and `switch` and set
  `BORG_SHADOW` in the child after the adapter clears inherited `BORG_*`
  settings. Record the effective mode with the cell identity.
- Retain keys, messages, state, summary, child exit status, effective
  configuration hash and the build receipt for every arm. For check and
  switch, also retain `shadow.jsonl` and `shadow-stats.jsonl`. The
  frontend derives `BORG_RUN_DIR` from each cell's absolute `-summary`
  path. A local probe verified that passing only `BORG_SHADOW=check`
  enables both streams.
- Retain the partial output of a failed cell, including a replay stop before
  the summary. For the parity check, stage successful receipts in
  `harness/suite.sh` layout. Each arm has its build `manifest` and
  completion marker. Each named cell has `config.txt`, `run_exit`, and the
  raw files above. Preserve the result and build identity from each shard
  while staging. Reconcile natural deaths in Cemetery.
- In the pinned Borg Rust adapter, raw key, message and state logs can be
  retained with the keep path. The child environment needs the explicit
  mode after ambient cleanup. [Pinned Zoo adapter](https://github.com/diegoami/borg-zoo/blob/7b373a702e7b9bb491ba468d97d34cf99ec3df2d/zoo/borgrs.py)

## Pilot and exit gate

Start with the owner-dispatched adapter pilot on the three starts in
`harness/suites/smoke`, subject to Zoo’s pilot rules. Check effective
settings, compare the raw streams and demonstrate retention of a failed
cell before requesting the 45-start run. Estimate shard time from the pilot.
Local shadow smoke took about four minutes per arm.

The exit slice uses the exact named seeds, race and class indices, caps
and menu letters from `harness/suites/full` (SHA-256
`7c81c28c03be3e6c188cb555fa8c917402bae0819d0791a1a810e031214d2eed`).
Keep all attempts and report incomplete cells. Apply the receiving gate
to the staged artifacts from a checkout that has the checker.

```sh
harness/check-r45-suite.py --candidate 83e28cd4c3fb9a3a0eb4cb75e0ff8d635f25a9d2 \
  retained/native retained/check retained/switch
```

For the three-start rehearsal use `--suite smoke`. A pass requires exact
paired gameplay logs and meaningful summaries in both shadow arms, along
with mode-correct, nonempty statistics and zero mismatches in every cell.
