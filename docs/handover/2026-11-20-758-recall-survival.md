## 2026-11-20: flight Recall survival guard (#758)

Separate change on `task/T758-recall-survival`, stacked on #763's
`28670e85b691d72e507f7e516be4ab19793aa0f6` (#760/#759), including accepted
main `ec0f3b28` and the exact-observed-HP and fractional-timing refinements, as requested.

- `caution_recall_survival` is compiled/shipped 0. Its named horizon,
  `caution_recall_survival_delay`, is 34 world ticks (the engine maximum),
  a conservative hypothesis. World ticks are converted using energy;
  they are not assumed to be one action at every player speed.
- The guard sums damage from awake known reachable monsters during the
  delay, after approach time. Reachability uses remembered terrain and
  known movement/door abilities. It reads no unseen engine map or future
  Recall random roll. Unknown terrain is permitted conservatively.
- Caution calls `borg_recall_for_flight`. On this base the “Leaving
  (failed to teleport)” branch in borg_escape sets flight flags and then
  reaches the same caution read; it does not independently read Recall.
  Both requested paths therefore share the gate. Ordinary trip-home
  `borg_recall` calls do not use it.
- Read #582/#583/#754/#641, including comment threads. This changes no
  pending-Recall, light-wait or exploration policy from those issues.
- Unit tests cover speed/world time, approach delay, blocked/open routes
  and immobile monsters at range/adjacent. Rust checks passed locally.

## Migration / resume state

The owner considered moving to another computer, then chose waiting here. All
of our pending lock requests were cancelled; no enabled local games had
started. Other sessions' jobs were left untouched. The parent
[task note](2026-11-20-760-fight-fragile.md) is the authoritative resume and
transfer inventory for the whole task; read it first.

Previous CI-checked code pin: `6cdd214bbc7a65ba96dbae21ad88fd9ecdb6eb1b`.
[CI 44758476887](https://github.com/diegoami/borg-reborn-rs/actions/runs/44758476887)
is green, including shipped-off/reference smoke and all required checks.
The later Haste/free-fox follow-up requires CI on the latest head. The
earlier pin already established tested shipped-off/default neutrality.

Still required under #758: enabled before/after cases, the paired cohort,
canonical Cemetery packages/report and review. Keep
[PR #764](https://github.com/diegoami/borg-reborn-rs/pull/764) a draft.
The branch now includes observed monster Haste and the native free fox
revert before weapon attacks. Reachability and damage both include Haste;
Slow is conservatively not credited as persisting. Validate these in the
large cohort and keep runtime script pins current. No improvement has been measured.

A mobile caster with a possible route is given no approach discount,
because it can gain line of fire before arriving in melee. An immobile
caster still requires current projectability. The timing model reads the
player's own displayed speed and uses world ticks rather than assuming
one action per tick. These are conservative modelling choices to validate,
not owner-approved tuning/adoption decisions.

The private script `/tmp/T760-fight/recall_trials.sh` prepares immutable
runtime/profile copies and runs six controls, Recall-only, fights-only and
all-switches cases. Its expected build pin must match the current head.
Run under `flock /tmp/angband-validation.lock`; preserve existing outputs,
using new directories for any repeat iteration. The named baseline six
already have exact canonical key/state/message matches before implementation.

No reviewer has been named and no merge is authorized. #757/#761 remain
another session's work; do not touch them.
