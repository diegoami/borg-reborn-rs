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
