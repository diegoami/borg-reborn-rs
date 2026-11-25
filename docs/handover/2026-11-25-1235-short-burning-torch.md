## 2026-11-25: Use short-burning replacement torches first (#1235)

Routine maintenance, recovery in the dark, and harmful-light replacement
now prefer the clean torch with the least fuel among equal-radius torches.
An ordinary equipment upgrade uses the same tie-break after passing its
power and danger checks. The town equipment plan keeps the chosen radius
when finding the torch again and selects the shortest clean match.

The regression reconstructs the recorded 73-, 3,000-, and 5,000-turn
spares in two pack orders. It checks the first two replacements, a brighter
torch, spent and known cursed spares, and a working torch down to its last
turn. The fuel comparison uses the timeout of one wielded copy, including
when the spare is a stack.

Rust formatting, Clippy, both Rust test suites, and the parameter citation
check passed. All four default-core smoke runs matched `smoke-borg-rs`.
Track P integration awaits CI and owner review.
