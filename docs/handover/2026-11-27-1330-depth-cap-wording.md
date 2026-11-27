## 2026-11-27: Say what the depth caps do (#1330)

### What was found

The two depth caps have different meanings, and their texts didn't say so:

- `borg_no_deeper = N`: `borg_prepared` refuses depth >= N and
  `borg_leave_level` climbs only from below N (C: `borg-prepared.c:571`,
  `borg-think-dungeon-util.c:874`), so the Borg stays at DL N-1 at most.
  `harness/pborg.yaml` said it "stays at or above that level".
- A plan's `max-depth N` installs the limit N + 1, so the Borg may stand on
  DL N, but the reason read "No deeper N+1.".

### What was done

- `harness/pborg.yaml`: the comment on `borg_no_deeper` says N - 1 at most.
- `prepared_with_limit`: the plan's reason reads "Plan max-depth N.". The
  configured cap keeps C's "No deeper N." The requirement record
  (`plan_depth_limit`, limit N + 1) is unchanged.
- The plan test checks the text.

### Checks

CI's four cargo commands, `scripts/tests`, `check-param-citations.py`.
No decision changes: the reason is a note, and the smoke oracle compares key
streams (`harness/diff` does not read the config hash). Games under a plan
note the new text.

### What is left

Nothing for this task. The rest of the audit: #1326–#1329, #1333; owner
decisions #1331, #1332.
