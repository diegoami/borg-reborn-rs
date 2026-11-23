## 2026-11-23: The never-seen gate keeps a landing site's stricter tolerance (#1094)

Every check below ran on the **default core, borg-rs** (`borg-core/`). No
check ran on the reborn core: this change does not touch it. Base: track-p
`d347f42d`, which merged #1075 (#1066). Branch:
`claude/affectionate-babbage-4hl2h5`, restarted from that base.

### What was done

This is the P3 of Codex's last re-review of #1075
(https://github.com/diegoami/borg-reborn-rs/pull/1075#issuecomment-7089738987).
The gate that #1075 added reuses the never-seen count of a landing check
from the same think and grid. It serves the sites without a landing check,
such as the critical phase a refused emergency falls through to. Until now
it judged that count by its own `escape_phase_unknown_tolerance`. So an
emergency at 50 that refused 60 never-seen landings could be undone by a
gate at 75.

- `st.borg.phase_unknown_seen` now also records the landing site's
  tolerance (`caution_phase`'s `emergency`).
- `gate_count` returns the count and the tolerance to judge it by. On a
  reuse that is the stricter of the site's and the gate's. On a fresh draw
  it is the gate's own, as before.
- `unknown_gate` takes its tolerance from `gate_count`. The refusal note
  names the tolerance used.
- `escape_phase_unknown_tolerance`'s description and `docs/CALIBRATION.md`
  (#1066's section) say so. No parameter is added.

### Found

At the compiled defaults the emergency and the gate are both 75, so the
re-review's case does not arise there. But the landing sites' tolerances
differ: 2.2 is 50, 3.1 is 25, and 4.1 and 5.1 are 20. Under the policy, a
twitchy or fast-fragile gate later in a think that already ran those checks
is now stricter. The critical and last-ditch gates follow the emergency
check directly, so they keep 75. Both switches ship 0, so today's play is
unchanged.

### Checks

- The CI commands (fmt, clippy, the workspace tests, `cargo test --release
  -p borg-core` with 725 passing), `scripts/check-param-citations.py`,
  `scripts/borg-core-vendor.py check` and `git diff --check` all pass.
- The new test `the_stricter_tolerance_carries_into_the_fall_through` uses
  the reviewer's case:
  - 60 never seen, with the emergency at 50 and the gate at 75: refused at
    50;
  - a looser site (90) does not loosen the gate;
  - another think, or no record, uses the gate's 75.

  The earlier tests now pass the tolerance through `gate_count`.
- Smoke: 4/4 identical to `harness/reference/smoke-borg-rs`.
- Replays on the zoo's `206cf4e7` with #1066 and this change, from the
  bundles of run 45070528012:
  - the control still reproduces both zoo games key for key;
  - `escape_phase_unknown_scary = 1` plays the same games as on #1075's
    head (`48cb61f0d30ebb6f`, `1d138dd085db21e1`), so the stricter
    carry-over changed neither.

  These are leads, not measurements.

### Review of #1096 (Codex, GPT-6, on `fcafaaaa`)

The findings are in https://github.com/diegoami/borg-reborn-rs/pull/1096#issuecomment-7090495604.
It confirms the #1094 fix: with 60 unknown landings, the fallback keeps the
emergency's 50. It adds one P3, fixed in the commit after `fcafaaaa`.

**[P3] A gate refusal must leave the twitchy think's longer-range
teleports.** The twitchy Phase Door's cast also tries Dimension Door,
Teleport Self, Portal and Shadow Shift. A never-seen refusal returned before
the whole cast, and this PR's stricter tolerance makes that refusal more
likely. The gap was already in #1066 at any tolerance.

- `phase_door` clears `st.borg.phase_gate_refused` on entry. It sets it to
  the think only when the gate refuses.
- The twitchy think then tries those four, in their original order
  (`longer`), outside the decision. `borg_allow_teleport` still holds, and
  a gate refusal implies no teleport curse.
- Any other refusal (the curse, or the #584 rule) still stops them, as
  before. So this changes nothing with the policy off. Moving them out
  unconditionally would have let the rule's refusals reach them at the
  shipped settings.
- Test: `a_gate_refusal_leaves_the_twitchy_teleports` pins the sources (it
  needs the engine to play). The flag is cleared before any refusal and set
  only by the gate. The twitchy think uses it, keeps the order, and keeps
  the four inside the cast after the Phase Door ways.
- The other gated sites cast only Phase Door, except last ditch, whose
  `PORTAL` (as in C) stays inside the decision. The review did not raise
  it.

Not replayed: the change acts only with the policy on, after a gate refusal
in the twitchy think. 726 tests pass.

### Left, and for the owner

- #1073: the zoo comparison of #1066's switches and the Cemetery
  counterfactuals (unchanged).
- Still open from #1066: how strict the never-seen count should be, and
  whether the next-turn veto should read the danger with fear.
