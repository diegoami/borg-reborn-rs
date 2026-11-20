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
