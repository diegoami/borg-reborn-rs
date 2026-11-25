## Balance the kit on town arrival (#1256)

Track P now unloads at the start of a town visit. The Borg discards known
Sleep, Confusion and Slowness potions first, then compares surplus relative
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

Angband scatters drops made from stairs and those made next to piles. The
Borg uses a clear floor tile and marks the dropped stack through the
native item-specific ignore command so it stays out of automatic pickup.
Unknown potion flavours retain the existing identification and sale rules.

The regression uses the captured Half-Elf Priest inventory and restores
the oil and shot from the recorded discard sequence for a fresh-Borg
dungeon check. The source and its checksum are in
[#1256](https://github.com/diegoami/borg-reborn-rs/issues/1256).
Stopping torch purchases at the fuel target is tracked in
[#1257](https://github.com/diegoami/borg-reborn-rs/issues/1257).
