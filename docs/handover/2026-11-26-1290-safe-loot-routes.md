## Route stair-scum loot through legal steps (#1290)

The loot flow checks forward step legality while spreading backwards from
candidate items. It applies the same rule when picking a step from the
committed flow, including when continuing it after the map changes.

The native regression covers a legal detour around a shorter unsafe route
and an item with no legal route. All four shipped borg-rs smoke references
matched in keys, messages and state. On Track P, Necromancer seed 300036923
matched the #1260 fix's key stream. On the original `242a4724` build with both
fixes applied, the Borg ascended at turn 29,142 and reached CL6/DL2 by the
100,000-turn cap.

The spread uses the map at commit time. On later turns, the selector checks
the immediate step against the current map. The regression also removes
the legal next step after committing a route and verifies that selection
yields. Ordinary item flows still use their existing spread.
