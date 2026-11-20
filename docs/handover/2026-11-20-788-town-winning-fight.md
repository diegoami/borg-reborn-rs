## 2026-11-20: town kill on sight shares winning fights (#788)

Branch `task/T788-town-winning-fight`, worktree `.worktrees/T788-town-fight`,
from main fcaecf2b. Owner requested implementation and applicable town
entries in the common kill-on-sight list. Existing compiled0/shipped1
`caution_winning_fight` is reused, with no new town-only knob.

The main think loop already included town; the missing paths were legacy
town bonuses and hunts. A shared non-acting certificate now backs the
early action and every continued town GOAL_KILL step. Listed town targets
require an actual winning fight and the one-turn/other-threat exception.
Town utility bonuses cannot bypass it. The list adds mercenary/veteran
(melee danger) and rogue/urchin (item/gold theft), with engine citations.
Town rows retain ordinary scares for losing/unknown situations. Theft is
unmodelled and cannot force a certificate. Harmless zero-dice NONE
bystanders are distinguished from unknown threats using the engine handler.

Walking costs are charged before combat with healing during approach;
attacks must already be available. A stale/empty aiming target, speculative
future melee attack, bump, door/trap/web/shop interaction cannot borrow a
win. This deliberately limits some safe approaches. Modes0/2 retain legacy
town actions; ordinary shop/item/stair/flight flows are outside this gate.

Tests/checks, exact final head and review prompt will be recorded on the PR;
canonical gameplay evidence belongs to Cemetery. No local population arm or
Zoo dispatch. Prior Zoo#511/#528 remain pinned to their original f827
specimen and must not silently include this change. Population benefit is
not claimed. This new task needs owner-selected review before any merge;
the earlier merge instruction applied to#763/#764, not automatically#788.


Validation is complete after integrating accepted main d7e58cdf (#787
weighted flight and#785 Paladin Phase reserve). The automatic source merge
left the new guard on the old direction-index interface; the first build
failed before games, then the guard was adapted to the selected location.
All required local checks were rerun:537 core tests/four ignored, workspace,
fmt, Clippy, vendor and2266 citations0wrong/0CALdisagreements pass. Default
borg-rs smoke is3/3 keys/states/messages identical to accepted main, all alive;
no reference re-record was needed. Clean frozen candidate5f41b563 sourcehash
18d8aa4a39136f8342f1e377f4a28eb87cb6f6024391ad7955e57b986d897949.

Twelve named100k replays (Warrior101, Mage102, original Ranger807683971),
with exact d7 main controls, all reach the cap alive. Modes0/2 each preserve
baseline0 keys/states on101/102; enabled Ranger is byte-identical. Nine new
notes are hunt refusals, with no positive town priority action in these
replays; positive/wand runtime coverage remains a declared limit. Enabled
Warrior/Mage first key differences and contexts are retained, including a
later DL2 cure-timing difference, not attributed to a direct town kill.
Canonical evidence: [Cemetery#95](https://github.com/diegoami/borg-cemetery/pull/95),
75ec9c68, `studies/reborn-town-winning-fight-20261120/`;12 named+3 smoke
attempts and the pre-game build failure are all accounted for,0natural
deaths.1737 retained dossier identities verify; no dossier added.

Hunt checks are conservative: a town GOAL_KILL flow without a certified
listed seed is declined, including later generic resident hunts/aiming
positions; incidental ordinary attacks remain possible. This can forgo safe
pursuits. Owner-selected review and later population measurement remain
on#788; no merge or Zoo dispatch. Prior Zoo#511/#528 inputs are unchanged.
