## 2026-11-27: lightless recovery falls back to the ordinary think (#1359)

Owner, 2026-11-27, recommendation 5 of an audit of the wrapper against
borg-rs.

**What was done:** `lightless_fallback` (compiled 0, which keeps #811 as it
was; shipped 1 in `harness/pborg.yaml`). With it on:
- `borg_lightless::update` gives the turn to the ordinary think (caution,
  attacks, the floor, breeder and curse steps) while a monster seen in the
  last two game turns is next to the Borg.
- `handoff` no longer calls `borg_oops`. It notes `# Lightless recovery gives
  up (reason); ordinary play`, marks the episode `gave_up`, and the ordinary
  think, as borg-rs plays it, has the level until the Borg has light again or
  reaches a new level.

**Checked:** CI's cargo commands; smoke on the default core (see the PR).

**Not checked:** a game that loses its light. The harness has no switch to
start without light, and no lightless case was replayed. The #811 incident
(the vein reversal) can come back once recovery gives up, because the
ordinary think is borg-rs's. Whether the switch helps is for the zoo
(RULES 14): count `borg_stopped` endpoints and lightless deaths with the
switch at 0 and at 1.
