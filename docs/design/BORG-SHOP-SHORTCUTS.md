# The Borg's shop shortcuts

The Borg has no way to be told which shop to visit. The strategy picks one
(`borg_choose_shop` in `borg-rs/src/borg_think_store.rs`), the player walks to
one, and nothing else exists. This document is the analysis of how to add a
shortcut, which key to use, and what was built.

## What the game already gives us

**The Borg's own machinery.** `st.borg.goal.shop` is the store the Borg is
heading for, and `borg_flow_shop_entry` (`borg-rs/src/borg_flow_misc.rs`)
flows the Borg to that store's entrance and presses `5` to open it. "Switch
to shop N" is therefore a matter of setting that one field.

**The store order.** `square_shopnum()` returns the store index (0-based) and
`stores[i].feat` its feature, so the indexes are the terrain order:

| Borg's number | store index | store |
|---|---|---|
| 1 | 0 | General Store |
| 2 | 1 | Armoury |
| 3 | 2 | Weapon Smiths |
| 4 | 3 | Bookseller |
| 5 | 4 | Alchemy Shop |
| 6 | 5 | Magic Shop |
| 7 | 6 | Black Market |
| 8 | 7 | Home |

The Borg already speaks this numbering in its own log ("Currently in
preferred store 'N'" prints `goal.shop + 1`, `borg_think_store.rs:425`), so
the shortcut uses it rather than inventing a second one. The shops are not in
a row in the town (`town_gen_layout` in `angband/src/gen-cave.c` puts them in
lots around the crossroads), so "the next shop" is by store index, never by
visual order.

## The key question

The owner's shape was `1`, `2`, `3`, ..., `+`, `-`, `>`: a key that names
the shop. The question was where those keys may live.

**ESC cannot be one of them.** It is the cancel key of every prompt in the
game and of the Borg's own menu (`get_com` returns "not ok" on ESC).

**The keys are taken in the game.** The game's command lookup is a
single-key table (`cmd_lookup` over `converted_list[mode][key]` in
`angband/src/ui-game.c`); there is no general prefix mechanism, only the
three built-in prefixes in `textui_get_command` (`angband/src/ui-input.c`):
`0` (a count), `\` (keymap bypass) and `^` (a control character).

| key | in the game | in the Borg's command menu |
|---|---|---|
| `1` .. `9` | the movement keys (the keypad) | **free** (but `0` is the stats) |
| `/` | "Identify symbol" (`ui-game.c:190`) | **free** |
| `+` | "Alter a grid" (`ui-game.c:222`) | **free** |
| `-` | "Walk into a trap" (`ui-game.c:160`) | **free** |
| `>` | "Go down staircase" (`ui-game.c:155`) | **free** |

So the owner's exact keys work **in the Borg's own command menu** (reached
with `^Z`; its keys are `?$zxmGID@Ffucs^ythpP!C0wSloRv`, so `/`, `+`, `-`,
`>` and every digit but `0` are unused) and would each collide with an
existing command **in the game**.

At the game level the free printable ASCII is `! # $ % & ' ( 0-9 B J N Y ^ \``
and `y`; the digits are used by the store and inventory interfaces, so the
practical game-level prefixes are `#`, `%`, `&`, `'` and `(`.

**The prefix is not needed.** The first version of this change took `/` as
a prefix and then a key ("`/1`", "`/+`"), which cost a second keypress and
a second prompt. The Borg's menu has the digits free, so the shop keys are
the keys themselves: `^Z` then `1` is the whole command.

## What was built, and why there

The shortcut lives in **the Borg's command menu**, on the keys themselves:

* no prefix, so no keypress is spent on entering a mode;
* the keys are free there, so no existing command is unbound;
* the Borg's mechanism for it already exists (`goal.shop` +
  `borg_flow_shop_entry`), so the change is small and Rust-only;
* it needs no change to the C engine, so it is not a fork decision.

The player-side variant ("the *player* walks to shop N") is a different
thing and is **not** in this change: it needs a new command in the C engine
(a new game feature in the fork, which is the owner's call), and the pathing
it would use lives in `angband/src/player-path.c` (`find_path`,
`pathfind_direction_to`). If the owner wants that too, the key there would
have to be one of the free game keys above, not a digit.

### The commands

In the Borg's command menu (`^Z`, then the key):

| keys | effect |
|---|---|
| `1` .. `8` | visit that store, by the Borg's own numbering |
| `+` | the next store (wraps around) |
| `-` | the previous store (wraps around) |
| `>` | drop the choice: the strategy picks the shop again |

The choice is an override (`borg.shop_override`): it outranks the strategy's
own pick until the Borg has been to that store, and then it is dropped and
the strategy decides again. It is ignored while the negative-baseline arm
(`borg_town_no_stores`) is on, because that arm's contract is that the Borg
never walks into a shop.

### In the interactive session

The command is a *preference*, not a walk: it says which shop the Borg
should visit when the Borg next shops. The Borg walks there by itself,
with the flow it already uses (`borg_flow_shop_entry`), in the ordinary
way it walks to any shop.

With the handover on (`BORG_TOWN_PLAYER=1`) the player has the keyboard in
town, so the Borg does nothing there at all: the command records the
choice, and the Borg acts on it when it is in charge of the town again.
The handover still updates the Borg's map from the screen on every key the
game asks for, the way the Borg's own think does, so the store doors the
player has seen are the ones the Borg can find later. The update also reads
the game's messages, and the Borg notes each new one in the message log
(`# <message> (+)`), so in town the player sees their own messages echoed as
Borg notes - as they are when the Borg plays.

An earlier version of this change also had the handover walk the character
to the store (the Borg borrowing the keyboard one flow step at a time, and
opening the store with `5`). That is not part of the command, and it was
rolled back.

## What was checked

* `cargo test --release`: 139 passed, 0 failed, 1 ignored (6 new tests for
  the selector).
* `cargo clippy --release --all-targets -- -D warnings`: clean.
* `harness/diff --suite smoke --expect-impl rust`: 3/3 identical (the menu
  is only entered by hand, so the keypress stream cannot change).
* **Driven in a live session** (Xvfb, the Rust build, the handover on):
  * `^Z` then `1` gives "The Borg will visit the General Store." — one
    keypress, no prefix and no second prompt;
  * `+` steps to the next store: "The Borg will visit the Armoury.";
  * `>` gives "The Borg chooses the shop itself again." and the choice is
    dropped;
  * `?` (the menu's help) lists the new keys:
    "Commands '1'-'9', '+', '-', '>': the shop the Borg visits" - the keys
    the menu reads; with the town's eight stores, `9` answers "That is not a
    legal shop.";
  * the character does not move: the command chooses the shop, it does not
    walk there.

## What was not checked

* The C build's command menu has no such command: the shortcut is Rust-only
  and the C build's `^Z` menu is unchanged. The C and Rust keypress streams
  are identical in the harness, but a human opening the menu in the C build
  would not find it.
* The store the Borg opens is the one the player asked for; what the Borg
  *does* in the store (the Borg's own shopping) was not exercised here,
  because the handover leaves the keyboard with the player in town.
* That the Borg actually walks to the chosen store when it is in charge of
  the town was not driven end to end in a live session: the walk is the
  Borg's own shop flow, and the override only changes which store that
  flow aims at.
* The interactive session was driven with `xdotool` against an Xvfb
  display; the key delivery there is unreliable, so the session was driven
  step by step with screenshots rather than scripted end to end.

