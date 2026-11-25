## Balance the kit on town arrival (#1256)

Track P now unloads at the start of a town visit. The Borg ignores known
harmful consumables and devices using their effects. The rule covers
harmful potions, scrolls that summon enemies or curse equipment, mushrooms
that drain stats or cause terror, and devices that strengthen monsters.
It then discards
cursed weapons and armour and plain or low-value spares outclassed by
worn gear, including spares with unknown runes. Viable lighter swaps and
known special equipment remain available. The Borg compares surplus relative
to the existing supply targets across the pack and quiver. It refreshes
the inventory after each useful copy. Torch coverage uses remaining fuel
turns and the ranged allowance keeps independent throwing oil even when
the character also has usable ammunition. The dungeon fallback uses the
same ordering while the load costs speed.

The arrival pass aims for the configured free carry weight. When a kit
is still overloaded, it can exchange a weapon or armour piece for a viable
lighter replacement and deposit the displaced gear at Home. Ordinary
shopping begins after arrival processing. The planner and the store
executor both cap purchases and Home withdrawals at the carry allowance.
When no safe exchange can resolve the overload, the visit can finish
without acquiring additional stock.

Angband scatters manual drops made from stairs and those made next to
piles. The Borg uses a clear floor tile and marks these drops through
the native item-specific ignore command. Ignoring a harmful kind also
blocks later automatic pickups of that kind. Unknown flavours retain the
existing identification and sale rules.

The regression uses the captured Half-Elf Priest inventory and restores
the oil and shot from the recorded discard sequence for a fresh-Borg
dungeon check. The source and its checksum are in
[#1256](https://github.com/diegoami/borg-reborn-rs/issues/1256).

The reconstructed dungeon kit shed the harmful potions and 11 torches
while keeping all 12 flasks of oil and its shot. In town it shed 21
torches and shopped with 5 lb of headroom. The General Store torch offer
remained at zero. A larger native fixture also cleared all 14 Sleep and
22 Confusion potions plus examples of harmful scrolls, mushrooms, wands
and staves. The full-Home regression observed the lighter exchange and
deposit before the Recall acquisition. With no lighter gear at Home, the
Priest made room at Home, bought and wielded a Trident from the Weaponsmith,
and deposited the displaced mace before taking Recall within the allowance.

Stopping torch purchases at the fuel target is tracked in
[#1257](https://github.com/diegoami/borg-reborn-rs/issues/1257).

The first-town Ranger Phase Door fixture uses the historical shopping
order and weight policy. Its original purchase and first-contact
assertions passed for both the zero-scroll and two-scroll reserves.

The departure weight buffer is set per class in tenths of a pound.
The agreed range is 60 for Mage through 150 for Warrior. The intermediate
values are Necromancer 70, Druid 80, Priest 90, Rogue 100, Ranger 110,
Paladin 120, and Blackguard 135.
