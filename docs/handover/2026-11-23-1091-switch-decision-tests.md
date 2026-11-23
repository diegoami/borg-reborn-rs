## 2026-11-23: Decision tests for the immobile-toucher switches (#1091)

**Done.** The four switches of #931, #1023, #933 and #934 (PRs #965–#968) are
now covered by tests that fail when their rule is made to do nothing. That was
the MiniMax review's finding 5 on each PR. Tests and a mechanical refactor
only: play is unchanged.

- **The refactor** (`borg_caution.rs`). Each rule's decision moves into a
  function that takes its switch values as arguments. Parameters are fixed at
  init, and the race flags, melee and danger need the engine, so the release
  tests could not reach the rules before. The order of evaluation is kept: a
  later fact is still only asked when the earlier ones hold.
  - #965: `attacker_next_to` (the kill-list scan, with the race test passed in).
  - #966: `undone_run`, `confirm_landing`, `record_move_away` (the run state
    machine over `BorgMonInfo`).
  - #967: `lone_neighbour`, `immobile_neighbour_need` (the danger is asked only
    for a short fight, as before).
  - #968: `escape_first_applies`.
- **The tests** (`switch_decision_tests`, 11 tests).
  - The decisions with the switch values on, from the 829474344 numbers: the
    jelly's reach, the loop vetoed at the fourth attempt, progress or an
    unconfirmed landing never vetoed, the window edge, a lone recent neighbour,
    the affordable two-turn kill, and the escape-first conditions.
  - Source pins in the house's shape (#1011, #1015): each wrapper reads its
    switch before its decision; the stair flight refuses the reach in both
    searches; `borg_caution`'s order is landing → #968 → #966 → #967 → retreat
    → recording; the back-away site is wired too.

**Checked.**
- CI's commands and the citation check (2444 parameters, 0 wrong).
- Mutation: each of `in_immobile_reach`, `retreat_oscillation_veto`,
  `immobile_neighbour_veto` and `poisoned_escape_first` replaced by `false`
  fails a test. It left the suite green before.
- Smoke at the shipped settings: 4/4 identical to `smoke-borg-rs`.
- Smoke with all four switches on (reach 1, limit 3 with back-aways, kill 3,
  poison 75), this branch against `track-p` `57d47863`: 4/4 identical. #932's
  rule fires there once.
- Twelve 20,000-turn games starting next to white, yellow and green jellies
  (`start-monsters`), switches on: identical keys and messages on both builds,
  but no rule fired in them. So the refactor's equivalence is checked by
  construction and by smoke, not by a game in which #931, #933 or #934 act.

**Left.** A named-game replay with the switches on (829474344 acts on all four)
needs the four PRs ported to the zoo's `7f8c6b80`; not done here.

**Owner.** Merge into `track-p` after the review (RULES 1a).
