# 2026-11-24: Combat healing before an exposed turn (#1194)

Track P, based on `track-p` at `37b50104`.

With `caution_combat_heal = 1`, the Borg picks the weakest carried wound cure
that leaves it above the next monster damage estimate. It uses reachable
attacks, observed haste, and plausible landing grids for known teleport-to
pulls. Cure Critical Wounds also clears existing poison. The 20% wound
threshold for sustained combat is a hypothesis. A lethal estimate can
override it.

A suitable potion can take the turn before a Phase refusal attack, an
unchecked Phase Door after safer escape attempts, a pending Recall wait,
or an early winning fight. Poison triage checks lethal combat exposure
before spending an action on a poison-only cure. The switch defaults to 0.

The native fixtures reconstruct the decisions using the game data and the
published combat records. The original-command controls exercise the Phase
Door, Wand of Wonder, Neutralize Poison, and melee choices. The enabled
cases cover early sustained healing, potion strength at 89, 38, and 9 HP,
healing while Recall is pending, both Wormtongue decisions, and the Rogue's
winning fight verdict. [Tengu](https://github.com/diegoami/borg-reborn-rs/issues/1194)
· [Wormtongue](https://github.com/diegoami/borg-reborn-rs/issues/1186#issuecomment-7136040257)
· [Zombified orc](https://github.com/diegoami/borg-reborn-rs/issues/992#issuecomment-7136017013)

Flight Recall initiation belongs to the comparison in
[#1144](https://github.com/diegoami/borg-reborn-rs/issues/1144).

Verification passed for all eight enabled native decision cases and four
disabled controls. Rust formatting, Clippy, workspace tests, core tests,
parameter citations, and the vendor check passed. All four shipped
borg-rs smoke runs matched the reference keys, messages, and state.
