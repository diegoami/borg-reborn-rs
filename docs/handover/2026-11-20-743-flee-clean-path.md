## 2026-11-20: a clean path for every flight to the stairs (#743)

Owner, 2026-11-20 (closing #689): file the follow-up of #736's clean path
for the ordinary flight to the stairs; "do 743".

**What was done:** `caution_stairs_clean_path_flee`, compiled and shipped 0.
- Every flight to the stairs (`GOAL_FLEE` through `borg_flow_stair_both`,
  `_less` and `_more`: a scary level, too much danger, "Looking for any
  stair") takes a staircase only by a clean path.
- The check is in `commit_and_step`: the flight's own flow, just spread, is
  walked with #736's `path_blocker` before it is committed. An awake monster
  on, or next to, the path, whose danger is at least
  `caution_stairs_block_danger_percent` (50) of the HP, refuses the flight.
- A flight to any stairs then tries the up and the down stairs on their own.
  With none clean, the flight does not start, and the escapes and the attack
  decide.
- Noted once per level: `# Stairs not reachable: X in the way (flight to the
  …)` (`flee_path_noted`, reset on a new level).
- The desperate walk keeps its own check (`caution_stairs_clean_path`, #736);
  this one skips it (`flee_check_applies`). Unit test:
  `only_a_flight_is_checked`.

**Found on the way:**
- **A first version spread its own path map** (`stair_path_blocker`) before
  the flight's flow. `with_stair_spread` does not restore all of the flow
  state (`data_flow`, `data_hard`, `n`, the temp list). With the switch on,
  14 of the 18 starts changed with no refusal at all.
- #736's desperate walk never showed this, because it always flows again
  right after.
- Checking the flight's own flow needs no second spread, so a clean path
  changes nothing.
- `with_stair_spread` itself is unchanged: #736's shipped paths depend on it
  as it is.

**Checked:**
- CI's commands, `check-param-citations.py`, the vendor check.
- smoke-borg-rs 3/3 identical at 0.
- classes-18 through `harness/run.sh … 100000 exact,menu` (Human seeds 1-9,
  Half-Elf 10-18), main's shipped settings, off against on (local leads,
  RULES 14):
  - **18/18 identical.** Two flights were refused (a crow on the path to the
    up stairs; a cutpurse on the path to the nearest stairs), and the next
    keys were the same.
  - Deaths 1 and 1 (the same Druid, a clear icky thing, CL4).
  - The rule acts rarely here; the zoo pilot measures it.

**Left / for the owner:** the zoo pilot (classes-18, then classes-1800
starts 1-90 as the issue asks); watch the casters (borg-zoo#415: "caught
while already fleeing").
