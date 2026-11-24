## 2026-11-24: Shipped immobile decisions (#1091)

The test build runs the real caution, walk-away, stair flow, Phase gate,
bounce reset and expiry with the shipped profile. The harness fabricates
the Borg's observations after engine initialization and checks its queued
actions. The Phase comparison overrides only the Phase gate, and the
memory comparison overrides only stationary-race retention.

- At the shipped three-think, three-retreat limit, three strategic moves
  are each observed landing and then undone. The real caution chain
  vetoes the fourth move and allows a fresh one after the window expires.
  The fixture advances the Borg clock, once per think, with the engine
  game clock held fixed.
- A one-step stair inside a listed eye's reach is unavailable to the
  stair flow. The dungeon thinker steps west, out of reach. The flow
  can take a separate stair out of reach. Clearing the eye allows the
  original route. A usable stair underfoot takes priority.
  From the safe square a simulated shot scores zero with no launcher and
  scores positively with a bow and arrows.
- Cornered beside an awake eye, with the next stair inside reach and a
  readable Phase Door, the shipped gate refuses. The override permits
  Phase and queues the scroll read after the landing check. Both arms
  refuse Phase with the eye recorded as asleep. With no Phase source,
  the override still defers early combat. Neither arm queues a key in
  the cornered walk-away path, which leaves the rest of the think to act.
- A listed eye survives both the bounce reset and expiry. The override
  also retains an unlisted grey mold. A mobile monster clears in both
  arms. The flow still excludes the eye's reach and can seed beside the
  unlisted mold.

Run `BORG_TEST_FEATURES=test-stair-handoff harness/build.sh` and
`python3 engine/tests/test_immobile_decisions.py`. CI runs these cases
with the existing engine test build.

### What is left

[#1091](https://github.com/diegoami/borg-reborn-rs/issues/1091) keeps the
four-rule coverage request open. The remaining cases cover the older
immobile-route bias (#931), the retreat guard's back-away and reset cases
(#1023), the immobile-neighbour fight veto (#933), and poison cure choice
(#934). The proposed tests in #1104 were closed before integration.
The historical aim loop fixture for #1132 is tracked in
[borg-cemetery #150](https://github.com/diegoami/borg-cemetery/pull/150).

### For the owner

Keep the tested Phase and retention overrides separate for any gameplay
comparison. [Zoo #760](https://github.com/diegoami/borg-zoo/issues/760)
tracks the Phase pilot.
