## Torch purchases stop at the projected fuel target (#1257)

The live town planner limits ordinary Wooden Torch offers to the number of
copies that cover the fuel shortfall. It applies the limit before budgeting
and repeats the calculation after each transaction using the fuel actually
carried. A partly used torch contributes its actual timeout. Gear upgrades
retain their equipment evaluation.

The recorded Priest already carried more fuel than the 20,000-turn target.
The old reserve replenishment path correctly declined an urgent torch, but
the ordinary light curve still valued fuel above the reserve. General Store
torches entered that solve as stackable supplies, so the positive marginal
value could justify buying more on each visit. The recorded branch of the
town plan that later offered an eight-torch purchase at 132,003 turns was
reproduced with the inventory reconstructed from the captured save.

In a native town reconstruction, four full spares with the 2,003-turn
equipped torch reduced the offer from 30 copies to zero. Four partly used
spares started with 14,003 turns and the Borg bought two torches, then
declined more at 24,003 turns. With a 40,000-turn trip it bought four and
declined more at 42,003 turns. These are new town decisions from the recorded
save. The original visits are documented in [#1257](https://github.com/diegoami/borg-reborn-rs/issues/1257).

The disposition of fuel already carried is tracked in
[#1256](https://github.com/diegoami/borg-reborn-rs/issues/1256).
