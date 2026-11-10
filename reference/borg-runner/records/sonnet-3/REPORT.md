# REPORT sonnet-3 (seed 11, Human Mage)

## 1. What happened
Died on depth 1 at character level 1 at ask 48, game turn 1348, killed by a small kobold. Max depth 1.
Files opened before play: docs/BRIEF.md, knowledge/MANUAL-GAPS.md, LEARNED.md, sonnet-1.md, guide.rst (first 150 lines), command.rst (run/rest/throw/study), strategy-manual README, ch10 (exploration), ch14 s2 (mage), ch11 s8, A7 (spell list).

Asks 1-11 character (Human Mage, 334 gold, 10 HP, 2 SP). Learned Find Traps/Doors/Stairs first, cast it in town for 2 XP.
Asks 14-30 shopping: 20 oil, cloak, torch (General, 3/6/6 gold), 6 Phase Door, 5 CLW (Alchemist), Leather Shield [8] 66 gold (Armoury; its prices are far higher than the strategy manual suggests). 6 gold left.
Asks 31-34: DL1. Stair detection (`mac`) showed one `>` about 13 columns W and 6 rows S of the `<`, but my corridor only led north.
Asks 35-39: walked north into a room, picked up a torch. Ask 40: small kobold adjacent in a corridor. Asks 42-43: one flask hit it, then it left view.
Ask 44-45: ran on, gas trap, confused. Ask 47: sent 16 comma-holds in one ask. Ask 48: dead.

Last three decisions: (45) `ssssssssss` which only opened a direction prompt; (46/47) sent `,` x10 then `,` x16 to wait out confusion with a wounded kobold somewhere nearby; (48) screen was the tombstone.

## 2. Decisions I would take back
- Ask 43-44 (turn ~1012): I treated the kobold as gone when its health bar went blank. It had taken one flask and not died (no XP). Instead I should have waited a turn at the corridor mouth and thrown again when it reappeared, or checked with a look/monster list `[`.
- Ask 47 (turn ~1282): batching 16 holds while confused. Instead: one or two holds per ask, or quaff CLW (cures confusion? unverified) / stand still with a throw ready. A 10-HP character has no margin for a blind batch.
- Ask 11: choosing Find Stairs over Magic Missile as the only CL1 spell left me with no spell attack at all. With oil as the sole attack, a missed throw has no backup. Settle with: learn Magic Missile first and gain CL2 before detection, or pick a tougher race/class for a DL1 test.
- Ask 40: with 10 HP, an adjacent kobold in a corridor should have been answered with Phase Door + oil from range, not throwing point-blank.

## 3. Where the manual was wrong or not enough
- MANUAL-GAPS Sheet 1 says a bought item is already named. Partly true: the purchase message says "Scrolls titled ..." but the pack lists them by real name and the shop row loses {unseen} (asks 23-25). So buying identifies, and only floor items need a test.
- Strategy manual ch11 prices (Soft Leather Armour 84, Leather Boots ~34 for the Armoury) did not hold: Armoury at ask 27 had Leather Shield 66, Small Metal Shield 36, Studded Leather 150, nothing cheap for body armour. Different seed's stock.
- Selling gives 0 gold (ask 26, "0 au"), so Word of Recall in the mage kit is dead weight for money.
- The brief's "Mage ~322 gold" was right (334).
- Nothing said `,` is hold (it is; `s` asks for a direction), nor that a `-more-` eats the first key of the next string.

## 4. What I could not see
- At ask 43 the screen showed a blank health bar and no kobold; I could not tell fled/dead/unseen. A monster-list key (`[`) would have settled it; I did not press it.
- The detected `>` was displayed with no connecting floor, and the corridor led away from it; I could not tell whether my route was toward it.

## 5. Leads
- Stair detection at CL1 works on arrival and costs 1 mana; it finds a `>` within the 22x40 box (ask 33-34, turn 701-711). Test: on a fresh mage, cast on arrival at 5 levels and count asks from cast to `>`. Not shown yet: whether the detected `>` is cheaper to reach than blind exploring (I died before reaching it).
- A 10-HP mage dies to a single small kobold if two hits land; level 1 mage at DL1 needs a spell attack as well as oil. Test: `takeover --at-turn 1012` and re-play with `[` before moving on.
- Sending many waiting keys in one ask is unsafe while disabled; the cost is the run (ask 47-48).
