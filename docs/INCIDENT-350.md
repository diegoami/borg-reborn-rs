# Recorded wild-cat safety fix (#350)

The source case is a level5 Gnome Mage killed on DL1, despite16 CLW and one
CSW, while the errand returned to town. The frozen original replay verifies;
its final save and full core trace are byte-identical. Source, recording,
report and zipped native fixtures are preserved in player-cemetery. See
[the death report](https://github.com/diegoami/player-cemetery/blob/3162f9d/deaths/20261115T212725Z-PlanMage-wild-cat/report.md)
and [issue350](https://github.com/diegoami/borg-reborn-rs/issues/350).

## Final policy

- Remove the legacy return-to-town stair-looting function and its exclusive
  helper path. It must not spend turns collecting copper before caution.
- GOAL_FLEE rejects a known occupied next step and selects an available
  detour. It never implicitly authorizes a bump attack through that square.
- Known CLW/CSW/CCW bypass the RNG skip when modeled next-round damage
  can match the current wounds or kill the character, including while fleeing.
  Small cures must still pass the existing damage-efficiency limits. The heal's benefit is capped
  at missing HP before testing whether it buys a survivable next round.
- APHASE's conservative preparation count is not a complete escape
  capability test. In this case Phase Door was legal/affordable at5% fail,
  excluded from the3% preparedness count. The actual spell is now checked.
- An already fleeing character that is surrounded or faces modeled lethal
  next-round danger may try a short escape before further combat. Unknown
  landing squares alone cannot forbid this fallback. It uses the existing
  emergency spell-failure bound and honors arena/anti-teleport restrictions.
  It does not override the existing Morgoth-position hold policy.

The last rule is a targeted policy hypothesis supported by the recorded
case, not a calibrated survival guarantee or a population improvement.
The initial three-part patch still lost two late starts; those controls
remain preserved. Instrumentation then identified the false availability
and unknown-landing rejection, and the short-escape correction saved them.

## Reproduction boundary

Native saves at turns411638/648/658/668 have the exact world fingerprints
of recorded command frames26812/26816/26820/26824. They were captured through
the recorder's existing forked native-save callback in an isolated original
replay; taking them left all original streams unchanged.

Reloading compacts/rebuilds engine state and starts a fresh Borg; it does not
restore the old flow/goal/map memory. The exact original suffix is therefore
not claimed. Nevertheless, all four original-source fresh-Borg branches die
from wild cats, while the fixed branches survive the nominal2000-game-turn
cap and recover to full HP. Some stay on DL1; returning to town is not the
acceptance endpoint. Ordinary frontend rests may overshoot the nominal cap.
These are repeated starts from one life, not four independent observations.

The pinned fixture archive and hashes are in harness/fixtures. CI downloads
that exact player-cemetery revision and requires the real native regression;
a missing or corrupted archive cannot silently skip in CI. The loaded save's
old message history is excluded from current-action assertions.

## Checks and scope

Required Rust checks, vendoring and all69 native tests pass locally. Final
clean-build controls, required CI and the exact delivery pin are recorded
with the PR. No engine or recorder implementation is patched here. Default
vendored-core play intentionally changes; original-core smoke must retain
its reference streams. No full45-case parity or population benefit is claimed.

Rolling checkpoint retention and turn-based read-only viewing belong to the
separate angband-recorder change. Tagged consumer integration remains separate
from this safety fix; the preserved original runtime is never rebuilt.
