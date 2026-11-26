## Keep the stair-scum return active on DL1 (#1260)

Early DL1 looting now yields to an active stair-scum return on the current
level. In the original Necromancer replay (seed 300036923, `242a4724` with the
fix), the Borg took the up stair at turn 29,142 and finished the 100,000-turn
run at CL6, having reached DL2. Four Track P smoke runs matched their
reference keys, messages and state logs before the base was refreshed.

[#1290](https://github.com/diegoami/borg-reborn-rs/issues/1290) tracks making
loot routing agree with the step safety check.
