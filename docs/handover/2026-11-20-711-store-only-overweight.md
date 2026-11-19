## 2026-11-20: store at home only when overweight (#711)

Issue #711 reported that Cure Serious Wounds was bought, then stashed at
home. It blamed the pack-power rule (`power_inventory_maxclevel_threshold`).

**That cause was wrong:**
- With the threshold at 0 or 5, all 18 classes-18 games play the same keys
  as at 10, with the same 28 CSW stashes.
- With `borg_town_plan = 1`, the live town planner stores the CSW itself:
  `# Town plan executing action 5 at shop 7` (`STORE`).
- The buyers are the planner too. The #410 reserve and the learning
  purchases are off in the shipped `borg.txt`.

**The real cause:** the planner (`dispositions`,
`borg_town_utility.rs`) also stores a stack when its value kept for later
beats its value carried now. "Kept for later" is the value times
`borg_town_home_discount` / 1000 (shipped 500), minus a home slot's cost.
"Carried now" is its value minus its carrying cost. This happens with the
pack light. In 18 games it stored 329 stacks, and 130 were fight
consumables: oil 34, CSW 28, CLW 15, Blessing 21, Boldness 9, Heroism 9,
Phase Door 5.

**Owner, 2026-11-20:** "only when it weighs too much they should stash the
object with less value at home, untile they are not overcarrying again".

**What was done:** `town_store_only_overweight`, compiled 0. **Shipped 1** (owner, 2026-11-20: "it is a no brainer, turn the switch on"); `smoke-borg-rs` re-recorded.
- With it on, a pack stack goes home only while the pack is over its weight
  allowance.
- The pack is already visited lowest keep value per weight first, so the
  least valuable goes first, and the storing ends once the pack is under.
- Taking stacks back from the Home is unchanged.
- Test: `only_an_overweight_pack_stores_the_least_valuable_first`.

**Checked:**
- CI's commands; `check-param-citations.py` (2201); `borg-core-vendor.py
  check`.
- Switch off: the 18 starts play keys identical to main `dbc92d18`.
- Switch on, the same 18 starts (100,000 turns; local leads, RULES 14):

| | off (today) | on |
|---|---:|---:|
| Store actions | 329 | 39, all for weight |
| fight consumables stored | 130 | 25 (oil, over weight) |
| CSW stashed | 28 | 0 |
| purchases | 540 | 462 |
| deaths | 1 (Necromancer 1574344917, blue worm mass, CL7) | 0 |
| sum max CL / DL | 103 / 37 | 109 / 40 |
| oil thrown | 785 | 855 |
| cures quaffed | 74 | 59 |

**On the way:** the home discount alone (borg-zoo#456, now held) was
measured too. At 0, the planner stored 118 stacks, many of them still with
the pack light. At 250 it stored 327. The owner chose the logic change over
the value.

**Left / for the owner:**
- The zoo pilot (borg-zoo#457) measures it; it ships 1 already, by the owner's decision.
- #712 (oil stashed at home) is very likely the same cause: oil stores fall
  from 34 to the overweight ones.
