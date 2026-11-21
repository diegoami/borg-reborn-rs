## 2026-11-21 · Adjacent stair loop (#862)

An up-stair search that steps off a down stair can now take its next
step without ordinary or desperate flight immediately targeting that
same down stair. The unweighted and weighted stair routes exclude only
the down stair just left. The deferral expires at the end of the local
handoff, on a level change, or when the search loses its onward step.
A marked increase in danger restores the original stair choice.

The original recording, build and exact turn ranges are identified in
[#862](https://github.com/diegoami/borg-reborn-rs/issues/862).

The native Warrior smoke run reproduces the same transition at game
turn 7,456 in the old reference. With the fix it uses Recall instead
of walking straight back to the staircase. The Mage and Priest
smoke runs match their previous play.
