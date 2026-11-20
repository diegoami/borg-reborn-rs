## 2026-11-20: ship Recall wait-as-usual and Fox Form (#694)

Owner (2026-11-20, on #694's first switch): "I would adopt it".

- **Change:** `harness/borg.txt`, `borg_param_caution_recall_wait_as_usual`
  0 → 1, with the zoo evidence in its comment. While Word of Recall counts
  down on DL2 or deeper, the Borg plays by its ordinary rules: it flees by
  stairs or Teleport Level, uses its usual escapes and attacks, and treats a
  scary level as usual. It still rests or waits in place when safe. In town
  and on DL1, C's Recall rules (the buy-time block of `caution_recall_*`)
  still apply.
- **Evidence:** zoo-neutral everywhere, and rarely reached:
  - borg-zoo#400 (1,800 starts, 3ba17e2c): 103 deaths against 105;
  - `reborn-recall-usual-200k-main` (72 starts at 200k, 13e1187c): the same 4
    deaths, 17 games changed, CL −0.04, DL −0.05;
  - borg-zoo#428 (108 starts, e5cc5469): 2 against 2, 3 games changed.

  Adopted as the better way to play (zoo RULES 18: neutral, safe to adopt
  if preferred).
- **smoke-borg-rs:** `scripts/rerecord-smoke-borg-rs.sh` found the games
  3/3 identical to the reference, so nothing was re-recorded. No smoke game
  waits for Recall on DL2+.
- **Checked:** the smoke suite against the reference on main 6f03fea9 merged
  in. No code changed.

- **Fox Form** (owner, 2026-11-20: "adopt fox form both switches"):
  `escape_fox_form` 0 → 1 (a fleeing Druid casts Fox Form, #551) and
  `escape_fox_form_town` 0 → 1 (on long walks in town, #620). Both are
  zoo-neutral: borg-zoo#400, 1,800 starts, 105 deaths against 105, only Druid
  games change; borg-zoo#413 small, 90 starts, 4 against 4. smoke-borg-rs is
  still 3/3 identical (it has no Druid), so nothing was re-recorded.

**Left (#694):** `attack_town_dangerous_on_sight` stays 0 for now; the owner
hasn't decided it.
