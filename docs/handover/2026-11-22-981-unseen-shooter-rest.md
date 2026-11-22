## 2026-11-22: An unseen shooter's hit holds the rest (#981)

Every check below ran on the **default core, borg-rs** (`borg-core/`,
`harness/borg.txt`); none on the reborn core.

**Reproduced.** Elf Warrior seed 1312323730 (`-race 2 -class 0`) on
track-p `4b2c2fcf` plays key for key the zoo's game (`7243fba61d12f4b2`),
with the zoo report's overrides (borg-zoo#668's reproduction section).

**Cause (checked in the engine and the code).**
- "You are hit by something sharp!" is the engine's message for a bolt that
  hits the player from a monster that is not visible (or while blind):
  `project-player.c:798-799, 840-842, 886-888`. The kobold archer
  (`monster.txt:1655`, `spells:ARROW`) was out of the light, not invisible.
- The Borg reads the arrow's "You hear a soft twang." as
  `SPELL_005:Something`. That goes to `borg_locate_kill`, which notes
  "Invisible monster nearby." and creates no monster, and to regional fear.
  No monster, so `borg_danger` is 0, and `borg_recover`'s danger and
  fear-region tests pass.
- The #697/#831 invisible-attacker mode is fed only by blows
  (`HIT_BY:`/`MISS_BY:` → `note_unseen_attack`, `borg_update.rs`). A bolt
  never entered it: the replay has no `# Mode:` line. So the mode exists on
  this core, but nothing told it about a shooter.

**Fix, behind `recover_unseen_shot` (default and shipped 0).** With 1, the
engine's message is tagged `SHOT_BY:Something` and read in
`handle_late_messages` (where the Borg's grid is current) as an unseen
attack of a ranged episode (`unseen_ranged`): the existing rest hold, walk
away and leave rules and the heal below `recover_unseen_heal_hp_percent`
apply. A shooter gets no blind attack on the adjacent grids and no Detect
Invisible; the room is lit first if `borg_check_light_only` can; a revealed
monster's death ends the episode.

**Checked.**
- Replay, switch off: `7243fba61d12f4b2`, the same death.
- Replay, `borg_param_recover_unseen_shot = 1`: `ec9a2eb4f0450e55`. The
  first shot enters the mode; the first step away shows the archer, two
  flasks of oil kill it, the mode ends, the Warrior rests. It reaches the
  100,000-turn cap at CL8, DL3. One game; it is not a measurement.
- `smoke-borg-rs`: 3/3 identical. CI's cargo commands, `check-param-citations`,
  `borg-core-vendor.py check` and the python suites pass. Unit tests:
  `unseen_shot_messages`, `the_death_of_a_revealed_shooter_ends_the_mode`.

**Not done.**
- The replay records are not in borg-cemetery. This session could not get
  write access to borg-cemetery, and the branch the task named
  (`runs/2026-11-22-trackp-9085-full-deaths`) is not on its remote. The
  session kept the replay files, and the PR gives their hashes and the
  command.
- The zoo has not measured it. That needs a borg-zoo request with
  `recover_unseen_shot = 1` (the owner starts it, RULES 20).

**For the owner.** Whether to ship `recover_unseen_shot = 1` after the zoo
measures it. #982 (stacked on this PR) is a separate change in the same
function.
