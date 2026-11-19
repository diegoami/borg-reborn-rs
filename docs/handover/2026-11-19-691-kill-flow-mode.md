## 2026-11-19: the hunt removed in steps (#691)

Owner, on #691: "what if we got rid of it?"; "for me this flow kill is a candidate for rewrite"; "we need more of this" (#692 surveys borg-core for the next candidates).

- **Analysis** (comment on #691): the lineage (upstream C, better-borg-c's v11R fix, borg-rs's translation, ours), each check in order with its line, and a notes-only instrumented build on 19 games (keys identical to main). Below CL10:
  - the hunt walks 263 times against about 22,000 exploring steps;
  - the "chase old monsters" call finds a target 516 times and walks 27;
  - the CL16–25 danger gap and the other high-level checks are not measured.
- **Change:** `kill_flow_mode` (compiled and shipped 0 = C).
  - 1: only the targeted hunts (breeders #503/#558, dangerous town monsters #619/#637);
  - 2: no out-of-view call;
  - 3: never hunt.
  - `borg_attack` and the other `borg_flow_kill_*` functions are unaffected. The helper is `kill_flow_mode_hunt`, unit test `kill_flow_mode_tests`.
- **Measured** (CALIBRATION, #691), sum of max CL 108 / 100 / 111 / 104 for modes 0–3. Deaths 0 / 2 / 0 / 1, none of them a hunting decision:
  - an invisible monster touching a resting Borg (#462's pattern);
  - a Borg at low HP searching for secret doors.
  - Mode 0 is identical to main; smoke is identical.
- **Left:** the zoo pilot with the four arms (CL10+ is the open question). Then, from it, either a value to ship or the rewrite. Also, from the analysis:
  - (a) a danger skip for CL16–25;
  - (b) why the out-of-view call does not walk;
  - (c) speed in the two-grid skip;
  - (d) one stairs leash;
  - (e) the literals as parameters;
  - (f) immobile monsters hunted from range.
- **Seen on the way:** `1502695781` rests while an invisible clear icky thing kills it, on the borg-rs core. #462 has the same pattern for the reborn core.
