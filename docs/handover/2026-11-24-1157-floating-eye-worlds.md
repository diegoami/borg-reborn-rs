## Floating-eye worlds (#1157)

Added `eyes-x4` and `eyes-x8`. The generator keeps the floating eye at
its original depth and rarity and multiplies the rarity of other races
by the configured weight. The engine computes allocation weights as
integer `100 / rarity` (`engine/angband/src/mon-make.c`).

Ran `scripts/make-world.py` against the pinned engine data for each
configuration. The generator reported one favoured and 623 other races.
The generated game-data SHA-256 values are:

| World | Game-data SHA-256 |
|---|---|
| eyes-x4 | `d21ff2096cb1419dd715eddb565a88e7f4f3c73b62afcdd731d866a522f558ac` |
| eyes-x8 | `11c4aac337d02d66279d192910d56d92777430d34c538ace3b0414448774d097` |

Use eyes-x8 first in a Zoo pilot with the shipped borg-rs profile and a
second arm that enables only `escape_paralyser_fight_lost`. Keep
walk-away on in both arms. Record the number of known awake paralyser
contacts and how many of them reach and change the Phase decision. An
unlisted stationary race, such as the grey mold, is the control for
the separate `caution_remember_never_move` decision (#1150).
