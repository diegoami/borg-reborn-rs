# Report: 20261110T013230Z-1 — a Kobold Priest on depth 1

Written from the recording, not from watching: `frames.jsonl.gz` (6,105 frames
with hit points, depth, position and gold), `run/combat.jsonl` (97 game
messages) and `recording.json`. Everything below is from those files; where I
am inferring intent rather than reading a fact, it says so.

## What happened

| | |
|---|---|
| character | Kobold Priest (`race_idx` 10, `class_idx` 3) |
| ended | **alive**, depth 1, character level 2, 18/18 hit points, 18 experience, 2 gold |
| game turns | 4,741 |
| keypresses | 971 (the frame `input` counter, 29 → 1,000) |
| wall clock | 638 s, about ten and a half minutes |
| town | turns 1–451 |
| depth 1 | turns 451–4,741 — **4,290 turns on one level, and never a second one** |

Kills: an aimless-looking merchant and a pitiful-looking beggar in town, then
two giant white centipedes, a giant yellow centipede, a fruit bat and **nine
wild dogs**. Level 2 came at turn 3,091 from the second dog pack.

## The one decision that shaped the session

At **turn 601** the game said:

> *You feel that there is naught but cobwebs here.*

That is the level feeling, and it had just told you the level held nothing
worth finding. You stayed on it for the remaining **4,290 turns** — 90% of the
session — and stood on 151 distinct squares while doing so, which is a small
part of a level, so this was not exhaustive exploration but movement inside a
known area.

Two keys would have replaced it. Stairs are connected, so `<` then `>` from
the arrival staircase gives a fresh level of the same depth, and the second
`<` is free because you arrive standing on one. That is two keypresses against
4,290 turns.

**Why I think it is the whole story:** your hit points tell me you were never
in trouble — median 100% of maximum on depth 1, lowest 56% (10 of 18, during
the second dog pack at turn 3,181). Nothing was stopping you going deeper
except not having a way down.

## What went well, and should not be read as a mistake

- **The escapes you bought were never used, and that was correct.** A Potion of
  Cure Light Wounds and two Scrolls of Phase Door, 50 of your 88 gold, and you
  finished at 18/18 having never dropped below 56%. Insurance that does not
  pay out was still the right purchase.
- **You healed at the right moment.** At turn 1,701, mid-dog-pack, "You feel
  better" — Minor Healing, cast while being bitten, rather than after.
- **You fought the dogs in sequence, not in the open.** Nine of them across two
  packs and the worst you took was 8 hit points.

## Your keystrokes

They are in the recording after all — `run/events.txt`, a row per event, the
keycode in the fifth field — so this is the same measurement the model runs
get. 418 keypresses, written out in `keys.txt` beside this report.

| | count | share |
|---|---|---|
| single steps (arrows, and Home/PgUp/End/PgDn for the diagonals) | **256** | **61%** |
| `.` run | 32 | 8% |
| Escape | 22 | 5% |
| Enter | 21 | 5% |

**Three in five of your keypresses moved you one square.** The run command,
which follows a corridor until something happens, was used 32 times. That is
the difference between this session and the one that reached depth 7: not
judgement, not tactics, but how much ground a keypress covers. A model run
that walked with single steps capped out at depth 1 as well; the one that used
the game's routing did not.

You also drew your character rather than choosing it — `* *` at the two birth
menus, which is the rule the model runs follow for the same reason.

## What I would change

1. **Leave a cobwebs level.** `<` then `>`. The feeling is printed on arrival
   for exactly this decision.
2. **Use the game's pathfinder to find the stairs.** `l` then `x` then `g`
   walks you to the nearest square beside unexplored space, and `l` then `>`
   then `g` walks you to a known down staircase — the game's own routing, one
   keypress string, each sent alone because a key after `g` cuts the walk
   short. This matters more for a Priest than for most classes: **no Priest
   prayer before character level 5 detects anything** (Call Light, Detect Evil,
   Minor Healing, Bless at CL1; Sense Invisible at CL3; Heroism at CL5), so
   you have no magical way to find a staircase at all.
3. **Thirty gold went on problems you did not have.** Detect Invisible 15 and
   Blessing 15. Neither is on the keep list, and nothing invisible or
   unbeatable turned up. In fairness there was no good alternative on 88 gold:
   Magic Mapping is 65 and buying it would have left nothing for escapes.
4. **Eight keys did nothing at all** — "There is a wall in the way!" eight
   times. Each is a keypress and a decision spent on a square you could not
   enter.
5. **At turn 1,701 you pressed aim with no wand**: "You have no wands to aim."
   One wasted key, while a dog was biting. Worth knowing your own inventory
   letters before the fight rather than during it.

## Things the recording shows that are not criticisms

- **You did not choose your prayers.** Minor Healing at CL1 and Call Light at
  CL2 were given to you: Priests and Paladins receive a random prayer where
  other classes choose (`class.txt`, and the game's own study prompt).
- **A locked door cost four turns** — three failures then "You have picked the
  lock" at turns 4,601–4,631. That is the dice, not a decision.
- **"You spin around"** at turn 1,721 looks like a teleport or confusion effect
  during the dog fight; the recording does not say which and I am not going to
  guess.

## How this compares with the model runs

| | keypresses | turns | deepest | level |
|---|---|---|---|---|
| **this session** | 971 | 4,741 | 1 | 2 |
| `sonnet-1` (no prior knowledge) | 644 | 4,659 | 1 | 3 |
| `sonnet-2-again` (used `l>g`) | 1,107 | 10,416 | **7** | 8 |

Your session and the first model run are almost the same session: the same
number of turns, the same depth, the same level, and **the same failure** —
neither found a way down and both stayed where they were. The run that reached
depth 7 differed in one respect, which is that it used the game's pathfinding
to walk to a staircase it had seen.

That is the thing worth taking from this, and it is in
`manuals/INSTRUCTIONS.md` under "The one thing to take from this page".
