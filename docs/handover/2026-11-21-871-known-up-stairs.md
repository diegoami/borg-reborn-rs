## 2026-11-21: Keep following a known up staircase (#871)

Extend the up-stair handoff from #862 to an explicitly chosen flight
to a known up stair. Before emergency movement targets an adjacent down
stair, the Borg replans the up route from the square it reached. It can
return to normal fallback if the route stalls or danger rises. It still
considers other emergency escapes and nearby up stairs first.

The exact BBorg recording ranges are on
[#871](https://github.com/diegoami/borg-reborn-rs/issues/871).
The engine test uses Warrior seed 224. Before the fix the Borg stepped
off a down stair at turn 143,021, then moved back onto it.
