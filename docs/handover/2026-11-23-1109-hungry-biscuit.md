## 2026-11-23 · Hard Biscuit at Hungry (#1109)

The existing Hungry path now eats a known Hard Biscuit when its earlier remedies
are unavailable. The spell, scroll, potion, named food, and Waybread activation
priorities are preserved. The Hungry food selection uses the shared aware-pack
lookup. The engine's Hard Biscuit adds 10 nutrition and emits a crunch message
(`engine/angband/lib/gamedata/object.txt`). A regression covers biscuit selection,
the priority of a known ration, and skipping unknown food and Scrap of Flesh.

Formatting, Clippy, the workspace and release core tests, parameter citations,
and the vendor check passed. All four shipped borg-core smoke runs matched the
tracked keys, messages, and state on the fix commit `f0e13e99`.

The town planner forecasts food from exploration and stair travel, with a
five-ration (150 nutrition) floor for short forecasts. Its forecast has no
breadth-first adjustment. The recorded Mage had biscuits at Weak, so the
incident alone cannot establish that the reserve needs to rise.

[borg-zoo #729](https://github.com/diegoami/borg-zoo/issues/729) requests the
matched breadth-first and control arms on the fix commit. The results will show
trip duration, departure nutrition against forecast, retail purchases, Home
withdrawals, and food-driven returns. The owner starts the runs. The reserve
decision follows the measured results.
