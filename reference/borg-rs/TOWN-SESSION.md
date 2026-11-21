# The interactive session: the Borg plays the dungeon, the player plays the town

The baseline of inventory choices needs a second kind of evidence: the
*player's* choices, in the *real* game, at the *real* town states. The
headless corpus gives the control's choices; this gives a human's.

The shape of it: the player starts the game, activates the Borg (CTRL-Z,
then `y`, then `z` at the Borg command prompt), and buys things in town. The
Borg keeps playing the dungeon. Every time the character is in town, the
controls are the player's; every time it is in the dungeon, they are the
Borg's. When the character dies, the player starts a new character and the
same thing happens again.

## The handover

The engine asks for a key through `inkey_ex` (`angband/src/ui-input.c`).
If the Borg's input hook is installed, the hook's key is used; if the hook
returns no key at all (`EVT_NONE`), the engine falls through to the real
keyboard. That is the whole mechanism:

```c
if (inkey_hack)
{
    ke.key = (*inkey_hack)(inkey_xtra);
    if (ke.key.type != EVT_NONE)
    {
        ...
        return (ke);
    }
}
```

`generate_keypress` (`borg-rs/src/borg.rs`) returns `EVT_NONE` when the
character is in town and `BORG_TOWN_PLAYER=1`. The engine then reads the
keyboard, and the player has the controls.

The handover is *not* the null arm. The null arm (`borg_town_no_stores`)
stops the Borg's shop decisions but leaves the rest of its town behaviour
alone (it still recalls, still equips, still dives). The handover stops the
Borg from doing *anything* in town, which is what makes it safe for the two
of them to share the keyboard: the Borg's keys and the player's keys never
mix, because in town the Borg produces none.

The handover is off unless the environment variable says so, so the headless
arms (the null arm among them) are untouched. The keypress stream of the
headless runs is byte-identical with and without it.

## What is recorded

Per character, in `OUTDIR/char-N`:

- `state.jsonl` - the states the character was in, in town, one record per
  distinct state. The record is the integer projection's shape (schema
  version 3: the state on its own, with no decision in it): the traits, the
  pack, the equipment, the home, and every shop's stock. The clock is the
  game's turn.
- `native/` - the native recording (the C side's `SUPPORT_BORG_RECORDING`,
  which is an opt-in build: `BORG_RECORDING=1 harness/build.sh`). The
  inputs, the frames and the messages. The player's purchases are in the
  messages; the frames carry the character's state at every frame.
- `dump.jsonl` - the Borg's own town decisions. With the handover there are
  none: the Borg makes no decision in town.
- `objects.log` - the objects the Borg used, bought and picked up, one per
  line, numbered. See below.
- `feedback.md` and `corrections.jsonl` - what the owner said about the run
  when it was over. See "The end of the session".

## The objects the Borg uses

`BORG_OBJECT_LOG=<file>` turns on the object log. Every event is printed on
the console *and* appended to the file, with the same line, so the console
and the file always agree:

```text
0007 turn 142 depth 23 quaff a Potion of Healing
0008 turn 150 depth 0 buy a Scroll of Light at the General Store, cost 25 gold
0009 turn 150 depth 0 acquire 2 x a Potion of Healing
```

Four kinds of event:

- `quaff`, `read`, `use`, `zap`, `aim`, `activate` - the Borg used an object
  it carried: a potion, a scroll, a staff, a rod, a wand, or an activation of
  a ring, a dragon armour or an artifact. Food is left out: eating is not a
  decision worth a line.
- `buy` - the Borg bought an object, with the store and the item's *cost* (the
  price the Borg judged it at; a store can charge its own price on top).
- `take` - the Borg took an object back from its home (there is no price).
- `acquire` - something appeared in the pack that was not there before. With
  the handover the player is the one who shops, and the pack is the only place
  that shows it. The log cannot tell a purchase from a pickup, so it says
  `acquire` and not `buy`. The Borg's own purchases are not counted twice:
  they are the `buy` lines, and the pack does not report them again.

Two items are the same kind when their `tval` and `sval` agree. The `pval` is
left out: for a wand or a staff it is the number of charges, and a charge
spent, a recharge, or the charges becoming known must not look like a new
item in the pack.

The console line is written with the error ignored: a console that has gone
away (a closed pipe) does not stop the game.

The log never presses a key and never changes a decision, so a run with the
log is the run without it, key for key. With the variable unset nothing
happens at all: no file is opened, no line is printed.

## Buying with one keypress

The store's list gives every ware a letter. Pressing the letter selects the
row and opens that ware's menu (*Examine*, *Buy*, *Buy one*); *Buy* then asks
how many (the default is one) and asks to confirm, so buying takes the letter,
`p`, ENTER and ENTER.

The new interface option `store_buy_direct` ("Buy an item by pressing its key
in a store") makes the letter buy the item straight away: one of it - the same
amount the old flow bought by default - with no question asked. It is the
"Buy one" action without the confirmation.

Two things follow that are worth knowing before the option is turned on:

- It applies to the home too: in the home the letter takes one of the item
  back, with no question.
- It applies to the player, not only the Borg: with no confirmation, a stray
  letter in a store is a purchase.

The option is off by default, so the stored reference runs and the control
arm are untouched. It is a game option: it can be set in the game (the `=`
key, "User interface options") or from a file:

```
option:store_buy_direct:yes
```

The harness takes that file through `BORG_OPTIONS_FILE`:

```
BORG_OPTIONS_FILE=/path/to/customized_interface_options.txt bash harness/play.sh /tmp/session 1
```

The Borg uses the same shortcut when the option is on: it presses the list
key and nothing else. What the Borg *buys* does not change with the option -
the same ware, the same amount - only the number of keys does. The C and the
Rust Borg do the same thing, so their keypress streams stay equal with the
option on and with it off.

This is the *store's* side, and it is not the Borg's shop command: the Borg's
own command ("which shop does the Borg visit") is a separate thing, in
[PR #48](https://github.com/diegoami/borg-rs/pull/48), and it does not need
this option.

This is a change to the engine's store screen, not to the Borg: see
[FORK-CORRECTIONS.md](FORK-CORRECTIONS.md) for the fork question it raises.

## The end of the session

When the game is over, `harness/feedback.sh` asks three questions and
writes the answers next to the run:

1. How was the run?
2. How was the buying strategy?
3. Anything that was wrong? ("bought scroll on accident", "not sure about
   this one", ...) - one correction per line, each starting with the number
   of the object log line it is about.

The answers go to `feedback.md` (the three sections, with the run's
metadata) and to `corrections.jsonl` (one JSON object per correction:
`{"line": 12, "text": "bought scroll on accident"}`, and `"line": null` when
the correction names no line). The corrections are the labels the phase-1
usefulness table needs, in the shape the strategy work can read them from.

The script asks nothing when the terminal is not interactive (a scripted run):
it writes an empty feedback file and an empty corrections file instead.

## Running it

```
BORG_RECORDING=1 BORG_IMPL=rust bash harness/build.sh
bash harness/play.sh /tmp/session 1
```

(The connector that pushes the branch writes the shell scripts without the
executable bit, so `bash harness/build.sh` is the form to use; the bit is
set in the local commit.)

The game needs a display (the interactive front end this build has is X11;
there is no terminal front end in the cmake build). In the game: CTRL-Z,
then `y`, then `z`. The player buys what they want in town and dives; the
Borg plays the dungeon. With the handover the Borg does nothing at all while
the character is in town: the player shops, the Borg dungeons.

`harness/play.sh OUTDIR CHARACTERS` plays several characters, one directory
each: the player quits the game when a character is done, and the next one
starts. After each character the script runs `harness/feedback.sh`, which
asks the three questions above (nothing is asked when the terminal is not
interactive).

The environment variables the session uses:

| Variable | What it does |
| --- | --- |
| `BORG_TOWN_PLAYER` | `1` hands the controls to the player in town. Off otherwise. |
| `BORG_TOWN_STATE` | The file the town states are written to, one record per distinct state. |
| `BORG_OBJECT_LOG` | The file the objects the Borg uses, buys and picks up are logged to. |
| `BORG_OPTIONS_FILE` | A game options file (`customized_interface_options.txt`) copied into the run's user directory. |

## What was checked

- The headless arms are unaffected: `cargo test --release` and the smoke
  suite's parity (3/3 identical between the C and the Rust build) with the
  handover code in place, and the handover off (the default).
- The state dump in a headless run: 138 records for seed 201 (Human
  Warrior, 8000 turns), every one of them in town (depth 0), the clock the
  game's turn.
- The interactive session, driven under a virtual X server: the game starts,
  the Borg activates, the character stays in town with the controls handed
  back (the state dump fills), and the player's `>` takes the character into
  the dungeon, where the Borg plays on its own.

## What was not checked

- The pairing of the player's purchases with the states they happened in:
  the recording settles the clock question. The messages carry the frame
  number they happened in (`{"frame": 26, "turn": 1, ...}`), and the frames
  carry the state at every frame, so a purchase is paired with a state
  through the recording, not through the game's turn (store commands do
  not consume turns, so several purchases in one visit share a turn). The
  curation that does the pairing is not written yet.
- The death and the next character: the harness plays several characters in
  several directories, but the loop was not driven through a death here.

The native recording in the interactive session was checked with a build
that has it (`BORG_RECORDING=1`): the run left `events.txt` (the player's
inputs), `actions.txt` (the Borg's keys), `frames.jsonl` (the state at
every frame), `native-messages.jsonl` (the messages with their frame) and
`record-complete.json`.
