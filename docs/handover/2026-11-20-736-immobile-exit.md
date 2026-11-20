## 2026-11-20: immobile monsters and blocked stairs (#736)

Owner: the key-by-key replays of #717's Druid deaths show the step-away is
not the fault. It steps the wrong way, the stairs walk ignores the acolyte
in the way, and the attack targets the eye, not the pursuer. "I would put
them already on."

- **Code** (all compiled 0 = C, **shipped 1**):
  - **A, `caution_stairs_clean_path`** (`borg_caution.rs` `desperate_flow`,
    `borg_flow_stairs.rs` `stair_path_blocker`):
    - "Desperate for Stairs" first walks the desperate flow's path. An awake
      monster on or next to it, as dangerous as
      `caution_stairs_block_danger_percent` (50) of the HP, makes the stairs
      not reachable.
    - The probe spreads as `desperate_flow` does (desperate flag set; the
      first try without it never reached the Borg). It restores the flow
      state (the first try without that changed later decisions).
  - **B, `caution_immobile_step_to_exit`** (`immobile_step_away`): the step
    away takes the grid with the fewest steps to a staircase, then the least
    danger. It acts only with `caution_ranged_vs_immobile` 1–3 (shipped 0
    since #737).
  - **C, `attack_mobile_threat_first`** (`borg_attack`): while a mobile
    target is a threat (`attack_mobile_threat_percent`, 50, of the HP),
    immobile monsters are not targets.
- **Checked** on the three Druid deaths only modes 2 and 3 have
  (borg-zoo#461), replayed on #717's head `5af2f79f` with each part patched
  in:

  | Druid | mode 2 | mode 2 + A | mode 2 + A, B | mode 2 + A, B, C | mode 0 + A, B, C |
  |---|---|---|---|---|---|
  | 718471242 (eye, acolyte) | dead | alive | alive | alive | alive |
  | 1896767101 (mushroom patch) | dead | dead | alive | alive | alive |
  | 1508143544 (white worm mass) | dead | — | alive | alive | alive |

  - classes-18 on main with mode 0, A and C on against off: deaths 0 and 0,
    sum of max CL 104 → 106, sum of max DL 37 → 41, 11 of 18 identical.
  - fmt, clippy `-D warnings`, `cargo test --release -p borg-core` (483),
    the vendor check, citations (2216).
- **smoke-borg-rs:** identical to the reference with the three on (`scripts/rerecord-smoke-borg-rs.sh`: nothing to re-record).
- **Found on the way (Fox Form):**
  - Both CL1 Druids had learned Detect Life, not Fox Form. C's study table
    rates Fox Form 5 against Detect Life's 95 (`borg_magic.rs`, "need to know
    when to cast any of the shapechanges").
  - Patched to learn Fox Form first, with `escape_fox_form = 1`, all three
    Druids live in mode 2 without A, B or C.
  - The owner: "finish 736 first and then do the fox". That is the next
    issue.

**Left:** the zoo pilot (borg-zoo); the Fox Form study and flight (next
issue; #694 holds the Fox Form switch).
