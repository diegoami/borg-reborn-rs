# Native P1 tier regression fixture

`warrior-level5.build` is an actual Human Warrior level-5 observation from
seed 101, captured at engine turn 18057. It has native maximum HP 72 and the
observed physical inventory, including awareness flags and consumed light fuel.
The real source recipe and final clean-build provenance are retained by
borg-cemetery's `synthetic/p1-warrior/` catalog at its milestone pin.

This is a declarative tier projection, not a save or a claim of a typical kit.
Tests require capture on/off play parity and independently check its bytes and
native HP calculation. The source health (58/72) is recorded as provenance;
synthetic application explicitly starts healed (72/72).
