# The baseline of inventory choices

What the current Borg does in town, at real states, so that the new
algorithm can be checked against it. The fixture is the
machine-readable half; this is the readable half.

| fixture | records |
| --- | --- |
| `town-baseline-mage.jsonl` | 126 |
| `town-baseline-priest.jsonl` | 104 |
| `town-baseline-warrior.jsonl` | 71 |

## The runs

Harvested at the control commit `7973c0ea`, with the Rust Borg and
the dump on. The dump is observation only, so the control policy is
today's behaviour and the keypress stream is the same as the C Borg's.

| run | visits | distinct states |
| --- | --- | --- |
| `warrior-201` | 40 | 21 |
| `warrior-202` | 41 | 19 |
| `warrior-203` | 79 | 31 |
| `mage-301` | 64 | 30 |
| `mage-302` | 124 | 57 |
| `mage-303` | 82 | 39 |
| `priest-401` | 83 | 40 |
| `priest-402` | 59 | 31 |
| `priest-403` | 65 | 33 |

Together: **637 visits, 301 distinct (state, choice) pairs, 274 distinct states, 34 distinct choices**.

Each run was:

```sh
BORG_TOWN_DUMP=corpus/warrior-201/desc.jsonl BORG_TOWN_DUMP_INTEGERS=corpus/warrior-201/i32.jsonl BORG_IMPL=rust ./harness/run.sh corpus/warrior-201/out 201 0 0 20000
BORG_TOWN_DUMP=corpus/warrior-202/desc.jsonl BORG_TOWN_DUMP_INTEGERS=corpus/warrior-202/i32.jsonl BORG_IMPL=rust ./harness/run.sh corpus/warrior-202/out 202 0 0 20000
BORG_TOWN_DUMP=corpus/warrior-203/desc.jsonl BORG_TOWN_DUMP_INTEGERS=corpus/warrior-203/i32.jsonl BORG_IMPL=rust ./harness/run.sh corpus/warrior-203/out 203 0 0 20000
BORG_TOWN_DUMP=corpus/mage-301/desc.jsonl BORG_TOWN_DUMP_INTEGERS=corpus/mage-301/i32.jsonl BORG_IMPL=rust ./harness/run.sh corpus/mage-301/out 301 2 1 20000
BORG_TOWN_DUMP=corpus/mage-302/desc.jsonl BORG_TOWN_DUMP_INTEGERS=corpus/mage-302/i32.jsonl BORG_IMPL=rust ./harness/run.sh corpus/mage-302/out 302 2 1 20000
BORG_TOWN_DUMP=corpus/mage-303/desc.jsonl BORG_TOWN_DUMP_INTEGERS=corpus/mage-303/i32.jsonl BORG_IMPL=rust ./harness/run.sh corpus/mage-303/out 303 2 1 20000
BORG_TOWN_DUMP=corpus/priest-401/desc.jsonl BORG_TOWN_DUMP_INTEGERS=corpus/priest-401/i32.jsonl BORG_IMPL=rust ./harness/run.sh corpus/priest-401/out 401 5 3 20000
BORG_TOWN_DUMP=corpus/priest-402/desc.jsonl BORG_TOWN_DUMP_INTEGERS=corpus/priest-402/i32.jsonl BORG_IMPL=rust ./harness/run.sh corpus/priest-402/out 402 5 3 20000
BORG_TOWN_DUMP=corpus/priest-403/desc.jsonl BORG_TOWN_DUMP_INTEGERS=corpus/priest-403/i32.jsonl BORG_IMPL=rust ./harness/run.sh corpus/priest-403/out 403 5 3 20000
```

## The distinct choices

What the control actually chooses, by the kind of the thing chosen. The
slot is not the identity: a shop's stock shifts, so the same kind sits
in different slots in different states.

| choice | target | shop | gold | depth | the Borg's reason | states | visits |
| --- | --- | --- | --- | --- | --- | --- | --- |
| visit | - | Home | 2-354 | 0 | - | 69 | 158 |
| sell_home | Scroll of Word of Recall | Home | 91-354 | 0 | # Selling 'Scroll of Word of Recall' to the home | 9 | 9 |
| sell | Potion of Berserk Strength | Alchemy Shop | 137-149 | 0 | # Selling 'Potion of Berserk Strength' at 'Alchemy Shop' | 3 | 6 |
| buy | Wooden Torches (5000 turns) | General Store | 10-345 | 0 | # Buying 'Wooden Torches (5000 turns)'(h) at 'General Store' (for player 'b') | 63 | 131 |
| buy | Rations of Food | General Store | 4-354 | 0 | # Buying 'Rations of Food'(c) at 'General Store' (for player 'b') | 44 | 87 |
| buy | Scrolls of Phase Door {unseen} | Alchemy Shop | 27-306 | 0 | # Buying 'Scrolls of Phase Door {unseen}'(m) at 'Alchemy Shop' (for player 'b') | 19 | 44 |
| buy | Leather Shields [8,+0] | Armoury | 74-268 | 0 | # Buying 'Leather Shields [8,+0]'(j) at 'Armoury' (for player 'b') | 8 | 17 |
| buy | Small Metal Shields [5,+0] | Armoury | 87-98 | 0 | # Buying 'Small Metal Shields [5,+0]'(h) at 'Armoury' (for player 'b') | 4 | 7 |
| buy | Rings of Feather Falling {unseen} | Magic Shop | 62-118 | 0 | # Buying 'Rings of Feather Falling {unseen}'(u) at 'Magic Shop' (for player 'b') | 2 | 4 |
| sell | Wooden Torches (5000 turns) | Black Market | 77-289 | 0 | # Selling 'Wooden Torches (5000 turns)' at 'Black Market' | 2 | 6 |
| sell | Small Metal Shield [5,+0] | Armoury | 53-77 | 0 | # Selling 'Small Metal Shield [5,+0]' at 'Armoury' | 2 | 5 |
| buy | Sets of Gauntlets [3,+0] | Armoury | 28-77 | 0 | # Buying 'Sets of Gauntlets [3,+0]'(n) at 'Armoury' (for player 'b') | 4 | 8 |
| buy | Hard Leather Caps [2,+0] | Armoury | 39-53 | 0 | # Buying 'Hard Leather Caps [2,+0]'(j) at 'Armoury' (for player 'b') | 2 | 4 |
| buy | Pairs of Leather Sandals [1,+1] | Armoury | 39 | 0 | # Buying 'Pairs of Leather Sandals [1,+1]'(o) at 'Armoury' (for player 'b') | 1 | 2 |
| buy | Cloaks [1,+0] | General Store | 10-15 | 0 | # Buying 'Cloaks [1,+0]'(m) at 'General Store' (for player 'b') | 4 | 12 |
| buy | Soft Leather Armours [8,+0] | Armoury | 268 | 0 | # Buying 'Soft Leather Armours [8,+0]'(c) at 'Armoury' (for player 'b') | 1 | 2 |
| buy | Amulet of Slow Digestion {unseen} | Magic Shop | 68-118 | 0 | # Buying 'Amulet of Slow Digestion {unseen}'(o) at 'Magic Shop' (for player 'b') | 2 | 5 |
| sell_home | Rations of Food | Home | 62 | 0 | # Selling 'Rations of Food' to the home | 1 | 1 |
| buy | Metal Caps [3,+0] | Armoury | 28-202 | 0 | # Buying 'Metal Caps [3,+0]'(n) at 'Armoury' (for player 'b') | 4 | 8 |
| buy | Pairs of Leather Boots [2,+0] | Armoury | 23-38 | 0 | # Buying 'Pairs of Leather Boots [2,+0]'(u) at 'Armoury' (for player 'b') | 3 | 6 |
| buy | Robes [2,+0] | Armoury | 52-53 | 0 | # Buying 'Robes [2,+0]'(c) at 'Armoury' (for player 'b') | 2 | 4 |
| buy | Potions of Cure Light Wounds {unseen} | Alchemy Shop | 24-83 | 0 | # Buying 'Potions of Cure Light Wounds {unseen}'(a) at 'Alchemy Shop' (for player 'b') | 6 | 13 |
| sell | Yellow Potion | Alchemy Shop | 74 | 0 | # Selling 'Yellow Potion' at 'Alchemy Shop' | 1 | 3 |
| buy | Sets of Leather Gloves [1,+0] | Armoury | 63 | 0 | # Buying 'Sets of Leather Gloves [1,+0]'(m) at 'Armoury' (for player 'b') | 1 | 2 |
| buy | Flasks of oil | General Store | 12-67 | 0 | # Buying 'Flasks of oil'(f) at 'General Store' (for player 'b') | 16 | 30 |
| sell | Copper Speckled Potions | Alchemy Shop | 127 | 0 | # Selling 'Copper Speckled Potions' at 'Alchemy Shop' | 1 | 6 |
| buy | Ring of Protection [+8] | Magic Shop | 291 | 0 | # Buying 'Ring of Protection [+8]'(j) at 'Magic Shop' (for player 'b') | 1 | 2 |
| sell | Violet Potion | Alchemy Shop | 119 | 0 | # Selling 'Violet Potion' at 'Alchemy Shop' | 2 | 3 |
| sell | Scrolls titled "punclar loccub" | Alchemy Shop | 74-119 | 0 | # Selling 'Scrolls titled "punclar loccub"' at 'Alchemy Shop' | 8 | 18 |
| sell | Potions of Cure Serious Wounds | Alchemy Shop | 91 | 0 | # Selling 'Potions of Cure Serious Wounds' at 'Alchemy Shop' | 7 | 14 |
| sell | Scroll titled "sectus molens" | Alchemy Shop | 29-289 | 0 | # Selling 'Scroll titled "sectus molens"' at 'Alchemy Shop' | 6 | 12 |
| sell | Ultramarine Potions | Alchemy Shop | 82 | 0 | # Selling 'Ultramarine Potions' at 'Alchemy Shop' | 1 | 3 |
| sell | Orange Speckled Potion | Alchemy Shop | 82 | 0 | # Selling 'Orange Speckled Potion' at 'Alchemy Shop' | 1 | 2 |
| buy | Rings of Searching {unseen} | Magic Shop | 82 | 0 | # Buying 'Rings of Searching {unseen}'(q) at 'Magic Shop' (for player 'b') | 1 | 3 |

## How the fixture was made

The Borg re-decides the same thing every turn while it walks to the
shop, and the state does not change while it walks, so the raw dump
repeats each decision. The fixture keeps one record per distinct
(state, choice) pair: the state signature - the record without its
version, its record number and its turn - is what decides.

- **637 visits** became **301 records**;
- those are **274 distinct states**, and **34 distinct choices**.

**The control's choice is not a function of the state.** It carries a
standing goal from the turn before, so the same state can carry two
choices: **27** of the 274 states do. The fixture keeps
both, because each is a fact about the control. A new algorithm that
is a function of the state will agree with at most one of them, and
that is the comparison.

The fixture keeps the first record of each pair, in the order the runs
were made. The `seq` is the one the run gave it, so each record points
back at the run it came from:

| run | the fixture records it contributed |
| --- | --- |
| `warrior-201` | 0, 1, 2, 4, 7, 10, 13, 16, 19, 22, 25, 28, 31, 34, 35, 38, 39, 43, 47, 50, 53 |
| `warrior-202` | 0, 2, 6, 7, 9, 12, 15, 18, 21, 24, 27, 30, 33, 36, 39, 42, 45, 48, 51 |
| `warrior-203` | 0, 5, 6, 7, 9, 12, 15, 18, 21, 24, 27, 30, 33, 36, 40, 43, 46, 49, 54, 60, 63, 66, 69, 71, 74, 77, 80, 83, 86, 89, 90 |
| `mage-301` | 0, 1, 5, 6, 9, 12, 15, 18, 21, 24, 27, 30, 33, 36, 39, 42, 45, 48, 51, 54, 60, 62, 63, 66, 69, 72, 73, 75, 77, 79 |
| `mage-302` | 0, 1, 2, 5, 8, 11, 14, 17, 20, 23, 26, 29, 32, 36, 40, 43, 47, 50, 53, 56, 61, 62, 65, 68, 71, 74, 77, 79, 84, 88, 91, 94, 97, 100, 103, 105, 106, 107, 115, 118, 121, 124, 127, 128, 130, 133, 136, 139, 142, 145, 148, 151, 154, 157, 160, 163, 166 |
| `mage-303` | 0, 1, 2, 9, 10, 11, 12, 15, 18, 21, 24, 27, 30, 33, 36, 39, 42, 43, 52, 55, 58, 61, 64, 67, 68, 70, 73, 75, 77, 78, 80, 82, 84, 86, 88, 91, 93, 96, 99 |
| `priest-401` | 0, 2, 4, 5, 7, 9, 12, 15, 18, 21, 24, 27, 30, 33, 36, 39, 42, 45, 48, 60, 61, 62, 64, 67, 70, 71, 73, 76, 79, 82, 85, 88, 91, 92, 95, 96, 97, 99, 102, 103 |
| `priest-402` | 0, 1, 2, 3, 5, 8, 10, 14, 17, 20, 23, 26, 29, 32, 35, 38, 41, 44, 47, 50, 52, 55, 56, 58, 61, 63, 67, 68, 70, 73, 74 |
| `priest-403` | 0, 1, 3, 4, 6, 8, 9, 15, 18, 21, 24, 27, 30, 33, 36, 39, 42, 46, 49, 50, 54, 56, 58, 61, 62, 63, 65, 68, 70, 72, 75, 78, 79 |

Regenerate it with:

```sh
tools/town_baseline.py \
    --control \
    7973c0ea \
    --run warrior-201 /workspace/corpus/warrior-201/i32.jsonl /workspace/corpus/warrior-201/desc.jsonl "BORG_TOWN_DUMP=corpus/warrior-201/desc.jsonl BORG_TOWN_DUMP_INTEGERS=corpus/warrior-201/i32.jsonl BORG_IMPL=rust ./harness/run.sh corpus/warrior-201/out 201 0 0 20000" \
    --run warrior-202 /workspace/corpus/warrior-202/i32.jsonl /workspace/corpus/warrior-202/desc.jsonl "BORG_TOWN_DUMP=corpus/warrior-202/desc.jsonl BORG_TOWN_DUMP_INTEGERS=corpus/warrior-202/i32.jsonl BORG_IMPL=rust ./harness/run.sh corpus/warrior-202/out 202 0 0 20000" \
    --run warrior-203 /workspace/corpus/warrior-203/i32.jsonl /workspace/corpus/warrior-203/desc.jsonl "BORG_TOWN_DUMP=corpus/warrior-203/desc.jsonl BORG_TOWN_DUMP_INTEGERS=corpus/warrior-203/i32.jsonl BORG_IMPL=rust ./harness/run.sh corpus/warrior-203/out 203 0 0 20000" \
    --run mage-301 /workspace/corpus/mage-301/i32.jsonl /workspace/corpus/mage-301/desc.jsonl "BORG_TOWN_DUMP=corpus/mage-301/desc.jsonl BORG_TOWN_DUMP_INTEGERS=corpus/mage-301/i32.jsonl BORG_IMPL=rust ./harness/run.sh corpus/mage-301/out 301 2 1 20000" \
    --run mage-302 /workspace/corpus/mage-302/i32.jsonl /workspace/corpus/mage-302/desc.jsonl "BORG_TOWN_DUMP=corpus/mage-302/desc.jsonl BORG_TOWN_DUMP_INTEGERS=corpus/mage-302/i32.jsonl BORG_IMPL=rust ./harness/run.sh corpus/mage-302/out 302 2 1 20000" \
    --run mage-303 /workspace/corpus/mage-303/i32.jsonl /workspace/corpus/mage-303/desc.jsonl "BORG_TOWN_DUMP=corpus/mage-303/desc.jsonl BORG_TOWN_DUMP_INTEGERS=corpus/mage-303/i32.jsonl BORG_IMPL=rust ./harness/run.sh corpus/mage-303/out 303 2 1 20000" \
    --run priest-401 /workspace/corpus/priest-401/i32.jsonl /workspace/corpus/priest-401/desc.jsonl "BORG_TOWN_DUMP=corpus/priest-401/desc.jsonl BORG_TOWN_DUMP_INTEGERS=corpus/priest-401/i32.jsonl BORG_IMPL=rust ./harness/run.sh corpus/priest-401/out 401 5 3 20000" \
    --run priest-402 /workspace/corpus/priest-402/i32.jsonl /workspace/corpus/priest-402/desc.jsonl "BORG_TOWN_DUMP=corpus/priest-402/desc.jsonl BORG_TOWN_DUMP_INTEGERS=corpus/priest-402/i32.jsonl BORG_IMPL=rust ./harness/run.sh corpus/priest-402/out 402 5 3 20000" \
    --run priest-403 /workspace/corpus/priest-403/i32.jsonl /workspace/corpus/priest-403/desc.jsonl "BORG_TOWN_DUMP=corpus/priest-403/desc.jsonl BORG_TOWN_DUMP_INTEGERS=corpus/priest-403/i32.jsonl BORG_IMPL=rust ./harness/run.sh corpus/priest-403/out 403 5 3 20000" \
    --out-dir fixtures \
    --summary docs/TOWN-BASELINE.md
```

The raw dumps the fixture is made from stay outside the repository:
they are runs, not source. The fixture is the curated part.

## What it can and cannot settle

- It **can** say where the new algorithm disagrees with the current
  one: the fixture is a state and a choice, and the new algorithm
  answers the same question about the same state.
- It **cannot** be the whole comparison. The power need prices a
  candidate with the try-on, which reads the item bonuses the engine
  holds - the record does not carry those. So the full comparison
  runs in game, where the engine is there: the shadow-plan mode
  (phase 4) puts the two decisions side by side in one record.
- It **does not** claim the current choices are right. It says what
  they are.

