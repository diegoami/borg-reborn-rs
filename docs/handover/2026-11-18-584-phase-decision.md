## 2026-11-18: one Phase Door decision, a refactor with no change in play (#584, step 1)

Owner: "refactor first so the decision for Phase Door is comprehensible",
then the new rule. This is the refactor.

- **One decision point:** `phase_door(st, site, check, danger, cast)` in
  `borg_escape.rs`. Every place that reads, casts or activates Phase Door
  calls it: 13 levels of `borg_escape`, the Recall wait (`borg_caution`),
  the two twitchy paths (`borg_flow_misc`, `borg_think_dungeon`), 16 sites
  (`PhaseSite`). Each keeps the check it had (`PhaseCheck`: the landing test,
  only #539's "a monster next to me", Shoot N Scoot's, or none) and its own
  spell/scroll order, so the random draws and the keys are unchanged.
- **One note per decision**, naming the site: `# Phase (<site>): no, …`,
  `landing ok`, `done, HP h/H, danger d, nearest awake monster n`. It
  replaces `# Safe to Phase` / `# No Phase …`. `borg_caution_phase` is gone
  (its body is `caution_phase`).
- **docs/PHASE-DOOR.md:** every site, when it fires, its check and its
  settings, and what the terms mean (danger, avoidance, the landing test).
- **`harness/borg.txt`:** the comment on the six `escape_phase_*` settings now
  says they gate only escape level 3.1. Comment only; values unchanged.
- **Citations:** every parameter citing one of the four touched files now
  points at the first line that reads it (`Param::<name>`); 335 were moved,
  many already stale on main.

Checked: 18/18 classes-18 starts (borg-rs core, 100,000 turns, shipped
`borg.txt`) play the same keys as main `dbe99509`; smoke 3/3; CI commands.

What the new notes show in those 18 games (Phase Doors done): escape 4.1
compromised 40, escape 5.1 low level 31, escape 1 emergency 30, twitchy think
25 (no check), escape 2.2 9, escape 1.2 6, escape 3.3 4, escape 4.3 2,
escape 7.1 2, escape 3.1 1. Levels 4.1 and 5.1 fire below the HP (0.6 × and
0.5 ×).

Left (#584, step 2): the owner's rule behind a switch, compiled off: Phase
Door only with an enemy on an adjacent square and the fight judged lost;
otherwise the best attack by magic or missiles, else a boost (Speed,
Heroism, Bless, …). Then a zoo pilot.
