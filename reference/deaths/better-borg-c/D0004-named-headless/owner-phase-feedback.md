# Owner Phase Door feedback: earliest decisions

[Owner feedback on investigation34](https://github.com/diegoami/better-borg-c/issues/34#issuecomment-6709969033).
The owner disputes BOTH the first Phase Door at20293 and the response to landing
in a crowded room at20294. They propose fighting the manageable single opponent
initially, or leaving the landing room through its door immediately and seeking
upstairs, even via unexplored ground. They observed repeated Phase Door use and
three creatures eventually swarming the actor around20430. These are observations
and proposed alternatives, not established rescue paths or a confirmed bug.

## Recorded anchors

These PNGs render the original recorded terminal cells; they are not live desktop
screenshots. JSON/text retain original frame/phase and diagnostic state.
[Anchor hashes and source receipt](owner-phase-feedback-anchors.json).
All four are from actual C34 source1b95154f65b2, recording manifest
`ecdff734f85422ec21f1f09ac4b57f3c855c8c04618e8536f13706a22b6a1ff3`.
D0002/D0003/D0005 are qualified family references, not relabelled C34 originals.

| Engine turn / phase | Position x,y | HP | Evidence |
| --- | --- | --- | --- |
|20293 command, frame24832|66,33|46/49|[Before first Phase decision](snapshots/owner-feedback-20293-before-phase-decision.png)|
|20294 tick, frame24839|64,23|46/49|[Phase landing](snapshots/owner-feedback-20294-phase-landing.png)|
|20303 command, frame24850|64,23|46/49|[First post-landing decision](snapshots/owner-feedback-20303-first-post-landing-decision.png)|
|20430 tick, frame25049|61,20|22/49|[Late fight](snapshots/owner-feedback-20430-late-fight.png)|

20294 is the landing tick; the first subsequent command boundary is20303.
The intervening engine ticks are not nine declined walking opportunities.
At20293 the actor carries four known Phase Door and twelve known Cure Light
Wounds; at20294 three Phase Door remain. The adjacent visible cutpurse has21HP.
Landing exposes a scout, metallic green centipede, gallant and two acolytes in
recorded visible-monster diagnostics. Those diagnostics do not give sleep state;
the specific wake order is not proved. At20430 the gallant and one scout are
adjacent, another scout is two squares away, and Phase Door stock is exhausted.
Hidden diagnostic enemies must not be presented as Borg knowledge.

## Danger and repeated Phase Door

Original BORG trace:

```text
20293 HP46/49 Danger:p=188
20293 Safe to Phase. scary squares:14
20293 Reading Scrolls of Phase Door. / Danger Level1.
20303 HP46/49 Danger:p=451
20303 No Phase. scary squares:97
20303 Reading Scrolls of Phase Door. / Danger Level1.2 Critical Phase
20313 HP47/49 Danger:p=401 / No Phase100 / Critical Phase
20323 HP47/49 Danger:p=427 / No Phase100 / Critical Phase
```

Danger DOES rise at the next evaluation after landing:188 to451. At20293 the
recorded avoidance is46 (currentHP). 188 is about4.1 times that reference, so it
is high in this heuristic; it does not mean188HP of incoming damage.

The pinned source's borg_danger() combines remembered-monster danger with fear
fields, forces full_damage=true and applies risk penalties. In
borg_danger_physical(), gold theft adds50 at character level<5, independently of
the thief's actual HP damage. borg_danger_one_kill() also boosts danger for a
race with friends: at level<15, v1 += v1*18/10 (2.8 times before rounding).
The exact runtime decomposition of188 was not captured; these source mechanisms
explain why the score cannot be read as the cutpurse's damage, not an asserted
complete reconstruction of188.

The actual cutpurse data has a1d6 physical hit plus gold-stealing touch. This
warrior might beat it, but21 enemyHP versus46 playerHP alone does not prove that:
actual hit chance, blows, speed, other known threats and legal alternatives need
a native comparison. No fight/door/upstairs branch has yet been tested.

borg_caution_phase() samples100 destinations using remembered terrain/danger;
it is not a calibrated actual landing probability. Level1 uses the emergency75,
turns2 check:14 scary samples pass. At20303,451 exceeds4.5*avoidance(46)=207;
the Level1.2 critical-phase fallback can spend a scroll despite the failed97
sample safety check. The later401/427 danger similarly exceeds4.5*47. That
explains the emergency chain without proving it was preferable to walking.
Relevant danger/escape/caution and monster-data files are unchanged between the
historical C34 pin and the inspected f3ced6219 checkout.

## Open actions

On34: reconstruct pre20293 known danger and actual danger components; compare
legal fight/walk/phase choices; at the20303 decision evaluate the proposed door
route, unknown terrain, path cost and stair knowledge, including adverse RNG.
Keep stock and action selection distinct. [154](https://github.com/diegoami/better-borg-c/issues/154)
links source370's early escape/cure restock proposal: this actor already had both,
so that reserve alone is not an established prevention for this encounter.
[41](https://github.com/diegoami/better-borg-c/issues/41) remains the separate
reserve-priority proposal. No policy/threshold change or survival benefit claimed.

The replay interface lacks a time-aligned danger/thought-log pane. Per owner,
[153](https://github.com/diegoami/better-borg-c/issues/153) is FILED ONLY, deferred.
Further candidates follow the separate [analysis and experiment processes](../../measurement/DEATH-IMPROVEMENT-WORKFLOW.md).
