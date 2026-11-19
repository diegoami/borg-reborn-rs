## 2026-11-19: avoid breeder colonies, behind a switch (#556)

Owner, 2026-11-18: "maybe in that scenario one should not walk into a room
with breeders then". Started from the latest Cemetery death, Half-Elf Priest
592068863 (borg-cemetery#73; #616's commit `8a427494`, giant white mice,
DL1, turn 86,224), reproduced locally with the dossier's key hash.

**What the replay showed** (more than the case README's reading):
- The up staircase (flow cost 18) lies across the mouse room, so every
  flight step went into the colony and the next think's back-away
  (`back_away`, which runs in `borg_caution` before the flee flow) undid it.
  That stalemate at the room's edge lasted from 50,859 to 84,786 and was
  not fatal by itself (321 mice killed).
- **The fatal step was a Phase Door** at 84,806 (`escape 5.1 low level`):
  the landing check passed ("13 scary landing grids of 100, at most 20") and
  dropped the Priest in the middle of the colony; surrounded in a room
  corner, it never got out.

**What was done:** `caution_breeder_avoid` (flag, compiled 0, shipped 0),
with `caution_breeder_avoid_radius` (1) and `caution_breeder_avoid_turns`
(50 Borg turns):
- `note_breeders` (borg_caution.rs, called after `spread_monster_fear` in
  the monster update) remembers each known breeder's grid with the kill's
  `when`, drops marks older than the memory, and rebuilds a per-grid block
  map; `new_level` clears it. No random numbers.
- `borg_flow_spread` refuses a blocked grid except the grids next to the
  Borg (its first step), for every flow but a kill flow's: the kill flows
  and their digging spread through `borg_flow_spread_hunting`, so #503's
  hunt still reaches breeders. Stairs, flight, desperate flow,
  exploration, items and the breeder exit route around the colony or find no
  route; that also ends the Paladin case's exit/stair alternation.
- `count_scary` (the Phase Door and teleport landing checks) counts a
  landing grid there as scary.

**Checked:**
- Off is neutral: the 18 classes-18 starts play identical keys to main
  `4acf071f` (18/18); the smoke suite is byte-identical to main's (both
  2/3 against `smoke-borg-rs`, see below); the Priest with the switch off on
  `8a427494` plus this change matches the dossier's keys.
- The Priest, switch on (on `8a427494` plus this change): alive at the
  100,000 turn limit (CL5). It diverges at turn 40,334, where exploration
  went around a remembered breeder, so the fatal level is never met the same
  way: a different game, not a replay of the death avoided.
- classes-18, switch on: 15 of 18 games change (breeders are on nearly every
  early level). Deaths 2 against 1: the shared wild-cat death (934061548)
  and a new one, Half-Elf Rogue 1381972077, killed by Grip on DL1 at
  17,046. Its game changed at 14,108: fleeing a scary level, the only route
  to the stairs (cost 38) crossed a white worm mass mark, so the Borg went
  exploring for another staircase instead, and met Grip on the level 3,000
  turns later. The control walked that route and lived.

**For the owner to decide:**
- **A refused only route.** When the colony blocks the only known way to the
  stairs while fleeing, the switch keeps the Borg on the level (the Rogue).
  Options: as now; or, while fleeing, fall back to the plain route when the
  avoiding flow finds nothing (that is the Priest's walk-in again, but only
  when there is no other way); or a larger memory/radius trade-off. The zoo
  should measure it; one local game is a lead (RULES 14).
- **The zoo arm:** the issue's check is an arm on top of
  `caution_breeder_exit` at borg-zoo#240's tier (360 starts): breeder deaths
  are rare.
- **main's smoke reference:** main fails the smoke job since `e0c4a527`
  (`caution_rising_explore_ticks` 300 → 500,
  `caution_scary_damage_percent` 60 → 50) because `smoke-borg-rs` was not
  re-recorded (smoke-mage-2 diverges at game turn 5,703). This PR's engine
  check fails the same way until it is re-recorded.

Not done: the issue's "don't enter" part (exploration not entering a room
where breeders are known) is covered only through the flow block, not as a
room rule.
