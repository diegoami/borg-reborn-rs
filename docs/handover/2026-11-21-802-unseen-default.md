## 2026-11-21: ship unseen blocker handling (#802)

The owner asked to enable #781 by default, update smoke, and merge. The
shipped borg-rs profile remembers blocked grids for 200 game turns, enables
the detection, oil and Phase Door response, and uses the bump cure below
50% HP. The existing budget is three oil throws per grid. This is the
profile exercised in the [#781 review trials](https://github.com/diegoami/borg-cemetery/tree/b20537c68830e415c79a8766135a1b32c7e539c2/validation/borg-reborn-rs/unseen-bump-781/review).
The compiled fallback settings still allow a profile without these options.

The default-core smoke suite completed all three games with no deaths. The
Warrior diverges at turn 5,326 when the newly active response throws oil at
an unseen blocker. The Mage and Priest match the old reference. The updated
reference matches all three runs. Formatting and parameter citations passed.
The smoke suite verifies deterministic play. Population performance and
runtime Detect Invisible coverage remain unmeasured.
