## 2026-11-22: Fast-fragile: the heal floor, a quantile for "the next turn kills", and the Borg's own cut and poison (#926)

**Core.** Every check here ran on the **default borg-rs core** (`borg-core/`,
`harness/borg.txt`), the core the zoo runs. The reborn core is not touched.

**Done.** Three new parameters in `borg-core/src/borg_fast_fragile.rs`, each
0 by default. At 0 they reproduce today's play: both seeds' play hashes and
smoke are unchanged. Declared in `params.rs`, listed in
`harness/borg.txt` at 0, documented in `docs/CALIBRATION.md` (#842 section).

- `fast_fragile_heal_floor` (percent of max HP): at or below it the heal ban
  lifts (`heal_banned`). Attack on sight also gives way to a carried, known
  cure (`yield_to_cure`), after Phase Door and the stairs underfoot.
- `fast_fragile_escape_quantile` (percent; 0 = the mean, 100 = the maximum):
  "the next turn kills" (`escape_instead`, `cornered`) reads that quantile of
  the damage distribution before the Borg's next action, instead of
  `incoming_total`'s mean. The distribution uses the same blows, hit chances
  and speeds as `fight_incoming_with`. It is still compared with
  `fast_fragile_escape_percent` of HP.
- `fast_fragile_count_dot` (flag): adds the Borg's own poison and cut ticks
  expected before its next action (10 / its energy world ticks). When those
  alone take it below 0 HP, it drinks the potion that stops them (CLW for a
  cut up to 20, CSW or CCW deeper, Neutralize Poison or CCW for poison, CCW
  for both) before the attack, Phase Door and the stairs, and the heal ban
  lifts.

**The leads, checked at `4b2c2fcf`** (the base when the task started; the
branch later merged `track-p` `6bc4de1d`, #891/#878, which changes none of
these paths).
- Confirmed: `escape_wanted` compares the mean (`incoming_total` →
  `fight_incoming_with`, `borg_caution.rs:2609-2640`) with HP. The policy
  gate is `hp >= 0` only (`borg_caution.rs:2829-2832`). `heal_banned`
  (`borg_fast_fragile.rs:304` at the pin) has no exception at `= 1`. The cut
  and the poison are in neither (`game-world.c:513-538`).
- The engine's damage in one Borg turn. A wild cat has two CLAW:HURT:1d3
  blows at speed 120 (`monster.txt:1133-1147`). Against a normal-speed Borg
  that is 20 energy to 10 (`game-world.c:68-88`; `mon-move.c:1938-1945`),
  so two actions and four claws. At AC 0 each claw hits 95% of the time
  (`player-attack.c:196-215`). The mean is 7.6 and the maximum 12. At HP 8,
  P(death) = P(≥ 9) = 31.8%. The median, 8, already reaches HP under the
  existing "reaches HP" comparison. The 69th percentile (9) is the first
  that kills.
- **Refuted for 1294425391:** the heal ban is not the cause.
  `fast_fragile_no_heal` = 0, 1 and 2 all play `48c375a0737f6a60`. Its "4
  CLW" are unidentified ("4 Misty Potions", 8372), so `has_cure` finds none.
  That is new issue #1000.

**Replays** (the zoo report's recipe, `-turns 100000`). The receipts are in
the PR; the counterfactual details belong to borg-cemetery (RULES 12) and
are not written there yet.

| switch | 1398933929 (Dwarf Necro) | 1294425391 (Elf Priest) |
|---|---|---|
| all 0 (default) | `9653a0d480ef1dd2` dead 1830 (= pin) | `48c375a0737f6a60` dead 12397 (= pin) |
| `escape_quantile` 50 | `4ed2d1139804b47c` alive at cap, CL7 | same as pin, dead |
| `escape_quantile` 68 / 69 | alive (same game as 50) | not run |
| `escape_quantile` 90 | not run | same as pin, dead |
| `escape_quantile` 96 / 97 / 100 | alive (same game) | `2ffdae32df1762a3` alive at cap, CL8, DL3 |
| `heal_floor` 20 | same as pin, dead | same as pin, dead (no known cure) |
| `count_dot` 1 | same as pin, dead | same as pin, dead |
| `escape_percent` 75 (existing) | `4ed2d1139804b47c` alive | not run |
| `no_heal` 0 / 2 (existing) | 0: same as pin | 0 and 2: same as pin |

1398933929 flips at 1822: Phase Door at HP 8 instead of an oil throw.
1294425391 flips earlier, at 12326: Phase Door at **11/11 HP** against a
maximum of 11. A high quantile therefore spends Phase Door at full HP
against a cat. That is a cost only the zoo can weigh.

**Not checked.** No reproducing seed exercises the heal floor or the
cut/poison ticks: 798380952 plays another game on `track-p` (#926's first
comment). Those two switches have unit tests only. Damage reduction and
armour's cut of HURT damage are ignored, as in the mean.

**Where this meets other sessions.** #980 changes what attack on sight casts
(`borg_fight_attack.rs::attack_on_sight`). This PR does not edit that file,
but the new `yield_to_cure` branch decides there, before the attack. Before
the death at 12396, the 1294425391 game also flees at 2/11 HP for 50 turns
(12346-12386). That is the flight and rest order of #981/#982, not changed
here.

**Review** (MiniMax-M3.1, named by the owner, PR #1002: pass with
findings). Fixed in the PR: (1) the Borg's own ticks are the expectation 10 /
energy, not rounded up; (2) the cut/poison branch drinks the potion that stops
that damage (`dot_cure`), not any cure; (3) the attack-on-sight note names the
actual reason (heal floor or cornered); (4) three engine citations; (5) the
pins: PR body, this note and borg-zoo#672 now name the head on `6bc4de1d`.
After the fixes: 668 borg-core tests pass, smoke 3/3 identical, and every
hash in the table above is unchanged. Left to the owner: (6) the attribution
trailers on the first three commits (RULES 4; later commits have none); (7)
the branch name.

**Owner.** Choose whether to measure these (a borg-zoo request issue):
`escape_quantile` 50 against `escape_percent` 75, which play the same game
on 1398933929, and 96–100; also `heal_floor` and `count_dot`. All are
hypotheses (RULES 14).
