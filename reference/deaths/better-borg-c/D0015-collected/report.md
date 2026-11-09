# D0015: Fang, Farmer Maggot's Dog at turn 4158

[Discuss this death](https://github.com/diegoami/better-borg-c/issues/348) · [Feedback process](../FEEDBACK.md)

See record.json and build.json for immutable source/build/seed/options.

## Evaluation and diagnosis

See the [overall assessment](overall-assessment.md). A CL1 Necromancer explored into a dead end
while Fang approached unseen in the dark. Once bitten, it meleed Fang to death 7 corridor squares
from an open up staircase. Pristine code deliberately forbids the stair dash ("Don't run from
Grip or Fang"), while another rule lists Fang as a reason to flee the level. Its one Nether Bolt
came last and failed. These are source-checked mechanisms with native evidence. The dash outcome
is untested.

## Action items

- [159: walking retreat](https://github.com/diegoami/better-borg-c/issues/159)
- [327: scary-rule triggers](https://github.com/diegoami/better-borg-c/issues/327)
- [41: survival stock](https://github.com/diegoami/better-borg-c/issues/41)
