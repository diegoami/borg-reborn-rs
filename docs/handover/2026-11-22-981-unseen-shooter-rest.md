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
attack. The message covers every projection (arrows, bolts, breaths, spit;
`projection.txt`'s `blind-desc`), and every projection while blind, so it
says nothing about distance. Revised after Mavis's review (#998):
- The existing rest hold, Detect Invisible, walk away and heal below
  `recover_unseen_heal_hp_percent` apply.
- An episode of hits only (`unseen_shot_only`, set when the episode starts,
  lost to the first blow, cleared with the episode) makes no blind attack
  on the adjacent grids, unless the Borg is blind or has no light. A monster
  in view on a grid the Borg sees is visible unless it is invisible
  (`mon-util.c:359-388`), and Detect Invisible covers that case.
- A hit never counts as "followed": the attacker did not move, the Borg did.
  So a hit gives no reason to leave the level.
- An episode of hits only ends when a known monster that casts or shoots
  dies (`unseen_monster_died`, from `DIED:`), not on any death.
- The first version also lit the room through `borg_check_light_only`. That
  is the C Borg's lighting routine, which darkens the room for a
  Necromancer, so the branch is gone.

**Checked.**
- Replay, switch off: `7243fba61d12f4b2`, the same death.
- Replay, `borg_param_recover_unseen_shot = 1`, after the review fixes:
  `496861d1cf5d1a71`. The first hit enters the mode. The Borg reads Detect
  Invisible twice (nothing found), walks away while it is hit and rests. The
  archer comes into view at 24992, and the Borg kills it with oil. It
  reaches the 100,000-turn cap at CL7, DL5. One game; it is not a
  measurement. (Before the fixes: `ec9a2eb4f0450e55`, CL8, DL3.)
- `smoke-borg-rs`: 3/3 identical. CI's cargo commands, `check-param-citations`,
  `borg-core-vendor.py check` and the python suites pass. Unit tests:
  `unseen_shot_messages`, `a_shot_only_episode_and_its_end`,
  `a_hit_is_not_a_follow`.

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
