## 2026-11-21: Lightless dungeon recovery (#811)

The default borg-rs core gives lightless recovery its own decision path.
It refuels or equips a working light, can use local illumination, and
prioritizes safe routes to remembered upstairs. Recall is available as a
return option, including a reachable glowing grid when needed to read.
Search probes only the edge of the remembered map and refuses failed
steps. The recovery yields control with a reason when it runs out of safe
options or reaches its action limits.

The regression models the two grids and opposing movement in FBorg's
issue report. The original X11 recording and saved fingerprint were
unavailable in the workspace, so no playback of the incident was verified.

Workspace formatting, Clippy and tests passed, including the release
borg-core suite and the vendor check. The native shipped Borg smoke
suite matched the reference on all three runs.

### Review follow-up

Recovery checks carried light, then permits healing, curing and eating
before routing. It uses the brightest working spare light. The action,
search and door retry budgets can be set in borg.txt through
`borg_param_lightless_recovery_actions`, `borg_param_lightless_search_actions`
and `borg_param_lightless_door_attempts`. Their defaults are 250 total
actions, 32 search actions and three single door attempts from each approach.
A handoff ends the headless run as `borg_stopped`.

### Remaining validation

The tests cover the route, Recall priority, the movement direction and the
reason selected for handoff. Full engine playback of those recovery paths
requires a retained or constructed lightless scenario. The original
recording remains the evidence for the observed loop in #811.
