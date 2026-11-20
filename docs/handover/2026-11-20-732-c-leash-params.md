## 2026-11-20: the C Borg's leash as parameters (#732)

Owner, 2026-11-20: "I would remove our leash completely and expose the leash
of C with configuration parameters".

**What was done:**
- The owner's own leash (PR #734: `flow_leash`, 15/25/40/60 steps by level,
  growing with XP) is **dropped**. This branch starts from main, and #734
  is closed in its favour. The zoo had found it neutral (borg-zoo#475, starts
  1-360: 6 → 7 deaths, CL −0.14).
- C's leash numbers are ten parameters, each defaulting to C's literal and
  read once in `borg_flow.rs` (`c_leash`):
  - `borg_get_leash` (`leash_with`, unit-tested): `flow_leash_per_level`
    (3), `flow_leash_base` (9), `flow_leash_unbounded` (250),
    `flow_leash_pickup_max_level` (20), `flow_leash_depth_margin` (5) and
    `flow_leash_twitch_after` (21);
  - `borg_flow_far_from_stairs_dist`: `flow_leash_depth_margin` and
    `flow_leash_stairs_max_level` (20);
  - `take_select`: `flow_leash_take_near_max_level` (10),
    `flow_leash_take_nearness` (5) and `flow_leash_take_max_level` (20).
- Everything else is C's, as before.
- `harness/borg.txt` lists them at C's values, so a zoo profile can move
  any of them. CALIBRATION has the section.

**Checked:** see the PR (play identical to main at the defaults: smoke and
classes-18).

**Left / for the owner:** which numbers to try in the zoo. The obvious ones
are `flow_leash_per_level` / `flow_leash_base` (how far) and
`flow_leash_depth_margin` (how long the leash stays on as the Borg levels).
