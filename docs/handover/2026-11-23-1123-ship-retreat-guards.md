## Ship the retreat-loop guards (#1123)

The owner enabled `caution_retreat_reentry_ticks = 128` and
`caution_retreat_oscillation_limit = 3` in the shipped borg-rs profile.
The three-think repeat window and the strategic-retreat-only behavior
remain in place. No decision code or compiled defaults changed.

The [live Ranger report](https://github.com/diegoami/borg-reborn-rs/issues/1123)
describes repeated stair returns after retreats on DL3. These values
match the existing pilot thresholds and establish no population survival
claim. The owner authorized merging this setting change after CI without
a separate review.
