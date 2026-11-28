# #1294 Blackguard mana rest

Track P. On each world tick, the engine calls `player_regen_mana`
(`engine/angband/src/game-world.c:648`). For `PF_COMBAT_REGEN`, that
function drains SP (`player-util.c:507`). `R*` requires full HP and SP
(`player-util.c:1500-1501`), so the Borg skips that rest for combat
regeneration. The `R&` completion condition waives the full-SP
requirement for these characters (`player-util.c:1504-1508`). Dark-entry
detection now uses the same eligibility check for the mana-only reserve.

On Track P `91fa492c`, Half-Troll Blackguard `312081895` issued `R*` in
town at engine turn 80,484 with 66/67 HP and 1/3 SP. The log marks town as
scary in that decision, which rules out `R&`. The Borg then made no
further decisions before the 100,000-turn cap. With the fix, the replay
issued zero `R*` commands, used `R&` 19 times, and was exploring at turn
99,994 on its way to the cap at CL5.
[Issue and reproduction](https://github.com/diegoami/borg-reborn-rs/issues/1294)

The native CI regression replays this start to 90,000 turns and checks
for ordinary health rest, absence of the mana-only rest, and decisions
past turn 80,484. All four `smoke-borg-rs` runs matched the references for
keys, messages and state. Formatting, Clippy, Rust tests and parameter
citations passed.

The remaining decision is whether to merge PR #1295 into `track-p` after
owner review under RULES 1a. [PR #1295](https://github.com/diegoami/borg-reborn-rs/pull/1295)
