## 2026-11-27: the borg-rs core's compiled defaults have a smoke reference (#1357)

Owner, 2026-11-27, recommendation 2 of an audit of the wrapper against
borg-rs: check that the core at its compiled defaults still plays like
borg-rs.

**What was done:**
- `caution_rising_explore_ticks` compiles to 0, which is borg-rs at the pin.
  Its description always said so, but the default was 300. It is still
  shipped at 500, and the shipped smoke is 4/4 identical to `smoke-borg-rs`.
  The rising tests now name their budget (`rising_holds_descent_with`), and a
  new test checks that the hold compiles off.
- `harness/reference/smoke-borg-rs-defaults` is the smoke suite on the
  borg-rs core with `BORG_CORE_OVERRIDES=off`: `borg-core/pborg.yaml`, every
  `borg_param_` at its default.
- CI plays it after `smoke-borg-rs` and diffs it against the reference. A PR
  that changes how the defaults play must re-record it and say why.

**Found:** this reference guards the defaults *as they play now*; it is not
proof of parity with borg-rs. borg-rs's own smoke reference at the pin
(`8c5d000a:harness/reference/smoke`) was made with its C build (`dc790dc`)
on its older engine fork, through its own birth path. Its keys differ from
ours from the first line, and the endpoints differ too:

| run | borg-rs (turns / CL / maxDL / DL / keys) | ours at defaults |
|---|---|---|
| warrior-1 | 20001 / 3 / 2 / 1 / 2616 | 20007 / 3 / 2 / 2 / 2426 |
| mage-2 | 20008 / 2 / 2 / 1 / 1639 | 20007 / 2 / 1 / 0 / 1797 |
| priest-3 | 20879 / 2 / 2 / 0 / 1913 | 20352 / 2 / 2 / 1 / 2102 |

The changes that apply at every setting, listed in the audit, also stand
between the two: unknown items never used (#470), lightless recovery (#811),
healing while fleeing (#369), the stair-flight handoffs (#871, #1052) and
the remove-curse trip (#830).

**Left / for the owner:** a real parity check needs borg-rs at the pin built
on our engine and harness, or each of those changes put behind a parameter.
That is recommendation 8 of the audit, not done here.
