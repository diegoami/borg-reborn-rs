# #1294 Blackguard mana rest

Track P. `borg_recover` now offers its mana-only `R*` command only to
characters whose mana can regenerate at rest. `PF_COMBAT_REGEN` drains
mana at rest (`engine/angband/src/player-util.c:507`). The engine's
completion condition for the ordinary health rest (`R&`) already supports
combat regeneration (`player-util.c:1495`).

On Track P `91fa492c`, Half-Troll Blackguard `312081895` issued `R*` in
town at engine turn 80,484 with 66/67 HP and 1/3 SP. It made its last
Borg decision there and reached the 100,000-turn cap at CL4. On the same
start with the guard, it issued zero `R*` commands, used `R&` 19 times,
and was exploring at turn 99,994 on its way to the cap at CL5.
[Issue and reproduction](https://github.com/diegoami/borg-reborn-rs/issues/1294)

All four `smoke-borg-rs` runs matched the references for keys, messages and
state. Formatting, Clippy, Rust tests and parameter citations passed.
