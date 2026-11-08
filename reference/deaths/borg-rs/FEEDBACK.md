# Death feedback and investigation

Adapted from better-borg-c's death workflow at f3ced6219768. Open the
[registry](README.md) and use a case's Discuss link when assigned. Reports
without an investigation link remain explicitly unassigned. Logging never
creates issues or changes gameplay automatically. Supply --death-issue URL
on the headless command, BORG_DEATH_ISSUE for GUI play, or --investigation URL
when exporting a dossier. Do not use the closed M4 review as a new death queue.

Useful feedback names the case, observed danger/resources, engine turn and
question or suggested alternative. Observations and hypotheses remain separate
from native facts, the Borg's actual knowledge and demonstrated causes. Record
feedback links in assessment.json, add concrete actions with stable keys/status/
issue links, investigate, then reply on the authorized discussion and regenerate
the index. Do not silently treat acknowledgement as resolution or a death as
proof of a C/Rust defect. Original-C findings follow CLAUDE.md and the canonical
neo-angband-tracker duplicate/routing policy; never post to angband/angband.

During active investigation, check new owner feedback on linked discussions
and open action issues at session start and before stopping. Unassigned links
are not an excuse to invent discussion threads or use another repository's
cases. This is active-session work, not unattended comment monitoring. Copy
chat feedback to a discussion only within the owner's authorization to post.
Preserve original build/run identity as assessment and action status evolve.

Default observation offsets are 5,12,25,40,60,82,105 **engine world turns** before
death ([profile](window-profile-v2.json)). Each exported window must be an
actual original frame with the exact turn/phase. Missing frames are listed as
unavailable, including gaps between headless key boundaries; do not interpolate,
relabel a nearby frame or claim distinct actions at every world turn. Additional
earlier entry/inventory/retreat anchors should be separately labeled.

Replay verification reruns the whole original frozen session; --from-turn only
chooses where viewing begins. Living engine saves do not restore private Borg
memory, and loading one is a diagnostic branch until equivalence is established.
Dead native saves are terminal evidence, not restorable pre-death worlds.
Repeat/reference runs and backup branches do not create new independent lives.
Bug fixes follow the existing owner policy; policy proposals need evidence and
are not implemented automatically from a single death.

Full raw bundles, executables and saves stay outside Git. Dossiers contain
curated observations, input/build hashes, checkpoint metadata and load-validation
status. Original older cohort reports remain linked and do not acquire a movie
or stronger replay claim merely by entering the registry.


## Explicit owner replay queue

Use ./replay_next_death.py --list, then the default command or --case D0005
with optional --from-turn. Only --mark-reviewed D0005 or --mark-unreviewed
D0005 changes the family-level owner review ledger. Technical assessments,
viewer exit, verification and issue closure never imply that the owner watched
or accepted the encounter. Marks are local and do not rewrite assessment.json.
./produce_death.py collects a bounded prospective set of new recorded games
when another death is needed. No automatic cohort expansion or issue posting.


## Shared discussion and required analysis

[Death review hub #3](https://github.com/diegoami/borg-rs/issues/3) is the
default owner-feedback thread for this Rust registry. Each new collected case
links it unless --death-issue/BORG_DEATH_ISSUE overrides it. Registry overrides
without their own discussion.json remain unassigned; no C thread is borrowed.
Collection/export does not complete analysis. On processing feedback, use
[REVIEW-TEMPLATE.md](REVIEW-TEMPLATE.md): overall picture, actual decision
timeline, brainstorming/candidate fit, prioritized next evidence and explicit
limits. Mark provisional sections and missing evidence honestly. Preserve
original run/build/identity, and distinguish player-known information, native
spell readiness and actual Borg decision eligibility. No tactic or promised
rescue follows automatically from filling the template.
