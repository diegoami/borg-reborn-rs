## Shipped immobile decisions (#1091)

The test build runs the real caution, walk-away, stair flow, Phase gate,
bounce reset and expiry with the shipped profile. The harness fabricates
the Borg's observations after engine initialization and checks its queued
actions. The Phase comparison overrides only the Phase gate, and the
memory comparison overrides only stationary-race retention.

- At the shipped three-think, three-retreat limit, three strategic moves
  are each observed landing and then undone. The real caution chain
  vetoes the fourth move and allows a fresh one after the window expires.
- A one-step stair inside a listed eye's reach is unavailable to the
  stair flow. The dungeon thinker steps west, out of reach. Clearing
  the eye allows the same route. A usable stair underfoot takes priority.
  From the safe square a simulated shot scores zero with no launcher and
  scores positively with a bow and arrows.
- Cornered beside an awake eye, with the next stair inside reach and a
  readable Phase Door, the shipped gate refuses. The override permits
  Phase and queues the scroll read after the landing check. Both arms
  refuse Phase with the eye recorded as asleep.
- A listed eye survives both the bounce reset and expiry. The override
  also retains an unlisted grey mold. A mobile monster clears in both arms.

Run `BORG_TEST_FEATURES=test-stair-handoff harness/build.sh` and
`python3 engine/tests/test_immobile_decisions.py`. CI runs these cases
with the existing engine test build. [PR #1104](https://github.com/diegoami/borg-reborn-rs/pull/1104)
covers older optional switches via helper tests and source assertions.
The historical aim loop fixture for #1132 is tracked in
[borg-cemetery #150](https://github.com/diegoami/borg-cemetery/pull/150).
