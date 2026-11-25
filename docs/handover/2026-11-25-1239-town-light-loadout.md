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

### Review follow-up

The planner protects displaced gear only after it observes the matching
quantity increase at Home. A rejected deposit leaves the gear in the pack
and available to the normal disposition rules. The armor, damage, and
positive hit floors compare against half of the original value without
rounding the threshold down.

Restoring an upgrade from Home requires one free pack slot at checkout.
A full pack defers restoration until ordinary inventory handling opens a
slot.

The full-Home, trim, full-pack, and restoration setups each changed
equipment power from 436,786 to 436,486 on the lighter swap. Restoration
after a strength gain went from 436,950 to 437,250. The no-storage case
proposed no exchange. The rejected-deposit regression proposed the same
300-point loss but confirmed no deposit. The successful light swaps used
one tenth of the 3,000-point trial allowance. These fixtures exercise an
acceptable loss and do not calibrate the upper bound.

[Review on PR #1242](https://github.com/diegoami/borg-reborn-rs/pull/1242#issuecomment-7154421823)

The re-review found a plan could select gear already rejected by Home.
The candidate check now consults the same recorded rejections as the
deposit step. [Re-review](https://github.com/diegoami/borg-reborn-rs/pull/1242#issuecomment-7157287113)
