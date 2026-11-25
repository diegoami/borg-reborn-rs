# #1265 Remove sandal-specific junking

The generic identified-spare comparison stays in place. The Leather Sandals exception and its setting are removed. A legacy setting is accepted and ignored with the existing retired-parameter diagnostic so an older profile still loads.

The redesign is tracked in #1264. The existing town-arrival and dungeon disposal rules use different eligibility checks. In the dungeon, priority disposal requires an identified same-kind pair and a stack worth no more than 15 gold. Town arrival can drop plain, worse gear with unknown runes, regardless of the dungeon's identification gate. The full-pack fallback also spends some selected consumables to make space and has separate protections for unknown gear. The fallback's initial desperate bonuses for boots, weapons held while digging, and unknown runes are overwritten by its generic 7,000 bonus before later checks run.
