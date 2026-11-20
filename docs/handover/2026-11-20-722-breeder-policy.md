## 2026-11-20: one breeder policy (#722)

Owner: "so many breeder settings"; "assume the level is lost if you could not kill all the breeders"; "search for stairs first, but if paths are blocked by breeders, recall at some point"; "maybe use the opportunity to see if you can start the engine with more breeders".

- **Analysis** (on #722): what the 20 breeder settings decide, and which are dead with stoppable on:
  - the class hunt counts and `caution_breeder_fight`;
  - the avoidance and exit sub-settings while their switches are 0;
  - `recover_breeder_hunt_hp_percent` with no-rest on.
  - Leads: a paused stand can latch the "ignore and leave" state for the level; a blind Borg skips its mana rest to hunt.
- **Change:** `caution_breeder_policy` (-1 = switches as set, compiled; 0..5 by layer; **shipped 3**), applied in `params::value`.
  - At 3 and above: the level is lost above the class count; a lost level is left and searched for stairs; Recall after `caution_breeder_lost_recall_turns` (500).
  - Unit tests `breeder_policy_tests`.
- **Test bed:** `-start-monsters NAME:COUNT[:DIST]+...` in the frontend (`main-borgh.c`), `start-monsters=` in `run.sh`, recorded in `summary.json` and `config.txt`.
- **Measured** (CALIBRATION, #722):
  - -1 is identical to main 19/19.
  - Policy 3: deaths 2 against 1 (neither from breeders), CL 111 and 111, DL 43 against 40.
  - smoke-borg-rs: identical at the shipped 3 (on main with #721, which ships its switches; the full-level and Recall rules do not fire in smoke), so not re-recorded.
  - The test bed as tried (CL1 fresh characters) kills nearly every character under every policy and tells nothing yet.
- **Left:**
  - The test bed needs a captured build (#382's `start-build`).
  - Then compare 3, 4 and 5 there, and in a zoo pilot (borg-zoo#460 measures the switches).
  - The two leads above.
  - Retire the six switches' names once nothing reads them as set (they stay readable now).
