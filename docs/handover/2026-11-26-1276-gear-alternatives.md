# Clear redundant equipment across pack and Home

`junk_gear_alternatives` defaults to 2 when `junk_exploration_first` is on.
Set it to 3 for another distinct backup or to 0 to replay the earlier
cleanup. Comparisons use worn equipment, the pack and Home. The candidate
stack contributes no alternatives. Equivalent items share an option and
prefer worn gear, then Home, then the earlier pack position. Duplicates
within a stack can leave a usable copy behind. Rings can use both finger
slots, so two equivalent ring copies can be useful.

Melee comparisons use the character's known loadout to check accuracy,
damage, blows, speed, armour and burdens. Armour comparisons preserve
weight advantages. Brands, slays, other properties, launcher ammunition
types, and distinct digger roles can keep an alternative useful. Unknown
runes, marked items, artifacts and activations receive conservative
treatment. Fueled lights still follow their light and fuel policy.

The spare's value determines whether a shop sale is practical. In the
dungeon, a safe drop does not depend on the stack's gold value or a full
pack. Town still honors immediate shopping and active loadout changes.
Home cleanup uses the existing withdrawal and follow-up transaction.

The private Blackguard checkpoint from [#1276](https://github.com/diegoami/borg-reborn-rs/issues/1276)
shows the tradeoff. Within the early replay window, the earlier rules
leave all three plain stacks, two alternatives clear all five daggers,
both spears and all three robes, and three alternatives clear the daggers
and reduce the robe and spear stacks to one apiece. The checkpoint is
local and runs with `python3 engine/tests/check_spare_gear.py /path/to/bundle`.
The public tests cover cross-location counts, tie handling and a distinct
resistance role. The owner still needs to review the task PR before merge.
