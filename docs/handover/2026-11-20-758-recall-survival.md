## 2026-11-20: flight Recall survival guard (#758)

Separate change on `task/T758-recall-survival`, stacked on #763's
`03e95ba4858099f10268599c3f24c2b266f033fe` (#760/#759), as requested.

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

Still in progress under #758: enabled before/after cases, paired cohort
validation, canonical Cemetery references and smoke/default neutrality.
Update this note with outcomes before marking the PR ready. The baseline
six deaths already reproduced on clean `08f94e1f` before either change,
with exact supplied-control key streams.

Owner review and merge remain pending. Do not merge without the owner's
word. #757 and #761 remain another session's work and are untouched.
