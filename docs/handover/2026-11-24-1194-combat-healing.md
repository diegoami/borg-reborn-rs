# 2026-11-24: Combat healing before an exposed turn (#1194)

Track P, updated to `track-p` at `11214501` after the rereview.

With `caution_combat_heal = 1`, the Borg picks the weakest carried wound cure
that leaves it above the next monster damage estimate. It uses reachable
attacks, observed haste, and plausible landing grids for known teleport-to
pulls. Cure Critical Wounds also clears existing poison. The 20% wound
threshold for sustained combat is a hypothesis. A lethal estimate can
override it.

Ordinary healing uses the combat forecast for wound potions, including
while Recall counts down. A suitable potion can also take the turn before
a Phase refusal attack, an unchecked Phase Door after safer escape
attempts, or an early winning fight. Poison triage checks lethal combat
exposure before spending an action on a poison-only cure. The switch
defaults to 0. [Tengu review](https://github.com/diegoami/borg-reborn-rs/issues/1194#issuecomment-7137413179)

The native fixtures reconstruct the decisions using the game data and the
published combat records. The original-command controls exercise the Phase
Door, Wand of Wonder, Neutralize Poison, and melee choices. The enabled
cases cover early sustained healing, potion strength at 89, 38, and 9 HP,
healing while Recall is pending, both Wormtongue decisions, and the Rogue's
winning fight verdict. [Tengu](https://github.com/diegoami/borg-reborn-rs/issues/1194)
· [Wormtongue](https://github.com/diegoami/borg-reborn-rs/issues/1186#issuecomment-7136040257)
· [Zombified orc](https://github.com/diegoami/borg-reborn-rs/issues/992#issuecomment-7136017013)

At the recorded 32 HP reply, a light cure brings 38 HP to 58 and covers
the reply. At 9 HP it only reaches 29, so the regression expects a serious
cure. The review's serious-cure result at 38 HP applies the original level
and wound bands.

The early Phase Door veto at full HP requires a retreat decision that
accounts for pull exposure over time. Flight Recall initiation also needs
to price the pull and the turns spent taking cures during the countdown.
[Early escape replay](https://github.com/diegoami/borg-reborn-rs/issues/1194#issuecomment-7134454211)
· [Flight Recall comparison](https://github.com/diegoami/borg-reborn-rs/issues/1144)

Verification passed for all eight enabled native decision cases and four
disabled controls. Rust formatting, Clippy, workspace tests, core tests,
parameter citations, and the vendor check passed. All four shipped
borg-rs smoke runs matched the reference keys, messages, and state.


The latest-base run repeated the fixtures and smoke comparison. A local
paired check loaded the archived Tengu save at turn 278,453 with the saved
game RNG and recording settings. Both arms initialized fresh Borg memory.
At their first differing decision (278,736, 180/225 HP), the enabled arm
drank Cure Light Wounds and the control attacked. At the common stop turn
279,657 on DL5, the enabled arm had 202 HP and had used 23 light cures. The
control had 41 HP and had used five. [Tengu case](https://github.com/diegoami/borg-cemetery/tree/23d1497a5fc2ccb8fad4a09ce9d18829524434fc/incidents/2026-11-24-watch-half-troll-warrior-tengu-279663)
· [Rereview](https://github.com/diegoami/borg-reborn-rs/issues/1194#issuecomment-7139017399)
