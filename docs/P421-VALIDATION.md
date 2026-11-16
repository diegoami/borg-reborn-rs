# Stair-livelock fix validation (#421)

Canonical [incident and bounded trial receipts](https://github.com/diegoami/borg-cemetery/blob/f6b800273c187f632dcaf727c88118936349e9ff/validation/borg-reborn-rs/stair-livelock-421/README.md)
and [machine-readable identities](https://github.com/diegoami/borg-cemetery/blob/f6b800273c187f632dcaf727c88118936349e9ff/validation/borg-reborn-rs/stair-livelock-421/validation.json)
are retained in Cemetery PR22. Raw outputs and executable images remain
outside Git at the paths recorded there.

A dungeon-only loop now defers a blocked descent to native play on the
current level, retaining its waypoint. Native preparedness and genuine
retreat remain authoritative; town-ended loops and one-shot commands keep
their refusal behavior. No engine timing change or retry threshold.

The short constructed native regression fails on baseline40a6fa9c (1409
keys at turn2092) and passes on candidatecce8ccfc (peak39). Existing loop,
recall and town-retreat checks pass. The current Half-Troll reported-seed
case drops from two threshold bursts to zero; all four reported-settings
candidate checks have zero threshold bursts and finish alive at cap.
Current Warrior/Orc baseline cases already had no threshold bursts after
other accepted fixes; no reduction is claimed for those cases. The original
Warrior hash5d6b065592fb was separately reproduced on its preserved source.

Candidate runtime source: d012f3af1441c69dbdd4c7dcf8200d4eda55bb8c82b6c4bafa8dec784f390226.
Clippy/formatting/workspace checks,339 release core tests (4 ignored),91 tool
and19 script tests pass. Three new natural CI deaths reconcile against the
pinned independent canonical registry. Required CI gates track-p integration.
No population or survival-improvement claim; trials and source epochs stay
separate. Configuration failures and wrong-race controls are retained and
excluded from reported-character reproduction claims.
