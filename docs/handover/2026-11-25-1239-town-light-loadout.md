## 2026-11-25: Make room with a lighter town loadout (#1239)

The live planner uses the selected urgent supplies' weight and pack slots
when evaluating a lighter weapon or armor piece. It applies the existing
plain ammunition reserve first. The trial evaluates the kit after the old
gear has left the pack, preserving effective speed, worn resistances,
usable attacks and casting ability.

The exchange observes each move before continuing. A single-item Home
withdrawal can open the slot for the displaced piece. A single pack
replacement can be worn in a full pack because it frees its own slot.
When there is no safe exchange, ordinary town planning continues.
Stashed heavy gear remains protected during the run. A later visit can
retrieve it if the resulting kit keeps the departure reserve.

Five native scenarios exercise the reported CL20 Priest kit, a full
Home, a full pack, the no-storage fallback and restoration after a
strength gain. In the trim scenario, the pack was still 4.0 lb over
the allowance after dropping 37 pebbles. It kept 20 iron shots and
retrieved Word of Recall after depositing the mace. The original
captured save was unavailable in the workspace.

`town_loadout_power_loss` ships at zero. The constructed scenarios use
3,000 equipment power as a trial tolerance. The shipped class quiver
reserves follow [#1171](https://github.com/diegoami/borg-reborn-rs/issues/1171).
The [class and level calibration](https://github.com/diegoami/borg-reborn-rs/issues/1199)
continues to own the reserve policy.
