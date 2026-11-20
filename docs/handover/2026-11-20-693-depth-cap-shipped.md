## 2026-11-20: the depth cap shipped, with values per class (#693)

Owner: "can we have explicit borg_param_prepared_depth so on values in each
class", then the per-class table, then "switch goes 1".

- **Values** (`harness/borg_<class>.txt`, folded as
  `borg_param_prepared_depth_*__<class>`), div / knee / below:

  | class | div, knee, below | CL1–5 | CL6–10 | then |
  |---|---|---|---|---|
  | Druid | 5, 10, 4 | DL1 | DL2 | CL11 DL7, CL15 DL11 |
  | Mage, Priest, Necromancer | 4, 8, 3 | DL1 to CL4, DL2 to CL8 | | CL9 DL6, CL12 DL9 |
  | Ranger, Rogue | 3, 6, 2 | DL1 to CL3, DL2 to CL6 | | CL7 DL5 |
  | Paladin, Blackguard | 3, 5, 1 | DL1 to CL3, DL2 to CL5 | | CL6 DL5 |
  | Warrior | 2, 4, 0 | DL1 to CL2, DL2 to CL4 | | CL5 DL5 |

  The common values apply to no class.
- **Shipped `prepared_depth_plan = 1`.** The switch is one common line, so one
  line (or a zoo arm) turns it off for every class.
- **smoke-borg-rs re-recorded** (`scripts/rerecord-smoke-borg-rs.sh`): 0/3
  identical to the old reference. No smoke game dies; the Warrior stays on
  DL1 (max depth 2 → 1).
- **Not measured with these values:** the local check of #701 (200k,
  classes-18) used the old casters/fighters split. With the switch on,
  borg-zoo#446's ctl arm is now this shipped plan; the request needs
  re-pinning to main after merge, with an arm for the plan off.

**Known shape of the formula** (owner: keep it): after a long DL1–2 phase
the cap jumps (the Druid from DL2 at CL10 to DL7 at CL11; the casters DL2 → DL6
at CL9). The Borg's own preparation checks still decide how deep it goes
under the cap.
