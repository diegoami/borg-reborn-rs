## 2026-11-18: review of #569 (#539's last part), the scan's scare rule made explicit

Two reviews of #569 accepted #539's last part (`fba1c6e0..4a482c58`, on
`main` without a PR). Fixed here:
- **[major] scan-level tests:** the scare rule of `borg_near_monster_type` is
  now two pure functions in `borg_caution.rs`. `classify_scare` takes the
  named-list, resist-based and damage-rule inputs and the calm; a calmed
  monster keeps only its resist scare. `stand_contribution` gives the stand
  mode's (scary, pursuer). The scan calls them. New tests cover: off = C,
  including when the damage rule is asked and its note; the four calmed
  combinations; and stand mode through to `stand_decision`.
- **The 18 classes-18 starts,** this branch against main `68cf9eb5`:
  identical keys in 18/18 with the switches off, and in 18/18 shipped.
- **"Pursuer"** is explained at `stand_decision`: with one, the Borg stands.
- **Citations moved by #565 and #566:** `caution_scary_stand` `:2447`,
  `caution_scary_latch` `:2463`, the two #539 parameters `:2815`.
- **New issue #572:** a remembered "afraid" keeps a monster calmed after its
  fear has ended (use `kill.seen`).
- **Owner's decision:** RULES 4's AI attribution trailers already on `main`
  (a cleanup pass or a CI guard; published commits cannot be amended, RULES 5).
