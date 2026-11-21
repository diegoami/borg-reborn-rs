# Known C Borg bugs (reproduced faithfully)

Unless explicitly corrected under `docs/FORK-CORRECTIONS.md`, the Rust port
keeps these as C behaves. Entries preserve the original findings and identify
the owner-authorized exceptions so changes are deliberate and reviewable.

This file is not where new findings get catalogued any more. It is the
record of what was found through M5; existing sections only grow a new
"MilestoneN" bullet when a better-borg-c adoption supersedes them (see
[C-CONSUMER-ADOPTION.md](C-CONSUMER-ADOPTION.md)). A new suspected C defect
is coordinated through the C consumer's own ownership/tracker instead of
being added here fresh.

The seven milestone2 carries in [adoption/c-m2](adoption/c-m2/README.md)
correct selected findings below in local C and Rust. Their source ownership
and historical descriptions remain; consumer adoption is not upstream closure.

The milestone3 carries in [adoption/c-m3](adoption/c-m3/README.md)
(better-borg-c `702f091f6`, [Rust#8](https://github.com/diegoami/borg-rs/issues/8))
correct further findings below; each affected section has a "Milestone3"
bullet. Fixes the C lane found after these sections were written (caster
grid in `borg_projectable_pure()`, glow-array bound in `borg_flow_light()`,
trapped walls counted once in `borg_surrounded()`, both lunal flags cleared
on the town/weak scum bail) are listed in that batch's ledger.

The milestone4 carries in [adoption/c-m4](adoption/c-m4/README.md)
(better-borg-c `70993393a`, [Rust#11](https://github.com/diegoami/borg-rs/issues/11))
are marked with "Milestone4" bullets in the same way.

The milestone5 carries in [adoption/c-m5](adoption/c-m5/README.md)
(better-borg-c `5384f12ed`, [Rust#13](https://github.com/diegoami/borg-rs/issues/13))
are marked with "Milestone5" bullets. Two of them have no section here: the
endgame recall-depth prompt answers `y` at max depth 99 with few uniques left
(source310), and flows that skip the danger check no longer mark grids as
checked (`borg_flow_ignores_danger`, source404).

The milestone6 carries in [adoption/c-m6](adoption/c-m6/README.md)
(better-borg-c `b3c825c97`, [Rust#15](https://github.com/diegoami/borg-rs/issues/15))
are marked with "Milestone6" bullets.

The milestone7 carries in [adoption/c-m7](adoption/c-m7/README.md)
(better-borg-c `3573d741e`, [Rust#17](https://github.com/diegoami/borg-rs/issues/17))
are marked with "Milestone7" bullets. Two of them replace retained local
corrections: 0068 supersedes #287 and 0065 supersedes #345.

The milestone8 carries in [adoption/c-m8](adoption/c-m8/README.md)
(better-borg-c `017e038e8`, [Rust#19](https://github.com/diegoami/borg-rs/issues/19))
are marked with "Milestone8" bullets. The same target also carries the
owner's munchkin-by-class change (PR302), which is not a C bug fix.

The milestone9 carries in [adoption/c-m9](adoption/c-m9/README.md)
(better-borg-c `33207fbee`, [Rust#21](https://github.com/diegoami/borg-rs/issues/21))
are marked with "Milestone9" bullets. 0098 replaces the local #295
correction; 0098 and 0099 are each carried with a local crash guard.

The owner-directed v11R carry in [adoption/c-v11r](adoption/c-v11r/README.md)
(the consumer's `4.2.6-impact-v11`, 18 units 0100-0117) is marked with
"v11R" bullets. It is the Rust counterpart of the consumer's milestone 11,
not an automatic post-M10 adoption queue. The C Borg sources in this tree
were updated to the same consumer blobs so the existing C oracle can check
the Rust changes.

Format: file, function, (seed, turn when observed), explanation.

Each slip is also filed as a lead in
[diegoami/neo-angband-tracker](https://github.com/diegoami/neo-angband-tracker)
(see `CLAUDE.md`, "C bug leads are filed in the tracker"). Sections 1-41
were imported there by the tracker itself as CBUG-0001 to CBUG-0152 (from
commit `c375a8245`). From section 42 on, each section has a `Tracker:` line
with its issues.

## 1. `borg.has[BI_CURHP]` instead of `borg.trait[BI_CURHP]`

* File: `angband/src/borg/borg-fight-defend.c:2498`,
  `borg_defend_aux_banishment()`
* Seen: by code reading during M2/M3 (not tied to a seed). The banishment
  path needs a LOSKILL activation item, which the suites have probably never
  reached.
* `using_artifact = borg_equips_item(act_loskill, true) && borg.has[BI_CURHP] > 100;`
  `borg.has[]` is indexed by object kind, so `BI_CURHP` is treated as a
  k_idx. The test reads how many of some unrelated object kind the Borg
  carries, when the intended check is surely "current HP > 100" (as the
  comment-free sibling checks elsewhere do with `borg.trait[BI_CURHP]`).
* Rust (`borg_fight_defend.rs`): `st.borg.has[BI_CURHP as usize]` (same
  index into the same table).
* Milestone8 ([Rust#19](https://github.com/diegoami/borg-rs/issues/19)): source37 reads `borg.trait[BI_CURHP]` (patch 0078), in C and Rust.

## 2. Copy-and-paste slips in sval lookups

* `borg-item-val.c`, `borg_init_item_val`, all seeds, startup:
  `sv_wand_confuse_monster` is looked up as "Scare Monster" (the same as
  `sv_wand_fear_monster`) and `sv_wand_elec_ball` as "Lightning Bolts" (the
  same as `sv_wand_elec_bolt`); copy-and-paste slips. `sv_mush_cure_mind` is
  "Clear Mind" like `sv_mush_mana` and `sv_potion_berserk` is looked up twice
  (both harmless). Kept as is in `borg_item_val.rs`.

## 3. Last token of each block not ordered (formulas)

* borg-formulas-calc.c, `adjust_order_operations()`: the blocks passed to
  `adjust_order_ops_depth_block(f, pdepth, start, itoken - 1)` are scanned
  with `tok < end`, so the last token of each block is never looked at.
  Harmless for well-formed formulas (a block ends in a value); a malformed
  one such as `(1 + 2 +) 3` parses and evaluates to 3 with "error
  calculating a formula value" notes. Only with `borg_uses_dynamic_calcs`;
  not seen in any suite. Pinned by a unit test in `borg_formulas_calc.rs`.
## 4. "West of the player" lights the wrong grids

* `borg-cave-light.c`, `borg_update_light`, every run with a radius-2+
  light (the full suite reaches this branch thousands of times).
* The "East" block lights (y, x+2), (y+1, x+2), (y-1, x+2), but the "West"
  block lights (y, x-2), (y+2, x-2), (y-2, x-2) instead of (y+1, x-2) and
  (y-1, x-2). So (y+-1, x-2) never get `BORG_LIGHT`, and (y+-2, x-2) can be
  added a second time to `borg_light_x/y` (already added by the South/North
  blocks), contrary to the macro's comment. Also, with radius 4+ the four
  (y+-3, x+-3) grids can be entered twice (once by the diagonal checks and
  once by the box scan). Duplicates only make readers of the array
  (`borg_flow_dark`, `borg_update`) visit a grid twice.
* Rust: kept as is in `borg_cave_light.rs`.
* Milestone8 ([Rust#19](https://github.com/diegoami/borg-rs/issues/19)): source40 lights (y+1, x-2) and (y-1, x-2) (patch 0081), in C
  and Rust.

## 5. Spell status set from a bool; books 0-7 only (magic)

* `borg-magic.c`, `borg_init_spell()`, all caster seeds, startup:
  `spell->status = spell_okay_to_cast(player, spell_num);` stores a `bool`
  in a field that holds `BORG_MAGIC_*` values, so a spell starts as 0
  (`BORG_MAGIC_ICKY`) or 1 (`BORG_MAGIC_LOST`) until
  `borg_cheat_spells()` sees its book in the pack. Harmless in practice:
  every reader also requires the book to be carried, and carried books are
  re-cheated first.
* `borg_cheat_spells()` resets `borg.book_idx[0..9]` but only cheats books
  0..7 (`book_idx < 8`); no class has a ninth book.
* `borg_spell_stat()` tests `&books[0].spells[0] != NULL`, which is always
  true.
* Rust (`borg_magic.rs`): all kept (`u8::from(status)`, `0..8`, no check).
* Milestone9 ([Rust#21](https://github.com/diegoami/borg-rs/issues/21)): source296 applies the Necromancer penalty on any lit square
  (`square_islit`, patch 0099), in C and Rust, with a local guard: the
  position must be in bounds, as just after a level change it can still be
  on the previous level (FORK-CORRECTIONS; better-borg-c#377).

## 6. Formula error text lost to `format()`'s static buffer

* borg-formulas.c, `parse_value()` and `get_value_index()`, only when a
  borg.txt formula has a bad `value()` and `borg_uses_dynamic_calcs` is on
  (by code reading, then reproduced with the C file and stubs).
* Three callers pass `format(...)` as the `error` argument of
  `borg_formula_error()`: "value type (first parameter '%s') not found" and
  (twice) "value error for object type '%s' subtype '%s'". `format()`
  returns its static buffer, and `borg_formula_error()` formats its own
  notes with `format()` before it notes `error`. So the error note repeats
  the "** error for section value '...'" note and the caller's message is
  never shown. (That message also has an off-by-one: it prints
  `value_type_names[t].name`, but the table starts at `VT_NONE`, so it is
  the name of type `t - 1`, e.g. 'scroll' for a potion.) If the "error on
  line" note grew `format()`'s buffer (a line of more than ~1000 bytes), C
  would read freed memory instead; the Rust port assumes it does not.
* Rust: `formula_error_format()` in `borg_formulas.rs` notes the "error for
  section" text twice. Pinned by `errors_match_c`.

## 7. `range()` errors (formulas)

* borg-formulas.c, `parse_range()`, only with `borg_uses_dynamic_calcs`.
  A missing ')' is reported as "** first value is not a number", and a first
  value that really is not a number fails with no note at all, so the
  formulas are turned off silently. (The section printed in these errors is
  cut at the ',', where C wrote a NUL.) Kept as is.
## 8. Home power: spell-stat drop at +26 and a dead neck branch

* `borg-home-power.c`, `borg_power_home_aux1()`, by code reading (no suite
  run stores +26 of the spell stat at home; checked by fuzzing the c2rust
  version against the Rust one in-game).
* Spell stat: the last tier is `20 * 100L + 6 * 300L + (s - 26) * 5L`; the
  tier below it starts from `20 * 400L`. So the home value of the spell stat
  falls from 9500 at +25 to 3800 at +26, surely meant to be `20 * 400L`.
* Amulets: `if (num_neck > 3) ... else if (num_neck > 3) value -= (num_neck
  - 3) * 700L;` repeats the condition, so the 700 penalty never applies
  (the other item types use a lower second threshold, e.g. rings 6 / 4).
* Rust: both kept in `borg_home_power.rs` (the dead branch is left out, as
  clippy rejects an `if` with the same condition twice; until milestone8 a
  comment marked it, and 0083 removes that comment).
* Milestone8 ([Rust#19](https://github.com/diegoami/borg-rs/issues/19)): source47 removes the dead `else if (num_neck > 3)` branch
  (patch 0083, an owner-decided cleanup with no behaviour change).
## 9. Dark-grid test uses `==` instead of `&` (magic play)

* `borg-magic-play.c`, `borg_play_magic()`, every call (study and town
  spell testing): `if (borg_grids[borg.c.y][borg.c.x].info == BORG_DARK)`
  compares the whole info byte, so the "Dark" early return only fires when
  `BORG_DARK` is the grid's only flag. Every other reader of `BORG_DARK`
  (borg-commands.c, borg-think-dungeon-util.c) tests the bit. The player's
  grid normally also has `BORG_VIEW`/`BORG_MARK` etc., so in practice the
  check almost never triggers; `BI_LIGHT` is the real guard.
* Rust: kept as `== BORG_DARK` in `borg_magic_play.rs`.

## 10. Unreachable and mislabelled preparedness checks (prepared)

* borg-prepared.c, `borg_prepared()`/`borg_prepared_aux()`/`borg_restock()`,
  static (non-formula) mode; found by code reading, confirmed by a
  randomized old-vs-new check (traits, config, uniques and home counts
  perturbed, 81M calls) that never produced these texts.
* `borg_prepared()` calls `borg_restock()` first and returns "Clevel <
  depth" when `max clevel < depth` (below 50), so in `borg_prepared_aux()`:
  "1 Lite" and "5 Fuel" (light 0 is already "restock light radius < 1"),
  "2 tele + teleport staffs" (same test as "restock tele + tele staff < 2")
  and the dlevel 10-19 class checks "dlevel - 4 >= clevel" and "dlevel >=
  clevel" never fire. "2 cures (clw + csw + ccw)" repeats the "2 cure"
  test exactly. The level-100 block ("10 restore mana", which tests for 15,
  "5 Heal", "15 *heal* or life", "10 speed potions") is dead because
  `borg_prepared_aux()` is only called for `depth <= 99`.
* `borg_restock()`: "restock fuel" and "restock fuel < 2" need light 0,
  which already returned "restock light radius < 1".
* "Scumming *Heal* potions (%d to go)" tests the home counts from before
  `borg_notice_home()` but prints `10 - num_ezheal` from after it, and
  ignores the Life potions and the ones carried.
* Rust (`borg_prepared.rs`): all kept.
## 11. Respawn picks the race/class with the wrong menu letters

* `borg-reincarnate.c`, `borg_reincarnate_start()`, only when the Borg
  respawns (game option `cheat_live` on and `borg_cheat_death` off, or
  `borg_respawn_winners`). It types `all_letters[race]` and
  `all_letters[class]`, but the birth menus use `all_letters_nohjkl`
  (ui-birth.c). So index 7 (Half-Troll, Ranger) types 'h', which is no
  menu letter, and indexes 8 and up select the entry before the one meant
  (race 8, Dunadan, types 'i' and gets a Half-Troll; class 8, Blackguard,
  gets the Ranger; race 10, Kobold, types 'k', no menu letter either).
  The random name is still made for the race meant. Seen in local respawn
  runs (seed 1011: race 8 chosen, a Half-Troll with a human name rolled).
* Rust: kept (`engine::all_letters`) in `borg_reincarnate.rs`.
## 12. Home notice: item sustains all count as STR; banishment never counted

* `borg-home-notice.c`, `borg_notice_home_aux()`, by code reading (any run
  that keeps or wears a sustain item reaches the first part).
* Sustains on items: the five `of_has(item->flags, OF_SUST_*)` tests all
  add to `num_sustain_str`, so `num_sustain_int/wis/dex/con` only ever count
  the player's own race/class sustains (added at the end with `rf_has`),
  and `borg_power_home()` sees too many STR sustains.
* `num_genocide` is reset but never counted (scrolls of Banishment are not
  looked for), so the home power reward for them is always 0. (The
  `num_brand_weapon` global is never written at all.)
* Rust: all kept in `borg_home_notice.rs`.
* Milestone7 ([Rust#17](https://github.com/diegoami/borg-rs/issues/17)): source54 counts scrolls of Banishment into `num_genocide` (patch 0077), in C and Rust.

## 13. Log and status slips (log)

* `borg-log.c`, found by code reading.
* `borg_write_map()`: "Your hands are glowing dull red." tests
  `TMD_CONFUSED` again (it meant a different timed effect), so a confused
  Borg's map dump prints both lines. Seen in the full suite's death maps.
* `borg_write_map(true)` (the `^z` map command, not in any suite): when the
  file prompt is cancelled or the file cannot be opened, the file stays
  NULL. `file_putf()` ignores a NULL file, but `file_put()` (the "Swap
  ...: NONE" lines, with `borg_uses_swaps`) and the final `file_close()`
  dereference it, so the game crashes.
* `borg_display_status()`: for "Demon" and "Dragon" the slay's blue is
  always overwritten (`if (WS) blue; if (WK) green; else slate;`), so a
  plain slay shows as slate.
* Rust (`borg_log.rs`): all kept; the NULL-file crash is a panic.
## 14. `borg_dont_react` is never cleared

* `borg-messages.c` sets the global `borg_dont_react` on "You have been
  knocked out." and "You are paralyzed"; `borg_react()`
  (`borg-messages-react.c`) then queues nothing and only notes "# Ignoring
  messages.". The reset in `borg.c` (`borg_inkey_hack` path:
  `borg.dont_react = false;`) clears the unused struct field
  `borg.dont_react` instead, so once set the global stays true for the rest
  of the process: the Borg never again queues hit/miss/kill/SELF messages
  for `borg_update()`, and `borg_caution()` notes "# Borg ignoring messges."
  every turn. By code reading; the full suite never sets it (checked with
  a temporary log of the flag).
* Harmless, same file: `borg_react_prompted()` returns `key` (a pointer)
  from a `bool` function in three places; it is never NULL, so this is
  just `true`.
* Original Rust reset the unused struct field. Milestone2/source14 now resets
  `st.msgs.dont_react` after parsing in `borg.rs`. The unrelated prompted-key
  return convention remains. Adoption: [Rust#6](https://github.com/diegoami/borg-rs/issues/6).
* v12R ([adoption/c-v12r](adoption/c-v12r/README.md)): the defect is
  gone upstream and in the C oracle. The official import unified the gate on
  the borg struct field (`borg.dont_react`); the global `borg_dont_react` no
  longer exists, and the reset in `borg.c` clears the field that
  `borg_react()` reads, so the knockout window is transient. The Rust gate
  moves to `st.borg.dont_react` to match the C, and the separate copy in
  `MsgState` is removed.
## 15. Inscribing a quiver item types an equipment letter

* `borg-item-id.c`, `borg_object_fully_id()`, by code reading. The loop
  runs to `QUIVER_END`, and every slot from `INVEN_WIELD` up is selected
  with `'/'` and `all_letters_nohjkl[i - INVEN_WIELD]`. For a quiver slot
  that is index 12 and up ('q', 'r', ...), past the 12 equipment slots;
  the quiver is selected with `'|'` and digits (as `borg_deinscribe()`
  does). Only a quiver item with tval in `TV_BOW..TV_RING` (a throwing
  weapon) that is an artifact or an ego with a random power is affected,
  and the function is only called from the Borg's debug commands
  (`borg-commands.c`), so no suite reaches it.
* Rust: kept in `borg_item_id.rs`.

## 16. Slot lookups: dead "largest pval" test; empty pack has no hole (inventory)

* `borg-inventory.c`, found by code reading.
* `borg_slot()`: the "Prefer largest pval (even if smaller pile)" test
  skips an item only when `item->iqty > borg_items[n].iqty`, but the
  "Prefer smallest pile" test just before has already skipped every such
  item. So pval is never compared: of equally small piles the last one
  wins.
* `borg_first_empty_inventory_slot()` scans down from the last pack slot
  for the last used one and returns the slot after it. With an empty pack
  there is no used slot and it returns -1 ("no room"), not 0, so the
  buy/wear/remove code that needs a hole gives up. (Unlikely in play:
  the Borg nearly always carries something.)
* Rust (`borg_inventory.rs`): the pval slip is kept.
* v11R ([0111](adoption/c-v11r/README.md)): an empty pack returns slot 0,
  not -1, in C and Rust.
## 17. Danger slips (danger)

* `borg-danger.c`, found by code reading; all are in every danger
  calculation, so every suite runs through them.
* `borg_danger_physical()`:
  * COLD blows test the temporary *acid* resistance (`borg.temp.res_acid`),
    not `res_cold`.
  * LOSE_CON blows add the "below 8" fear when `BI_CSTR < 8` (strength).
  * HURT: the stun test `z > d_dice * d_side` comes right after
    `z = d_dice * d_side`, so it is never true (dead code).
  * The hit chance `150 - ((ac * 300) / 4 / 1 + (level + power) * 3)` is
    below 5 for any armour class of 2 or more, so partial damage
    (`full_damage == false`) is always 5% of the blow damage.
* `borg_danger_spell()`:
  * `bolt` is set by the first bolt spell and never reset, so every later
    spell of the same monster (bolt or not) has its damage zeroed when the
    monster has no clean shot at the Borg.
  * The summon cases' grid scan assigns to the `x`/`y` parameters
    (harmless: not read again). S_HI_DRAGON with protection from evil
    adds no fear before dividing; S_AINU tests `borg.near.unique` in both
    the first and the (then dead) second branch. STORM ends with a no-op
    `if (borg.trait[BI_RCONF]) break;`.
* `borg_danger_one_kill()`:
  * The second `borg_crush_spell` block (in the ranged part) clears `v1`
    again instead of `v2`.
  * The Teleport Other skip list is read with `ii <= borg_tp_other_n`, one
    entry past the count.
    The list is `int[255]`: with 255 entries C reads one `int` past the
    array; Rust panics there (the bounds check of
    `bridge::borg_tp_other_index`). Not reachable in practice: it needs
    255 Teleport Other candidates in one `borg_defend` scan.
  * The first move simulation tests `feat == FEAT_GRANITE`, the second
    `feat >= FEAT_GRANITE` (so it also treats terrain above the permanent
    wall, e.g. lava and passable rubble, as wall).
* `borg_danger()` overwrites its `full_damage` argument with `true`, so the
  partial-damage paths above are only reached through direct
  `borg_danger_one_kill(..., false)` calls.
* Rust: milestone2/source26 corrects the COLD temporary resistance in
  `borg_danger.rs`; the other findings remain.
* Milestone3 ([Rust#8](https://github.com/diegoami/borg-rs/issues/8)): source64 resets `bolt` per spell and source67 reads
  the Teleport Other list only below its count, in C and Rust.
* Milestone8 ([Rust#19](https://github.com/diegoami/borg-rs/issues/19)): source62 removes the dead HURT stun test (patch 0085, an
  owner-decided cleanup with no behaviour change).
* v11R ([adoption/c-v11r](adoption/c-v11r/README.md)):
  * [0103](adoption/c-v11r/README.md): the partial melee hit chance divides
    the armour term by `(level + power) * 3` and is capped at 100%, in C and
    Rust.
  * [0107](adoption/c-v11r/README.md): LOSE_CON fear tests `BI_CCON`, not
    `BI_CSTR`.
  * [0108](adoption/c-v11r/README.md): the ranged Crush check clears `v2`.
  * [0109](adoption/c-v11r/README.md): the ranged move simulation treats only
    granite (and the same seam types) as a wall, not everything above it.
  * The COLD, bolt/summon and dead-stun findings remain in the sections above.

## 18. Item value: stat bonuses tested as object flags (item analyze)

* `borg-item-analyze.c`, `borg_object_value_known()`, by code reading
  (every identified wearable with a non-zero pval goes through it).
* "Give credit for stat bonuses" tests `of_has(item->flags, STAT_STR)` ...
  `of_has(item->flags, STAT_CON)`: stat indices (0..4) used as object flag
  numbers. `STAT_STR` is 0 = `FLAG_END`, which `flag_has()` always reports
  as unset; `STAT_INT`..`STAT_CON` (1..4) are `OF_SUST_STR`..`OF_SUST_DEX`.
  So no stat bonus is ever credited (stat boosts are modifiers, not flags),
  while a known STR/INT/WIS/DEX sustain adds `pval * 200` to the value of
  an item that has a pval (and a CON sustain does not).
* Rust: kept (`engine::flag_has(&item.flags, STAT_*)`) in
  `borg_item_analyze.rs` until milestone8, which corrects it (patch 0082,
  bullet below).
* Milestone8 ([Rust#19](https://github.com/diegoami/borg-rs/issues/19)): source70 credits the positive stat modifiers themselves, and
  no longer skips every credit when `pval` is 0 (patch 0082), in C and Rust.
## 19. Object tracking and item flows (flow take)

* `borg-flow-take.c`, found by code reading.
* `borg_flow_take()`, `borg_flow_take_scum()`, `borg_flow_take_lunal()`:
  the "nearest up stairs" scan skips a stair when `b_j >= j`, so it keeps
  the *farthest* up staircase (the first of equally far ones). The leash
  and "too far from the stairs" tests use that one.
* `borg_flow_take_lunal()`:
  * `if (!k_ptr) continue;` tests `&k_info[take->kidx]`, which is never
    NULL, so dead entries below `borg_takes_nxt` are not skipped: their
    location (0,0) becomes a target like any other (when in view and not
    too far from the stairs).
  * `item_bad` is worked out (with its "Lunal Item" notes) but never
    read: every entry that passes the view and stairs tests is a target,
    worthless ones (value <= 0) included.
  * Only reached in lunal/munchkin mode (`borg_lunal_mode`,
    `borg_munchkin_start`), which no suite turns on.
* `borg_follow_take()`: an object under the Borg is deleted, but the
  function goes on with the wiped entry: the "on screen" test reads grid
  (0,0), and if that grid is `BORG_OKAY` (and no monster on the object's
  old grid stops it) it notes "There was an object ..." and deletes the
  entry again (a no-op). Only an extra note.
* `observe_take_move()`: the attr test only applies when the kind's char
  is `'!'` and its attr is not 11; for every other kind the attr is not
  compared at all.
* Rust (`borg_flow_take.rs`): the remaining findings are kept.
* Milestone8 ([Rust#19](https://github.com/diegoami/borg-rs/issues/19)): source72 skips dead entries (`kidx == 0`) in
  `borg_flow_take_lunal()` (patch 0088), in C and Rust.
* v11R ([adoption/c-v11r](adoption/c-v11r/README.md)):
  * [0104](adoption/c-v11r/README.md): the three flows keep the *nearest*
    up staircase (first of equally near ones), in C and Rust.
  * [0101/0101a](adoption/c-v11r/README.md): the Lunal take routine skips an
    `item_bad` object outside munchkin mode. The runtime `borg.munchkin_mode`
    gate is used, not the class/config flag.
  * The `k_ptr`, `borg_follow_take` and `observe_take_move` findings remain.

## 20. Device use slips (item use)

* `borg-item-use.c`, found by code reading.
* The device fail check `100 * ((skill - lev) - 140) / ((lev - skill) - 90)`
  (rods, staffs, rings, `borg_activate_failure`) divides by zero when the
  item level is exactly 90 above the device skill (a crash in C, a panic
  in Rust). Only `borg_equips_dragon()` guards against it. Needs a very
  low skill (e.g. a confused low-level character) and a level 90+ device.
* `borg_quaff_crit()`: "Avoid drinking CCW twice in a row" returns false
  (74 times in 100) when the last CCW was quaffed *more* than 4 game turns
  ago; the extra `borg_timer(..) > 0` test is implied by `> 4`.
* `borg_equips_ring()` / `borg_activate_ring()` loop from `INVEN_RIGHT`
  to `< INVEN_LEFT`, so only the right ring is ever checked.
* `borg_use_staff_fail()`: the "desperation" Teleportation branch tests
  `fail > 500` again inside `fail > 500`, so a not-confused, not-blind Borg
  always gives up there. `borg_equips_staff_fail()` does the opposite: a
  not-confused Borg gives up when `500 < fail < 650` but accepts the
  staff when its chance is even worse (`fail >= 650`).
* `borg_eat_food_any()`: "when hurting" is `CURHP < 4 || CURHP <= MAXHP`,
  which is true whenever current HP is not above the maximum (so nearly
  always).
* `borg_use_things()`: the Enlightenment potion branch only `break`s
  (never quaffs); the comment says "Check the scroll".
* Rust: milestone2/source77 corrects CCW timing, and source78 includes both
  ring slots in both readiness and activation. Other findings remain in
  `borg_item_use.rs`.
* Milestone4 ([Rust#11](https://github.com/diegoami/borg-rs/issues/11)): source81 quaffs a recognised Enlightenment potion in
  the dungeon (never in town), in C and Rust.
* v11R ([0112](adoption/c-v11r/README.md)): `borg_eat_food_any()` uses
  `BI_CURHP < BI_MAXHP`, so a full-health Borg does not quaff cures for food,
  in C and Rust.
## 21. Equipment swaps and the "best" list (item wear)

* `borg-item-wear.c`, found by code reading.
* `borg_backup_swap()`:
  * "Pass on the swap which yields the best result" keeps the *higher*
    danger: `if (b_p1 <= b_p2) b_p = b_p2 (weapon) else b_p = b_p1
    (armour)`. So with both swaps the worse one is weighed and worn.
  * With an armour swap and no weapon swap, `b_p2` stays 0, so an armour
    danger of 0 picks `swap = weapon_swap - 1 = -1` and the key is
    `all_letters_nohjkl[-1]`, the byte before the array. In the C and the
    Rust harness binaries that byte is alignment padding (0, checked with
    `objdump`), so `borg_keypress` refuses it ("BAD KEY" note) and only the
    `'w'` is queued.
  * The `i == -1` fixes are dead (the swap index is non-zero there).
* `borg_one_step_wearing_best()`: `int skipping = skip;` sits inside the
  loop, so it is reset on every pass. With `skip > 0` (inventory or home
  full) every remaining entry is skipped instead of only `skip` of them:
  the function returns false (or, when it ends on an empty entry, clears
  the list but keeps `do_best`), so "try the next change" never tries one.
* `borg_best_stuff_aux()`: the "don't test the same item in both ring
  slots" test for `INVEN_LEFT` (order 8) reads `test[n + 1]`, the
  `INVEN_RIGHT` entry (order 9), which is only written deeper in the
  recursion: it holds the last right ring of the previous subtree, so a
  left-ring candidate equal to it is skipped for no reason.
* `borg_wear_stuff()`: `track_worn_time` is an `int16_t`, so
  `borg.time.now` is truncated; once the time passes 32767 the "worn
  more than 10 turns ago" test is true at once. The test itself skips an
  artifact worn on this level *more* than 10 turns ago (not recently).
* Rust (`borg_item_wear.rs`): milestone2/source82 chooses lower danger among
  available emergency swaps, including the zero-danger single-swap case.
  Other findings remain; `swap_letter(-1)` itself still gives the 0 key.
* Milestone7 ([Rust#17](https://github.com/diegoami/borg-rs/issues/17)): source85 sets `skipping` once, before the loop in
  `borg_one_step_wearing_best()` (patch 0075), in C and Rust.
* v11R ([0113](adoption/c-v11r/README.md)): the stale left-ring duplicate
  check in `borg_best_stuff_aux()` is removed; the left ring is chosen first,
  so only the right ring needs it, in C and Rust.
* v12R ([adoption/c-v12r](adoption/c-v12r/README.md)): the "keeps the
  higher danger" finding is fixed upstream (`bc67c099e`), and the consumer's
  carry 0001 is retired in favour of that upstream form. Both candidates now
  start at 9999, the early "make sure we have an appropriate swap" return is
  gone, and the lower danger wins with the weapon on a tie. The Rust
  `borg_backup_swap()` follows the same form, so the single-swap case no
  longer needs the `weapon_swap &&` guard and the zero-danger armour swap
  keeps its own index. C and Rust.

## 22. Light beam edge test and lantern refuel scan (light)

* `borg-light.c`, found by code reading.
* `borg_light_beam()`: "Don't do it if on the edge of shifting the panel"
  compares the Borg's absolute dungeon row `borg.c.y` with the panel
  height (`borg.c.y > borg_panel_hgt() - 2`), not its row on the panel,
  and uses that same "bottom edge" test for north (`dir == 8`) and south
  (`dir == 2`). So a north or south beam is only ever cast in the top
  rows of the dungeon (`borg.c.y <= borg_panel_hgt() - 2`).
* `borg_light_beam()`: the west test starts at step 1 while north, south
  and east start at step 0 (the Borg's own grid), so with a light radius
  of 0 the west loop never runs and never picks direction 4. The
  `blocked == false` test inside each loop is always true there.
* `borg_refuel_lantern()`: with no flask of oil, the lantern scan starts
  at `borg_slot(TV_LIGHT, sv_light_lantern)`, which (lanterns do not
  stack, all piles are 1) is the *last* lantern in the pack. Only that one
  is checked: if it is Everburning or empty, the next slot is not a
  lantern (or the pack ends) and no lantern is used, even when an earlier
  lantern has fuel.
* Rust (`borg_light.rs`): all kept.
* Milestone7 ([Rust#17](https://github.com/diegoami/borg-rs/issues/17)): source90 scans for the *first* lantern in the pack (patch 0076),
  in C and Rust.
* Milestone8 ([Rust#19](https://github.com/diegoami/borg-rs/issues/19)): source88 tests the Borg's row on the panel
  (`borg.c.y - Term->offset_y`) in `borg_light_beam()` (patch 0084), in C and Rust.
## 23. Curse removal item letters (item decurse)

* `borg-item-decurse.c`, `borg_decurse_any()`, found by code reading.
* `BI_FIRST_CURSED` is set by `borg_notice()` in the order equipment,
  quiver, inventory, so in the quiver branch `BORG_EQUIP` is never set and
  the `BI_WHERE_CURSED & BORG_EQUIP` test for the `'|'` key is dead (the
  `& 1` next to it is `BORG_INVEN` written as a literal).
* A cursed ammo stack in the pack (outside the quiver) sets `BORG_QUILL`,
  not `BORG_INVEN`. With such a stack and a cursed quiver slot first, no
  `'|'` is sent although the item prompt can open on the pack.
* `borg_decurse_armour()` / `borg_decurse_weapon()` send
  `all_letters_nohjkl[swap - 1]` without checking the swap: with a
  `decurse_*_swap` flag but `*_swap == 0` (not left behind by
  `borg_notice_*_swap`, which clears the flag when there is no swap) C
  would read the byte before the array (padding, 0, see section 21).
* Rust (`borg_item_decurse.rs`): all kept (`letter(-1)` gives the 0 key).
## 24. Monster tracking slips (flow-kill)

* `borg-flow-kill.c`, found by code reading.
* `borg_update_kill_old()`: the "unique is dead" fix-up writes
  `borg_race_death[i]` with the *kill* index `i`, not the race index.
* `borg_new_kill()`: `r_ptr` is taken from the slot's race *before*
  `kill->r_idx` is set, so the "force the grid to floor / granite" step at
  the end uses the old race of the slot (race 0 for a wiped slot, or a
  stale race when `borg_kills_nxt` was reset without wiping the array).
* `borg_follow_kill()`: `randint1(100) < 1` is never true (but still draws
  a random number). The "delete monsters that did not really move" test
  compares `kill->pos` with `oy`/`ox` before the position is changed, so
  every path that gets past the earlier checks deletes the monster; the
  code that moves it to the guessed grid is dead.
* `observe_kill_move()`: for a unique, `d = d * 2` changes the parameter,
  so the allowed distance stays doubled (and doubles again per unique) for
  the rest of the scan.
* `borg_flow_kill()`: `hall_walls` and `skip_monster` are set up once,
  not per monster. `hall_walls` keeps counting from the Borg's own
  surroundings, and once one monster sets `skip_monster` every later
  monster in the list is skipped too.
* `borg_flow_kill_aim()`: the "next to a monster" check scans slot 0 and
  dead slots too (their stale or (0,0) positions).
* `borg_shoot_scoot_safe()`: the danger of an adjacent monster is
  `borg_danger_one_kill(..., i, ...)` with `i` the *direction* index
  (0..7), not the monster's kill index `ag->kill`.
* `borg_init_flow_kill()`: the "extract dead uniques" cheat says "skip
  non-uniques" but skips uniques, so it marks non-unique races with
  `max_num == 0` (`borg_update()` redoes the unique check on each new
  level).
* `borg_locate_kill()`: `my_strnicmp(who, "It", 2)` treats any name that
  starts with "it" (any case) as an invisible monster.
* Rust (`borg_flow_kill.rs`): milestone2/source102 requires the exact unseen
  name "It" (case-insensitive, C-string termination semantics).
* v11R ([adoption/c-v11r](adoption/c-v11r/README.md)):
  * [0102](adoption/c-v11r/README.md): `skip_monster` is cleared and
    `hall_walls` zeroed for each monster; the "fewer than four walls" test
    runs once after the nine-grid scan.
  * [0110](adoption/c-v11r/README.md): `borg_shoot_scoot_safe()` passes the
    adjacent monster's kill index (`ag->kill`), not the direction index.
  * v12R ([0118](adoption/c-v12r/README.md)): the "force the grid to floor /
    granite" step is removed from `borg_new_kill()`, so a newly seen monster
    keeps the grid it was seen on instead of one derived from the slot's
    previous race; the stale-race read goes with it. C and Rust.
  * The other findings in this section remain.

## 25. Enchant item prompt keys (item enchant)

* `borg-item-enchant.c`, found by code reading.
* `borg_pick_weapon()` decides whether to send `'/'` (to equipment) or
  `'|'` (to quiver) by looking for bows, diggers, hafted weapons,
  polearms and swords in the pack. The enchant-weapon scrolls and spell
  accept any `tval_is_weapon()` item, which includes shots, arrows and
  bolts, and the item prompt opens on the pack when any accepted item is
  there (`textui_get_item`). So with ammo in the pack (outside the
  quiver) and no other weapon there, no `'/'` (or, for a quiver target
  with nothing wielded, no `'|'`) is sent, and the equipment letter or
  quiver digit is typed into the pack menu.
* `borg_pick_armor()` tests `TV_HELM` twice; `borg_enchant_to_d()` tests
  `BI_AMMO_POWER < 3` twice in its ammo loop (both harmless).
* Rust (`borg_item_enchant.rs`): all kept.

## 26. Recovery notes and checks (recover)

* `borg-recover.c`, found by code reading.
* The munchkin "cure blindness by resting" branch calls
  `borg_note("# Resting to cure problem. (danger %d)...")` without
  `format()`, so the note keeps a literal `%d`.
* The "expensive" poison cure tries `borg_eat(TV_MUSHROOM,
  sv_mush_fast_recovery)` twice in the same `||` chain (the second try
  repeats the first; harmless).
* The rod recharge rest indexes `borg_items[borg_slot(TV_ROD, sval)]`
  without checking for -1; it relies on `borg.has[kind]` meaning the rod
  is in the pack.
* Rust (`borg_recover.rs`): all kept (a missing rod panics instead of
  reading `borg_items[-1]`).
## 27. Protection from Evil rest timer overflows (flow stairs)

* `borg-flow-stairs.c`, found by code reading.
* `borg_prep_leave_level_spells()` sets `borg.no_rest_prep =
  borg.trait[BI_CLEVEL] * 1000` after casting Protection from Evil.
  `no_rest_prep` is an `int16_t`, so from clevel 33 on the value wraps
  (33 -> -32536, 50 -> -15536). The timer is then below 1, so the
  "don't rest while prepared" checks (`borg_update()`, borg-flow-misc.c,
  borg-think-dungeon.c) treat it as expired at once. borg-fight-defend.c
  and borg-fight-perm.c have the same assignment.
* `borg_flow_cost_stair()` does not check `b_stair` against
  `track_less.num` (it reads any allocated entry).
* Rust (`borg_flow_stairs.rs`): the `b_stair` check remains as before.
* v11R ([0105](adoption/c-v11r/README.md)): `no_rest_prep` is an `int` in C
  and `i32` in Rust, so the timer no longer wraps at character level 33; the
  three assignment sites follow the same type.

## 28. Perma-spell simulation and costs (fight perm)

* `borg-fight-perm.c`, found by code reading.
* `borg_perma_aux_regen()` has no `if (borg_simulate) return ...;` test,
  so the scoring loop of `borg_perma_spell()` really casts Rapid
  Regeneration (keys queued, `no_rest_prep = 6000`) while it is only
  simulating. If another move then scores higher, its spell is queued
  after it; if Rapid Regeneration wins (value 1), it is queued twice.
* `borg_perma_aux_venom()` checks the cost of `SMITE_EVIL`, not `VENOM`
  (for a class without Smite Evil the cost is -1, so it always passes).
* `borg_perma_aux_fastcast()` scores 5 when simulating but returns 1 when
  cast (harmless: the real call's value is not used).
* `borg_perma_aux_bless()` and `borg_perma_aux_resist()` ignore whether
  the cast worked and always set `no_rest_prep`; `..._resist_colluin()`
  always returns 2, even when no activation was sent.
* Speed and Protection from Evil set the `int16_t` `borg.no_rest_prep`
  to `CLEVEL * 1000`, which wraps negative from level 33 on (33000 ->
  -32536), so `no_rest_prep` has no effect then (the same pattern is in
  borg-fight-defend.c and borg-flow-stairs.c).
* Rust (`borg_fight_perm.rs`): milestone2/source110 makes regeneration
  simulation return without casting. Other findings remain, including
  `as i16` truncation for the level timer.
## 29. Sea of Runes scan slips (flow glyph)

* `borg-flow-glyph.c`, found by code reading.
* The "Skip every non floor/glyph" test is `ag->feat != FEAT_FLOOR &&
  ag->glyph`: it skips only glyphed non-floor grids, so every other grid
  (walls, doors, rubble, ...) is scored too. With an old center, such a
  grid in the 5x5 ring is a target if its flow cost is below 75.
* In the "old center" branch `tmp_glyph` is never counted, so the score
  is always `500 - cost`.
* On arrival `borg_needs_new_sea` is cleared (and the center stored) even
  if no glyph can be made. When the cast/read/activation works but the
  grid is already in `track_glyph`, the function returns false although
  the keys are queued.
* The "is the sea complete" check reads `borg_grids[center + ddd]`
  without a bounds check (a center on row/column 1 would read index -1).
* `track_glyph.num` can go below 0: borg-messages.c's "The object resists
  the spell" / "There is no clear floor" handler does `num--` without a
  check (after writing `x[num]`, `y[num]`); the next glyph would then be
  written at index -1.
* Rust (`borg_flow_glyph.rs`): all kept; an index of -1 panics instead
  of reading or writing outside the arrays.
* Milestone3 ([Rust#8](https://github.com/diegoami/borg-rs/issues/8)): the failed-glyph handler now decrements `num` only
  when it is positive and clears the tracked entry (source118), so it can
  no longer go below 0 from that path.

## 30. Munchkin attacks leave the attack flags set (attack munchkin)

* `borg-attack-munchkin.c`, found by code reading.
* `borg_munchkin_mage()` sets `borg_attacking = true` before its monster
  scan, but the two `return false` inside the scan (a monster adjacent,
  or within 2 grids and faster than the Borg; `borg.mon.scary`) do not
  clear it. `borg_munchkin_melee()` has the same leak on its
  `borg.mon.scary` return. While the flag stays set, `borg_danger()`
  ignores Protection from Evil (`pfe && !borg_attacking`), until the
  next `borg_attack()` (borg-fight-attack.c) clears it.
* Both functions set `borg_simulate = true` and, when no attack scores
  above 0, return without setting it back to false.
* `borg_munchkin_mage()` has no class check: despite the name and the
  comment ("use MM"), every class tries every attack type except
  `BF_REST` and `BF_THRUST` (ties go to the later type, `dam >= b_dam`).
* Rust (`borg_attack_munchkin.rs`): all kept.

## 31. Exploration flow quirks (flow dark)

* `borg-flow-dark.c`, found by code reading.
* `borg_flow_dark()`: the "nearest up stairs" scan skips a stair when
  `b_j >= j`, so it keeps the *farthest* up staircase (the first of
  equally far ones), as in borg-flow-take.c (section 19). Every method's
  "too far from the stairs" test uses that one.
* `borg_flow_dark_interesting()`: the "Explore visible traps" branch is
  dead. It is only reached for features from `FEAT_SECRET` up (closed
  doors return earlier), and in 4.2 only the open floor has the `TRAP`
  terrain flag, so `feat_is_trap_holding()` is always false there (the
  Borg keeps known traps in `ag->trap`, not in the feature). Visible traps
  are never "interesting" for exploration.
* `borg_flow_dark_4()`: "Clear the edges" sets `borg_data_know` and
  `borg_data_icky` of the whole border rectangle to false, so danger marks
  set on those grids before the call (they persist until the next
  `borg_danger_wipe`) are forgotten as well.
* `borg_flow_dark_5()`: the `clevel <= 5` and `clevel <= 30` spread
  branches are identical (short leash).
* Rust (`borg_flow_dark.rs`): all kept (the two identical branches are one
  test, `(a && b) || (c && b)`, without side effects).
## 32. Flow core slips (flow)

* `borg-flow.c`, found by code reading.
* `borg_flow_spread()` skips a shop entrance that is not the goal shop
  only when it is in neither the Borg's row nor its column
  (`y != borg.c.y && x != borg.c.x`); an `||` (any grid but the Borg's)
  was likely meant, so the flow may still run through other shops'
  entrances in line with the Borg.
* `borg_play_step()` (breeder levels) skips doors when
  `track_door.num >= 255`, but `track_door` has only 100 entries, so the
  test is never true: doors the Borg keeps closing are never skipped
  (once the list is full, new doors are just not noted). When several
  open doors are adjacent, the last one scanned is closed.
* `borg_play_step()` looks up the chest under the goal grid
  (`loc(x2, y2)`), not under the grid stepped on; the two are the same
  for the only caller (`borg_flow_old()` passes an adjacent grid).
* `borg_flow_old()` reads `borg_data_flow` at the neighbour before its
  "avoid screen edges" test (a Borg on row/column 0 would read index -1).
* `borg_play_step()` ends with a dead `ui_event` "unhook" (a local that is
  never set); `borg_free_flow()` frees `borg_data_flow` twice (the second
  test sees NULL).
* `borg_flow_enqueue_grid()` computes the danger of every new grid even
  when desperate/lunal/munchkin/digging, then ignores it (no effect but
  cost).
* Rust (`borg_flow.rs`): all kept (the index -1 read panics; the dead
  event code and the double free are left out, as they have no effect).
* Milestone8 ([Rust#19](https://github.com/diegoami/borg-rs/issues/19)): source125 skips a non-goal shop entrance on any grid but
  the Borg's (`||`, patch 0080), in C and Rust.

## 33. Selling and stashing slips (store sell)

* `borg-store-sell.c`, found by code reading.
* `borg_think_home_sell_useful()`: the two `memset`s that "clear out our
  initial best/test objects" pass
  `sizeof(z_info->store_inven_max * sizeof(uint8_t))`, the size of a
  `size_t` (8 bytes), not `store_inven_max` bytes. No effect: both arrays
  are rewritten before they are read.
* `borg_think_home_sell_best()`: for a staff or wand, each trial takes
  `charge_each` charges off the pack item, but the restore after the
  trial only puts the quantity back (`item->iqty++`). The pack item loses
  charges with every trial of the scan, so later trials (and
  `borg_think_home_sell_bad()`) see fewer charges. The caller restores all
  items from `safe_items` afterwards.
* `borg_good_sell()`: the "ego item not fully identified" test asks
  `borg_ego_has_random_power()` about the ego of the cloak slot
  (`borg_items[INVEN_OUTER].ego_idx`), not about the item being sold.
* The sale list: `borg_think_shop_sell()` sets `sold_item_num` to the
  index just written (not a count), and every reader (here and in
  borg-store-buy.c) scans `p < sold_item_num`, so the most recent sale is
  never seen. `sold_item_nxt` wraps at 9, so slot 9 is never used.
  borg-store-buy.c does the same with its `bought_item_*` list.
* `borg_count_sell()`: an unidentified item that is not part of a pile
  makes the function `return false` (0) at once. That return skips the
  final `borg_notice(true)` ("reset the results"), so the traits stay as
  computed for the last "what if" (the pack without an earlier sellable
  item).
* Rust (`borg_store_sell.rs`): all kept (`test_item`/`best_item` clear
  their first 8 bytes; a shorter array would panic where C overruns).
* Milestone6 ([Rust#15](https://github.com/diegoami/borg-rs/issues/15)): source134 skips (does not return on) an unidentified
  single item in `borg_count_sell()`.
* Milestone9 ([Rust#21](https://github.com/diegoami/borg-rs/issues/21)): source20 makes `sold_item_num` count the newest sale (patch
  0094), in C and Rust.
* v11R ([adoption/c-v11r](adoption/c-v11r/README.md)):
  * [0116](adoption/c-v11r/README.md): after each home-sell trial the pack
    item's staff/wand charges are restored with its quantity, in C and Rust.
  * [0117](adoption/c-v11r/README.md): `borg_good_sell()` checks the sold
    item's own ego for an unknown random power, not the worn cloak's.
  * The other findings in this section remain.
## 34. Shopping slips (store buy)

* `borg-store-buy.c`, found by code reading.
* `borg_good_buy()`: the Black Market Rod of Teleport Other test reads
  `class != CLASS_MAGE && class == CLASS_ROGUE` (comment: "mage and rogue
  can cast teleport away"), so only rogues may buy it there; `!=` was
  meant for both.
* `borg_think_shop_buy_useful()`: the "immediate shopping" test (no food)
  is `tval != TV_FOOD && (tval != TV_SCROLL && sval !=
  sv_scroll_satisfy_hunger)`: every scroll, and every item whose sval
  happens to equal Satisfy Hunger's, gets through; `!(tval == TV_SCROLL &&
  sval == ...)` was meant.
* `borg_think_home_buy_useful()`: the two "require two empty slots"
  `continue`s come after the ware was taken out of the home
  (`iqty -= qty`, and `pval` for staffs/wands), so that home ware keeps the
  reduced count until the next store read. `p_left`/`p_right` are not
  reset per ware: when a ring slot holds the One Ring, the other hand's
  value comes from an earlier ring. The left-hand trial uses one ring, the
  right-hand trial `qty` rings.
* `borg_think_home_grab_useless()`: `skip_it` is never reset, so after the
  first ware that was just sold to the home, every later ware is skipped.
* `borg_think_home_buy_swap_weapon()` / `_armour()`: when there is no
  hole, they return without restoring `weapon_swap_value` /
  `armour_swap_value` (left at -1). When a swap exists, the trial slot is
  the swap's own pack slot: each candidate is copied over it and its
  `iqty` set to 0 afterwards, and the slot is not restored from
  `safe_items`, so the swap item is gone from `borg_items` until the next
  inventory read (the final `borg_notice(true)` already sees no swap).
* `bought_item_num` is the index of the last entry written, but the
  readers (`borg_store_sell`) loop `p < bought_item_num`, so the latest
  purchase is never checked; `bought_item_nxt` wraps at 9, so entry 9 is
  never used. (`sold_item_*` in borg-store-sell.c work the same way and are
  read the same way here.)
* Rust (`borg_store_buy.rs`): all kept.
* Milestone9 ([Rust#21](https://github.com/diegoami/borg-rs/issues/21)): source17 lets every class but Mage and Rogue buy the Rod of
  Teleport Other (0096); source18 lets a Borg with no food shop at the
  Alchemist and fixes the Remove Hunger test (0090); source19 restores the
  home ware when a home-buy trial is skipped (0093); source20 makes
  `bought_item_num` count the newest purchase (0094), in C and Rust.
* v11R ([adoption/c-v11r](adoption/c-v11r/README.md)):
  * [0115](adoption/c-v11r/README.md): `borg_think_home_grab_useless()`
    resets `skip_it` for every home ware.
  * [0114](adoption/c-v11r/README.md): the home-swap trial slot is restored
    from `safe_items` after each candidate, not just its `iqty` cleared.
  * The Black Market Rod and `bought_item_num` findings above remain.

## 35. Escape slips (escape)

* `borg-escape.c`, found by code reading.
* `borg_escape()`, danger level 1: before the "critical" attempts (1.1 to
  1.3) the Borg pretends to have full mana
  (`borg.trait[BI_CURSP] = borg.trait[BI_MAXSP]`). The real value is put
  back only when all three fail. When one of them succeeds, the function
  returns with `BI_CURSP` still at `BI_MAXSP` until the traits are next
  recomputed.
* Danger levels 3.1, 4.1, 5.1, 6.1 and 8 test
  `(borg_escape_stair() || borg_caution_phase(..)) && ... && (phase spell
  || scroll ...)`. On an up staircase `borg_escape_stair()` already queues
  `<` (and its note) and skips the landing-zone check. If no phase is then
  possible (or the anti-summon timer is <= 50), the `<` stays queued and
  the function goes on to the next tests; if a phase is possible, both the
  `<` and the phase keys are queued and the phase bookkeeping
  (`borg.escapes--`, "Danger Level 3.1" etc.) is applied.
* The anti-summon reset (`if (borg_timer(borg.time.antisummon) < 50)
  borg.time.antisummon = 0;`) after 2.2, 3.1, 3.3, 4.1, 4.3, 6.1 and the
  first level 8 test is dead: those branches require the timer to be > 50.
* `borg_dimension_door()` targets the least dangerous grid of the 100x100
  box (offsets -50..49) that is in bounds, without checking that it is a
  known floor grid without a monster, or in range of the spell; it ignores
  the result of `borg_spell()`.
* Rust (`borg_escape.rs`): the remaining findings are kept.
* Milestone8 ([Rust#19](https://github.com/diegoami/borg-rs/issues/19)): source21 aims Dimension Door only at a known floor grid
  without a monster (patch 0086), in C and Rust.
* v11R ([0100](adoption/c-v11r/README.md)): danger levels 3.1, 4.1, 5.1,
  6.1 and 8 treat the stair escape as a complete alternative. Either the
  Borg climbs, or it phases when the landing zone and timers allow; it no
  longer queues `<` and then phase keys behind it, in C and Rust.
## 36. Shop choice slips (think store)

* `borg-think-store.c`, found by code reading.
* `borg_choose_shop()`: "If poisoned or bleeding -- flow to temple"
  (`goal.shop = 3`) and "flow to general store" (`goal.shop = 0`) have no
  effect on the choice: none of the buy/sell helpers reads `goal.shop`, and
  unless one of them sets a ware the next step resets
  `goal.shop = goal.ware = goal.item = -1`. They matter only if
  `goal.ware != -1` came in with `goal.shop == -1`: then
  `borg_think_shop_buy_useful()` returns false at once ("already have a
  target") and the second "already flowing" test returns true with the
  forced shop and the stale ware.
* The money-scumming test repeats `!borg.trait[BI_CDEPTH]`, already
  returned on at the top of the function.
* Rust (`borg_think_store.rs`): all kept.
* Milestone8 ([Rust#19](https://github.com/diegoami/borg-rs/issues/19)): source23 sends a poisoned or bleeding Borg to the Alchemy
  shop (4) in `borg_choose_shop()` and `borg_think_shop_buy_useful()` (patch
  0087), in C and Rust.

## 37. Junk and drop choices (junk)

* `borg-junk.c`, found by code reading.
* `borg_drop_junk()`: the "# Dropping junk %s (valued at %ld)" note prints
  `value`, which still holds the value of the *last* item scanned, not
  the value of the item being dropped.
* `borg_drop_hole()`: the "BIG HACK! don't crush cool stuff" tests for
  rods, wands, scrolls and potions (Borg above clevel 35) chain `!=` with
  `||`, so they hold for every sval (the svals compared are distinct).
  Every aware rod, wand and potion is valued at `iqty * 300000`, every
  aware scroll at `iqty * 30000`, and the `value / 2` / `value / 10` else
  branches are dead. The staff test is false only for Dispel Evil when
  `BI_GOOD_S_CHG >= 2` and `BI_ASTFDEST >= 2`.
* `borg_drop_hole()`: the second "Skip artifacts" test (`art_idx &&
  !ident`) is dead (all artifacts were skipped earlier). In the
  desperate pass `value_boost = 7000` overwrites the 10000 (boots, weapon
  while digging) and 5000 (unknown runes) boosts set just before it.
* `borg_drop_slow()`: the scan skips every equipment slot
  (`pack_size <= i <= INVEN_FEET`), so the `/` + equipment letter branches
  of the inscribe and drop keys are dead.
* `borg_remove_stuff()`: `b_p` stays `borg.power`, so the *last*
  equipment slot whose removal does not lower the power is taken off, not
  the best one, and "Power with" always prints `borg.power`. The hole
  slot is restored with `iqty = 0` only: it keeps a copy of the last
  tested item during the final `borg_notice()`.
* `borg_consume()`: Brawn (`sv_potion_inc_str2`) has no `break` after a
  failed quaff and falls into the other potion tests (harmless).
* Rust (`borg_junk.rs`): all kept.
* Milestone7 ([Rust#17](https://github.com/diegoami/borg-rs/issues/17)): source136 uses `==` in the five drop/hold device and potion
  comparisons (patch 0074), in C and Rust.

## 38. borg.txt parsing slips (init)

* `borg-init.c`, found by code reading; checked with the C parser on
  4,000 random borg.txt files (see PROGRESS.md, module 45).
* `borg_proc_setting()`: for an int setting, a byte after the name that is
  not ` `, `=`, `-` or a digit (e.g. `borg_stop_dlevel = x`, or an unknown
  key that starts with an int setting's name, `borg_stop_dlevelx 5`) is
  never consumed: the loop spins forever and the game hangs at start-up
  (seen with the C parser at -O2).
* `borg_proc_setting()`: a `-` with no digits after it still negates the
  setting, so `borg_no_deeper -` gives -127 (the default, negated). The
  value is `atoi()`, so a number past `INT_MAX` is the low 32 bits of the
  `long` (`borg_enchant_limit 99999999999` is 1215752191, then 15).
* `borg_proc_setting()`: names are prefix-matched, so `borg_verbosex = TRUE`
  sets `borg_verbose`; leading spaces are skipped for settings but not for
  the `REQ`/`FORMULA`/`CND`/`POWER`/`[BEGIN FORMULA SECTION]` tests.
* `borg_init_txt_file()`: if borg.txt is missing and cannot be written, it
  returns before `borg_cfg` is allocated; `borg_init()` then reads the NULL
  `borg_cfg` and crashes. If the written file cannot be reopened,
  `file_getl()` reads through a NULL file.
* `borg_reset_ignore()` frees `borg.init_save.ego_ignore_types[i]` for every
  ego without checking the array for NULL (only reached without an earlier
  `borg_reinit_options()`, which no command path does).
* Rust (`borg_init.rs`): all kept (the hang is the same loop; the NULL
  reads are panics).
* Milestone6 ([Rust#15](https://github.com/diegoami/borg-rs/issues/15)): source141 ends the parse at a non-value character (keeps
  the default; no hang), source142 applies `-` only to a number that follows,
  and source144 sets the defaults before reading borg.txt and warns if the
  rewritten file cannot be read (C and Rust).

## 39. Misc flow slips (flow misc)

* `borg-flow-misc.c`, found by code reading and the cross-check.
* `borg_flow_spastic()`: the reverse flow is spread with `optimize` set,
  and the origin is the Borg's own grid (cost 0), so `borg_flow_spread()`
  stops at its first dequeue: every other grid keeps cost 255 and is
  skipped as unreachable. Only the Borg's own grid can be picked; the
  commit succeeds (cost 0) but `borg_flow_old()` finds no step, cancels
  the goal and returns false. The next call takes the "We have arrived"
  branch (note, `xtra += 5` on the 9 grids) and returns false as well. The
  function never moves the Borg and never returns true.
* `borg_flow_spastic()`: the value `v` subtracts `ag->xtra * 40`, but
  `ag` was reused by the neighbour loops and points at the last one
  (`ag_ptr[7]`, the north-west grid), not at the candidate grid.
* `borg_flow_vault()`: the flow is not cleared before the grids are
  enqueued, so the queue and costs left by the previous flow are spread
  further; each excavatable grid is listed once per adjacent permanent
  wall (duplicate entries; up to 8 per grid, so `borg_temp_*` could in
  theory overflow).
* `borg_check_rest(y, x)`: the vault test loops `-1 <= i, ii < 1`, so it
  looks at the 2x2 grids up and left of the Borg (not the 3x3 around
  him), and always around the Borg's position, not around (y, x). The
  function clears `borg.time.last_kill_mult` once the early tests pass, so
  `borg_flow_recover()`'s scan resets it on its first candidate. The
  monster danger is `borg_danger_one_kill()` at the monster's own grid,
  not at (y, x).
* `borg_twitchy()`: the 20th random pick decrements `count` to 0 before
  its tests, so even a good 20th pick runs the "surrounded by walls"
  scan, which may choose another direction; `borg.goal.g` is left at the
  last random pick.
* `borg_flow_vein()`: `borg_flow_clear()` runs once per listed vein inside
  the scan loop (no effect beyond the first, except the work).
* Rust (`borg_flow_misc.rs`): all kept. Where C would read or write out of
  bounds (a `borg_temp_*` overflow, a shop index past `store_max`), the
  Rust code panics.
* Milestone7 ([Rust#17](https://github.com/diegoami/borg-rs/issues/17)): source146 runs the spastic reverse flow with `optimize` false
  (patch 0064), in C and Rust.
* Milestone8 ([Rust#19](https://github.com/diegoami/borg-rs/issues/19)): source149 no longer clears `last_kill_mult` in
  `borg_check_rest()` (patch 0079), and source147 uses the candidate grid's
  own search count in the spastic value (patch 0089), in C and Rust.
* Milestone9 ([Rust#21](https://github.com/diegoami/borg-rs/issues/21)): source348 stops the `borg_flow_light()`/`borg_flow_vault()`
  panel scans at the map edge (patch 0097), in C and Rust.

## 40. Caution slips (caution)

* `borg-caution.c`, found by code reading; all checked against the c2rust
  code (see PROGRESS.md, module 47).
* `borg_caution()`: when the Borg stands on a tracked up or down staircase
  it notes "# Standing on up-stairs" / "dn-stairs" and sets `on_upstair` /
  `on_dnstair` to `false` (they start `false`), so both are always false.
  The "danger and standing on stair" clause of the take-the-stairs test is
  dead, and the stair keys only follow the grid's own feature
  (`FEAT_LESS` / `FEAT_MORE`).
* "Desperation Head for stairs": the scans say "How far is the nearest up
  stairs" but `if (b_j >= j) continue;` keeps the *farthest* stair without a
  monster on it, so `b_j < 3` / `b_j <= 2` / `b_j < 20` (and `b_m <= 2`)
  only hold when every free staircase of that kind is that close.
* Strategic retreat: "Skip this grid if it is adjacent to a monster" only
  `break`s out of the monster scan, so the test has no effect.
* Back away: `adjacent_monster` is set inside the grid loop and never reset,
  so after the first grid next to a (fast) monster every later grid is
  skipped too; the `!adjacent_monster` term of the `MAXCLEVEL < 35` test is
  then always true. "Mega-Hack -- skip traps" is a `break`: a known trap
  without a glyph on one neighbour ends the whole scan.
* "Excessive danger": the Borg only starts fleeing when
  `borg.ready_morgoth == 1` (the comment says "do not flee level if going
  after Morgoth"); with `ready_morgoth` 0 or -1 it never flees here.
* Waiting for recall in danger, step (1): `borg.trait[BI_CURSP] <
  (borg.trait[BI_CURSP] / 4)` compares the mana with itself (true only for
  negative mana); `BI_MAXSP` was meant.
* `borg_heal()`: the two critical blocks (poisoned, bleeding) set
  `BI_CURSP = BI_MAXSP` and put the real value back only after their spells
  fail. When a spell is cast, or (bleeding block, where the fake mana is set
  before the potions) a cure or restore-mana potion is quaffed first, the
  function returns with `BI_CURSP` still at `BI_MAXSP` until the traits are
  next recomputed (as C bug 35).
* `borg_heal()`, "Heal step two": the chain zaps the healing rod twice, the
  second time without the "keep the rod if I can escape" test, so that test
  has no effect there. `cmw_heal` is computed and never used.
* Rust (`borg_caution.rs`): all kept. `borg_heal()` divides by `BI_MAXHP`
  and the resistance note by `borg_game_ratio`; a 0 there panics where C
  traps.
* Milestone6 ([Rust#15](https://github.com/diegoami/borg-rs/issues/15)): source156 flees on excessive danger unless ready for
  Morgoth; source157 compares the recall-wait mana with `BI_MAXSP / 4`;
  source153 takes the nearest free stairs; source155 skips trapped grids and
  resets `adjacent_monster` per grid in the back-away scan.
* Milestone9 ([Rust#21](https://github.com/diegoami/borg-rs/issues/21)): source364 gives the three healing-chance adjustments the
  direction their comments state (patch 0092), in C and Rust.

## 41. Debug command slips (commands)

* `borg-commands.c` (the `^z` commands; the suites only use `z`), found by
  code reading and the fuzz cross-check.
* `borg_cmd_steps()` (`y`): the loop shows `track_step.{x,y}[num - i]` for
  `i = 0 .. num - 1`, i.e. entries `num` down to 1: one past the newest
  step (a stale or zero entry) and never entry 0. With a full list
  (`num == size`, 100) C reads one `int` past each array; Rust panics.
* `borg_cmd_targeting()` (`t`): when the "Show Arc" prompt is cancelled the
  function returns without restoring `player->opts.delay_factor`, which
  stays 200.
* `borg_cmd_power()` (`p`): `msg("... %ld", p)` with an `int32_t` (Borg and
  home power). gcc passes the value zero-extended (`mov %eax,%esi`, checked
  in the C build) and `vstrnfmt` reads a `long`, so a negative power prints
  as its unsigned 32-bit value.
* `borg_cmd_flow()` (`^`): the start cost `b_c` is read at the Borg's real
  grid on every step, not at the simulated one. Neighbours of a simulated
  grid on the map edge are read at `y` or `x` = -1 (C reads outside the
  array; Rust panics).
* `borg_cmd_grid_target_feature()` (`G`): the "game Feat" is the feature
  under the Borg (`square(cave, borg.c)`), not at the targeted grid.
* `borg_cmd_flags()` (`f`) and `borg_cmd_grid_target_info()` (`I`) pass
  `format(...)` output as the format of `msg()`. The texts never contain a
  `%` (setting names, numbers), so this has no effect.
* `borg_cmd_flags()` and `borg_cmd_grid_feature()` never free their menus
  (`menu_new` without `menu_free`; kept, the Rust `engine::Menu` is never
  freed either).
* `borg_cmd_object_desc()` (`o`): "Which item?" accepts `pack_size`, the
  always-empty end of `player->upkeep->inven`; the display then shows
  `borg_items[pack_size]` (the weapon slot) with all-zero flags.
* `do_cmd_borg()`: an escaped command prompt calls `Term_load()` without a
  matching `Term_save()`, so it pops a screen saved by someone else (or
  nothing). `borg_cmd_help()` saves once and loads twice when the nested
  prompt is escaped, and `borg_cmd_target()` (`G`, `I`, `D`) starts with a
  `Term_load()` whether or not it was reached from the help.
* Rust (`borg_commands.rs`): all kept; the out-of-bounds reads are panics.

## 42. Defence slips (fight defend)

* `borg-fight-defend.c`, found by code reading (the banishment slip is #1).
* `borg_defend()` sets `borg_simulate = true` and returns with it still
  set when it refreshes Resistance or finds nothing worth doing; only a
  performed defence clears it.
* `borg_defend_aux_resist_e()` notes "# Attempting to cast RAcid".
* `borg_defend_aux_grim_purpose()` keeps `BI_RCONF`/`BI_FRACT` in `bool`s
  and writes them back as 0/1, not the saved values.
* `borg_defend_aux_tele_away()`: the third fail-rate branch
  (`p1 > avoidance * 5 / 2`) is dead (`p1 > avoidance * 2` comes first).
  `borg_defend_aux_tele_away_morgoth()` resets `borg_tp_other_n` before
  its debug loop, and `borg_launch_bolt()` already resets it before it
  returns, so neither function ever writes its "# T.O. %d, index %d" /
  "# %d, index %d" notes.
* `borg_defend_aux_earthquake()`, real call: sets `borg_needs_new_sea`
  before trying, and a cast Tremor (keys `*5` queued) falls through to the
  checks and returns 0 instead of a value. `borg_defend_aux_destruction()`
  returns 500 whether or not anything was cast;
  `borg_defend_aux_teleportlevel()` falls through to the landing-zone
  simulations (`borg_caution_teleport`) when the spell fails.
* `borg_defend_aux_banishment()`: the notes give the danger of
  `ag->kill` (the kill index on the monster's grid, not always `i`); on a
  real call the booted monster is deleted before `p2 -=` reads its danger,
  so the real call returns 0 even when it banished monsters.
* `borg_defend_aux_genocide()`: the per-character arrays have `UCHAR_MAX`
  (255) entries but are indexed by `(unsigned char)d_char`, so a race
  character 255 would write one past the end (Rust panics; no race uses
  it); `tmp_genocide_target` is a signed `char`. The genocide removal
  loops (also `..._genocide_nasties()`) read `r_info[0]` for dead slots.
* `borg_defend_aux_light_morgoth()` reads `best` uninitialised if Morgoth
  is not found on the real call (not reachable: the real call follows a
  simulation of the same state that found him). Rust starts it at (0, 0).
* Rust (`borg_fight_defend.rs`): all kept.
* Tracker: one lead per slip, in the order above:
  diegoami/neo-angband-tracker#173 to diegoami/neo-angband-tracker#183.
* Milestone9 ([Rust#21](https://github.com/diegoami/borg-rs/issues/21)): source549 lets the Teleport Other defence zap a Rod of
  Teleport Other (patch 0091), in C and Rust.

## 43. Power slips (power)

* `borg-power.c`, found by code reading; all checked against the c2rust
  code (see PROGRESS.md, module 51).
* `borg_power_inventory()`, "collect Resistance pots if not immune -- All
  Classes": `k` is not reset before the loop, so it starts where the
  previous loop left it (the low-calorie food count, capped at 15, or the
  Cure Serious count of the cut/poison block, capped at 5). With 4 or more
  of those, Resist Poison potions get no reward here.
* Same function, "Reward Resistance Potions for Warriors": `k` is reset
  before the heat and cold loops but not before the poison loop, which
  starts at the Resist Cold count (capped at 4).
* Same function, the encumbrance test scans for the "last" item from
  `PACK_SLOTS` down, one slot past the last usable pack slot (other Borg
  code starts at `PACK_SLOTS - 1`). With an empty quiver that slot is
  `INVEN_WIELD`, so when a weapon is wielded it is taken as the "test item"
  and the exemption for enchant scrolls and stat potions just bought never
  applies.
* Same function: the penalty divides by `borg.trait[BI_CARRY] / 10`
  unchecked; a carry limit below 10 traps in C and panics in Rust (not
  reachable in play).
* `borg_power_equipment()`: `adj_str_hold[borg.trait[BI_STR_INDEX]]` is not
  bounds checked (38 entries); an index above 37 reads outside the table in
  C and panics in Rust (`borg_notice` caps the index at 37). The books loop
  of `borg_power_inventory()` reads `player->class->magic.books[book]` for
  every book the Borg counts; a book the class does not have would read
  past the array in C and panics in Rust (`borg.amt_book` only counts the
  class's own books).
* `borg_power_equipment()`, the speed rewards with `borg_worships_speed`:
  the 140-144 tier repeats the 135-139 base (175000), so going from +19
  to +20 speed gains no step bonus, unlike every other tier boundary.
  Kept.
* Rust (`borg_power.rs`): all kept; the out-of-bounds reads and the
  division by zero are panics.
* Milestone6 ([Rust#15](https://github.com/diegoami/borg-rs/issues/15)): source184/185 reset `k` before both Resist Poison loops.
* Tracker: one lead per slip, in the order above (the out-of-bounds bullet
  is two):
  diegoami/neo-angband-tracker#184 to diegoami/neo-angband-tracker#190.

## 44. Message parsing slips (messages)

* `borg-messages.c`, found by code reading and the cross-check.
* `borg_parse_aux()`: "You are too afraid to attack ..." is tested twice;
  the first test reacts `AFRAID:` and returns, so the second (`MISS:`) is
  dead.
* "You have removed the ...": the "remove rubble around the Borg" loop
  runs over 2x2 grids (`y` from `c.y - 1` to `c.y`, `x` likewise, not
  3x3) and only ever looks at the goal grid `ag`, never at `(x, y)`: it
  marks the goal grid as a broken door if it is rubble (and the Borg has no
  light).
* The "monster spell moved the Borg" test (`prefix(msg, "commands you to
  return.")`, "teleports you away.", "gestures at your feet.") uses
  `prefix` on texts that always start with the monster's name ("Something
  commands you to return."), so it never matches and Lunal mode is never
  switched off here.
* "Identifying The Phial" has no `return`: the remaining tests run on the
  message too (none matches it).
* The failed-glyph handler ("The object resists the spell", "There is no
  clear floor") writes `track_glyph.x[num]`/`y[num]` with no check: with a
  full list (`num == size`, 200) C writes one `int` past each array, and
  `num` can go below 0 (C bug 29). Rust panics on both.
* `who` buffers: for "hit by" and spell patterns the monster name is
  `strnfmt(who, start - msg, "%s", msg)`. When the pattern is at the start
  of the message (spell texts without a `{name}`, e.g. "You feel something
  roll beneath your feet."), `max` is 0, `vstrnfmt`'s `max - 1` wraps and
  the whole message is copied: the "name" is the message itself, and a
  message of 256 bytes or more would overflow the 256-byte `who` (Rust
  panics). Likewise every `who` text is limited only by the message length
  (`1 + len - tmp`), not by `sizeof(who)`; fragments that long do not occur
  (the message line is 80 columns and long messages are split at sentence
  ends).
* `borg_get_parsed_pain()` stops at `pos < strlen(pain)` (the buffer is
  `strlen + 1` bytes and starts with the added space), so a message without
  `[|]` loses its last character: " can move again", " falls under your
  spell", " is no longer under your control", " shimmers for a moment",
  " There is a puff of smoke" (no final '.' or '!'). Messages end in
  punctuation, so these five pain suffixes never match.
* "It hits you" (with known glyphs) and "The rune of protection is broken!"
  call `square_iswarded(cave, borg.c)` without a bounds check. Messages
  are parsed before `borg_update()` moves `borg.c` to the new level, so on
  the first key after a level change `borg.c` can still hold the old
  level's position, outside a smaller level (seen in the fuzz check: the
  town, 66x22, with `borg.c` = (113, 48)); C then reads outside the
  level's row array. `engine::square_iswarded` panics.
* Rust (`borg_messages.rs`): all kept; the out-of-bounds reads and writes
  are panics.
* Milestone3 ([Rust#8](https://github.com/diegoami/borg-rs/issues/8)): source118 guards the failed-glyph handler (decrement
  first, only when `num > 0`) in C and Rust.
* Tracker: one lead per slip, in the order above:
  diegoami/neo-angband-tracker#191 to diegoami/neo-angband-tracker#198.
* Milestone7 ([Rust#17](https://github.com/diegoami/borg-rs/issues/17)): source198 checks `square_in_bounds()` before both
  `square_iswarded(cave, borg.c)` calls (patch 0070), and source197 sizes the
  pain buffer for the leading space and the terminator, so the last
  character is no longer cut (patch 0071), in C and Rust.


## 45. Swap slips (trait swap)

* `borg-trait-swap.c`, found by code reading.
* `borg_notice_weapon_swap()`, final block for the chosen weapon: sets
  `weapon_swap_resist_acid` from the *electricity* resistance
  (`el_info[ELEM_ELEC]`), so a swap weapon with only acid resistance
  does not count as one for `BI_SRACID` (the valuation loop reads acid
  correctly).
* `borg_notice_weapon_swap()`: the first-half troll reward adds
  `(dam * 2) / 2` instead of `(dam * weapon_swap_slay_troll) / 2`.
* `weapon_swap_resist_fear` and `armour_swap_resist_fear` are never
  written, so they stay false: the weapon's +5500 fear reward never
  applies and `BI_SRFEAR` never comes from a swap.
* `borg_notice_weapon_swap()`: with swaps off (`borg_uses_swaps()`
  false), the first wearable non-digger at max depth 50+ makes the
  function `return` in the middle of the scan: the swap flags keep the
  values of the previous item looked at (or of the previous call), and
  `enchant_weapon_swap_to_h/_d` keep the previous call's values (only the
  normal end clears them).
* `weapon_swap_digger` is cleared per item but not after the scan, so it
  keeps the digging power of the last weapon looked at.
* `borg_notice_armour_swap()`: gloves are wearable but not in the
  switch, so they are never a swap armour.
* Rust (`borg_trait_swap.rs`): all kept.
* Milestone5 ([Rust#13](https://github.com/diegoami/borg-rs/issues/13)): source205 adds gloves to the armour-swap candidates.
* Tracker: one lead per slip, in the order above:
  diegoami/neo-angband-tracker#199, #201, #202, #203, #204, #205.
* Milestone7 ([Rust#17](https://github.com/diegoami/borg-rs/issues/17)): source199 reads `ELEM_ACID` for the acid resist (patch 0066)
  and source201 uses the real troll slay multiplier (patch 0073), in C and Rust.

## 46. Update slips (update)

* `borg-update.c`, found by code reading; all checked against the c2rust
  code (see PROGRESS.md, module 54).
* `borg_update_map()`: `old_wall` is taken after the grid's `feat` has
  already been overwritten with the game's feature, so `old_wall` and
  `new_wall` only differ when a later branch changes `feat` again (a shop
  entrance not in view on a grid marked `BORG_IGNORE_MAP`). Newly seen
  walls, opened or closed doors and dug tunnels are therefore never taken
  out of the current flow (`borg_data_flow` 255, `borg_data_know`/`icky`
  cleared), never trigger a view or light update, and never reset the
  wizard light and detect walls stamps. Not seen once in real play
  (smoke, full, deep, classes); the fuzz cross-check reaches it.
* Same function: the "Floors" branch tests `g.f_idx == FEAT_NONE`, the
  condition of the "Darkness" branch just before it, so it is never taken:
  floor grids are never marked `BORG_GLOW`, `BORG_DARK` or `BORG_LIGHT`
  from the game's lighting (only darkness, "call light" and the light
  code mark them).
* Same function: when the 9000-entry `borg_wanks` list is full it calls
  `borg_oops()` and then still writes the next entry past the end of the
  array (out of bounds in C; Rust panics). Not reachable with the 80x24
  map panel (at most 66 x 21 grids per update).
* `borg_update()`, the first message pass ("monsters at the goal"):
  `STATE__FEAR:` is tested twice in the same `else if` chain; the second
  test (which would note that shooting through darkness worked) is never
  reached.
* Same function, the hit-distance scan: it stops at the first unparsed
  `SPELL_` or `QUAKE:` message, so a teleport-to or teleport-away spell
  queued after another monster spell does not widen the distance.
* Same function, the second message pass ("monsters near the old
  location"): `created_traps` is set by a trap-creating spell and never
  reset inside the loop, so every later unparsed message of the same turn
  clears the trap detection of the old panel again and marks the floor
  grids around the Borg as traps again (with the notes). In the same pass
  a `BLINK:` message only counts as a successful dark shot when
  `successful_target == -1`, where every other message tests `< 0`.
* Same function, new level: the panel flags of walls, traps, doors and
  evil are cleared, but `borg_detect_obj` is not, so object detection
  done on the previous level still counts on the new one (the arrays are
  only reallocated, cleared, when the number of panels changes).
* Same function, new level, the live unique count: the fourth live unique
  is stored in `unique_depths[3]` without sorting, and a later unique only
  replaces it when it is shallower than the third. So the fourth unique is
  never counted among the three shallowest, even when it is shallower.
* `borg_fear_spell()`: after the switch, `ouch` is doubled when it is at
  least half the hit points ("Things which hurt us a lot need to be a
  concern"), but `ouch` is not read again, so the extra concern never
  applies.
* Rust (`borg_update.rs`): all kept; the out-of-bounds write is a panic,
  and so are a detection panel index outside the arrays and a queued
  `SPELL_` message without a ':' (C reads through `NULL + 1`; every
  message the Borg queues has one).
* Milestone5 ([Rust#13](https://github.com/diegoami/borg-rs/issues/13)): source213 makes the Floors branch live (`FEAT_FLOOR`, lit
  test via `square_islit()`), and source219 also clears `borg_detect_obj` on
  a new level (C and Rust).
* Milestone4 ([Rust#11](https://github.com/diegoami/borg-rs/issues/11)): source212 takes `old_wall` before the feature is
  overwritten, so wall/door changes reach the flow/view updates (C and Rust).
* Tracker: one lead per slip, in the order above (the `created_traps` /
  `BLINK:` bullet is two):
  diegoami/neo-angband-tracker#212 to diegoami/neo-angband-tracker#221.
* Milestone7 ([Rust#17](https://github.com/diegoami/borg-rs/issues/17)): source213 marks floors as glowing from the permanent-glow
  flag `square_isglow()` (patch 0069, correcting the impact v5 patch 0050),
  so a torch-lit grid is no longer marked glowing; C keeps local #295.

## 47. Dungeon-think helper slips (think dungeon util)

* `borg-think-dungeon-util.c`, found by code reading; all checked against
  the c2rust code (see PROGRESS.md, module 55).
* `borg_think_dungeon_light()` (lines 152-154): the entry test is
  `!borg.trait[BI_LIGHT] && (borg.trait[BI_LIGHT] <= 0 ||
  borg_items[INVEN_LIGHT].timeout <= 3)`. The second part is always true
  once the first holds, so a light that is about to burn out (timeout
  `<= 3`) never starts the maneuvers on its own.
* Same function, "Test for stairs" (lines 186-195): on an up staircase it
  queues `<` with the note "# Testing for stairs ." and does not return, so
  the code below may queue a rest, a light spell or a flow step behind the
  `<` in the same turn.
* Same function, the Call Light scan (lines 268-275): the open/broken door
  case requires `y == borg.c.y && x == borg.c.x`, but the scan only visits
  the 8 neighbours, so the case is never taken and the "is the door lit"
  cheat (`square_isglow`) never runs; an open door next to the Borg is
  treated like any other grid.
* `borg_think_stair_scum()` (lines 400-404): the "do I already know this up
  staircase" loop `continue`s where the down-stair loop above it `break`s,
  so it always ends at `track_less.num` and the staircase under the Borg
  is appended to `track_less` again (while there is room) every time the
  map does not show it as `FEAT_LESS`. (`borg-think-dungeon.c` has the
  same loop twice.)
* Same function (lines 451-513): `b_j`, the distance to the nearest down
  staircase, is not reset before the up-stair scan, so a close down
  staircase makes the "upstair is close" test (`b_j < 8`) pass even when
  every up staircase is far away.
* `borg_leave_level()` (lines 705-788): `prep_cur_depth` keeps the pointer
  `borg_prepared()` returned. A formatted reason ("No deeper N.", "Scumming
  *Heal* potions (N to go).", "Collect from house (N potions).", "Must kill
  X.", "X still alive!") lives in the one static `borg_prepared_buffer`,
  which the later calls `borg_prepared(depth + 1)`, `(depth + 5)` and
  `(max_depth * 5 / 10)` overwrite. The notes "heading up, not prep for
  current level", "Returning to town (too deep)", "Climbing (too deep)"
  and the "Collect from house" test then read the reason of the later
  depth, not the current one.
* Same function (line 723): "Do not dive when drained" is `if (g &&
  borg.trait[BI_ISFIXEXP])`, and `g` is only non-zero here when the Borg
  is already heading up, so a drained Borg is not kept from diving.
* `borg_excavate_vault()` (lines 1086-1089): after a cast it clears
  `BORG_GLOW` ("Not Lit") and then sets `BORG_GLOW` again under the
  comment "Dark" (surely meant `BORG_DARK`), so the excavated grid is
  remembered as glowing.
* Rust (`borg_think_dungeon_util.rs`): the remaining findings are kept. A
  `borg_feeling_stuff` outside 0..11 panics in
  `borg_time_to_stay_on_level` (C reads past `borg_stuff_feeling`), and a
  grid outside the map panics (C reads outside `borg_grids`).
* v11R ([0106](adoption/c-v11r/README.md)): `borg_leave_level()` copies the
  current depth's reason immediately, before the later `borg_prepared()`
  calls overwrite the static buffer; the notes and the "Collect from house"
  test use the copy, in C and Rust.
* Milestone6 ([Rust#15](https://github.com/diegoami/borg-rs/issues/15)): source235 keeps a drained Borg from diving whenever
  `BI_ISFIXEXP` is set (the dead `g` gate is gone).
* Milestone5 ([Rust#13](https://github.com/diegoami/borg-rs/issues/13)): source231 scans the Borg's own grid in the Call Light
  loop, and source236 marks the excavated vault grid `BORG_DARK`.
* Milestone4 ([Rust#11](https://github.com/diegoami/borg-rs/issues/11)): source229 lets a dying fuelled light (timeout <= 3,
  not `NO_FUEL`) start the maneuvers; source230 returns after queuing `<`;
  source232 `break`s the known up-stair loop (the `borg-think-dungeon.c`
  copies are unchanged).
* Tracker: one lead per slip, in the order above:
  diegoami/neo-angband-tracker#229 to diegoami/neo-angband-tracker#236.
* Milestone9 ([Rust#21](https://github.com/diegoami/borg-rs/issues/21)): source348 stops the `borg_excavate_vault()` panel scan at the
  map edge (patch 0097), in C and Rust.


## 48. Trait slips (trait)

* `borg-trait.c`, found by code reading.
* `borg_notice_equipment()`, racial resistances: `BI_RACID` is set when
  the *race* resists fire (`el_info[ELEM_FIRE]`, tested twice, once for
  `BI_RACID` and once for `BI_RFIRE`); a race's acid resistance is never
  counted.
* `borg_notice_equipment()`: an item's light modifier
  (`modifiers[OBJ_MOD_LIGHT]`) is added to `BI_LIGHT` twice, before and
  after the `LIGHT_2`/`LIGHT_3` and Necromancer "unlight" adjustments.
* `borg_notice_equipment()`: the slay traits (`BI_WS_ANIMAL` ...
  `BI_WS_DRAGON`) are assigned, not or-ed, for every equipment slot, so
  they end up holding the slays of the last occupied, non-cursed slot
  (usually the boots, i.e. 0): the weapon's slays are lost. The
  `BI_WK_*` ("kill") traits are never set.
* `borg_notice_ammo()`: the "brand the ammo" test is copied twice (once
  inside `if (BI_AMMO_POWER >= 3)`, once after the `< 3` return), so each
  qualifying stack adds 20 to `BI_NEED_BRAND_WEAPON` instead of 10.
* `borg_notice_equipment()` tests `OF_PROT_FEAR`, `OF_PROT_BLIND`,
  `OF_PROT_CONF` and the sustains of the player flags with `rf_has` (size
  `RF_SIZE`) on an `OF_SIZE` array. Harmless: the object flags are below
  `8 * OF_SIZE`, so the same byte is read as with `of_has`.
* Rust (`borg_trait.rs`): all kept; the `rf_has` tests use `of_has`
  (same byte). Out-of-range table indexes (a negative stat index, a book
  number of -1, level 0 for `player_hp[lev - 1]`) panic where C would
  read or write out of bounds; none is reachable with valid game data.
* Milestone4 ([Rust#11](https://github.com/diegoami/borg-rs/issues/11)): source240 removes the duplicate brand-ammo count.
* Milestone3 ([Rust#8](https://github.com/diegoami/borg-rs/issues/8)): source239 or-accumulates slay traits and sets the
  `BI_WK_*` traits for multipliers of 5 or more; source238 adds the light
  modifier once; source313 weighs one weapon in `borg_calc_blows()`.
* Tracker: one lead per slip, in the order above:
  diegoami/neo-angband-tracker#237 to diegoami/neo-angband-tracker#241.
* Milestone9 ([Rust#21](https://github.com/diegoami/borg-rs/issues/21)): source18 counts Remove Hunger scrolls as food when there is no
  other food (0090), and source560 counts a Rod of Teleport Other in
  `BI_ATPORTOTHER` (0095), in C and Rust.

## 49. Dungeon-think slips (think dungeon)

* `borg-think-dungeon.c`, found by code reading; all checked against the
  c2rust code (see PROGRESS.md, module 57). Lunal and munchkin mode are
  off in the stock `borg.txt`, so the suites never reach them.
* `borg_think_dungeon_lunal()` (lines 132-136) and
  `borg_think_dungeon_munchkin()` (lines 487-491): the same "do I already
  know this up staircase" loop as `borg_think_stair_scum()`, with
  `continue` for `break`. Not a new slip: it is the one in section 47.
* `borg_think_dungeon_lunal()` (lines 241-363): `b_j`, the distance to
  the nearest staircase, is set by the "full of items" up-stair scan and
  is never reset before the down-stair scan or the second up-stair scan,
  so a close staircase of one kind makes the "close enough" test
  (`b_j < 8`) of the other kind pass. `borg_think_dungeon_munchkin()` has
  the same carry-over into its last up-stair scan (lines 773-791) and
  into the adjacent-monster scan (lines 848-886), where `b_j` also builds
  up from one adjacent monster to the next. (The same kind of slip as the
  `b_j` one in section 47.)
* `borg_think_dungeon_munchkin()` (lines 844 and 904-913): the
  adjacent-monster scan reuses `ag`, the pointer to the Borg's own grid,
  for each neighbour. After the scan it points at the last neighbour in
  bounds, so "Take the Up Stair" and "Take the Stair" test that
  neighbour's feature: the Borg can press `<` or `>` when it is not on a
  staircase, and does not take one it stands on.
* Same function (lines 586-588): "Increase the range of the borg a bit"
  adds `(clevel - 10) + (10 - depth)` to `closeness` (8). Below clevel 10
  this shrinks the range instead (to -1 for a clevel 1 Borg on level
  10), so no staircase counts as close.
* Rust (`borg_think_dungeon.rs`): all kept. A grid outside the map
  panics (C reads outside `borg_grids`), and so does a negative
  `PACK_SLOTS - n` index (C reads before `borg_items`).
* Milestone6 ([Rust#15](https://github.com/diegoami/borg-rs/issues/15)): source245 widens the munchkin range only above clevel 10;
  source246 resets the clock in `borg_think_dungeon()` instead of aborting
  (the `borg_think()` reset stays as a failsafe).
* Milestone3 ([Rust#8](https://github.com/diegoami/borg-rs/issues/8)): source244 re-acquires the Borg's own grid before the
  munchkin stair keys, in C and Rust.
* Tracker: the duplicated loop is a comment on
  diegoami/neo-angband-tracker#232; the other three slips are
  diegoami/neo-angband-tracker#243, #244, #245.
* Milestone7 ([Rust#17](https://github.com/diegoami/borg-rs/issues/17)): source403 leaves munchkin mode while poisoned or hallucinating
  (patch 0067), in C and Rust.

## 50. Think slips (think)

* `borg-think.c`, found by code reading; checked against the c2rust code
  (see PROGRESS.md, module 58).
* `borg_think()` (lines 238-245): the "clock overflow" fix (reset
  `borg.time.now` to 1 and warn once it passes `BORG_TIME_MAX - 100`) is
  dead in play. Every dungeon turn increments the clock and then calls
  `borg_think_dungeon()`, which aborts the Borg with `borg_oops("clock
  overflow")` as soon as the clock reaches `BORG_TIME_MAX - 3000`
  (`borg-think-dungeon.c`, lines 1281-1288); a turn in a store returns
  before the increment. The clock only reaches the reset if the user
  restarts the aborted Borg about 2,900 times, so in practice the Borg
  stops at the overflow it was meant to fix.
* Rust (`borg_think.rs`): kept (the reset and the warning are converted
  exactly; the cross-check reached them only by fuzzing the clock).
* Tracker: diegoami/neo-angband-tracker#246.

## 51. Attack slips (fight attack)

* `borg-fight-attack.c`, found by code reading and the cross-check.
* `borg_mon_blow_effect()` compares the name with every entry of its table,
  including the `{ NULL, MONBLOW_NONE }` end entry, so a name that is not in
  the table would reach `strcmp(name, NULL)` (every name in
  blow_effects.txt is in the table). Rust panics there.
* `borg_launch_damage_one()`:
  * `BORG_ATTACK_DRAIN_LIFE` (Dispel Life) zeroes the damage for monsters
    that are not `NONLIVING` and again for those that are not `UNDEAD`, so
    only monsters that are both take damage (the spell hurts the living).
  * `BORG_ATTACK_TAP_UNLIFE`: `sp_drain = CURSP - CURSP` is always 0.
  * The Teleport Other list (`borg_tp_other_*`, 255 entries) is filled
    without a bound check (C writes past the arrays on the 256th monster in
    one scan; Rust panics), as its reader in borg-danger.c (C bug 17).
* `borg_launch_bolt_aux_hack()`: for a `PASS_WALL` monster the wall scan
  leaves `x`/`y` at the monster's grid + (1, 1), and the "hard sleeping
  monster" danger is taken there, not at the monster's grid.
* `borg_launch_bolt_at_location()` and `borg_launch_arc_at_location()`:
  the blast loops run `y2 - rad <= ry < y2 + rad` (likewise `rx`, and
  `y1 - max .. y1 + max` for arcs), so the south and east edges of the
  radius are never counted.
* `borg_launch_arc_at_location()`: the angle test uses the end of the
  spell's path (`y - y1 + 20`, `x - x1 + 20`) for every grid instead of the
  grid looked at (`ry`, `rx`), so it gives the same answer for every grid:
  an arc counts every monster in range (and line of sight) or none.
* `borg_attack_aux_launch()`: when no missile finds a target (every
  `borg_launch_bolt` returns -1), `b_n` stays 0 and `b_k` -1, which passes
  the `b_n < 0` test; a real (not simulated) call then reads
  `borg_items[-1].desc` for its note and sends the quiver key
  `'0' - 1 - QUIVER_START`. Rust panics on the read. Seen only in the
  fuzz check (a real call is preceded by a simulation that found a target,
  except when `borg_has_distance_attack()` in borg-flow-kill.c runs with
  `borg_simulate` off).
* `borg_attack_aux_object()` has no `return` after its keys, so a real call
  returns whatever is in the return register (no caller uses the value of
  a real call; Rust returns `b_n`, as the c2rust translation did). The
  throwing range `b_r` is computed and never used.
* `borg_attack_aux_maim_foe()`: a real call returns `d`, the score of the
  last monster looked at, not the best one's `b_d`; its second
  `if (borg_simulate) return d;` is dead.
* `borg_attack_aux_vampire_strike()`: "require relatively close"
  (`dist >= 20`) tests the distance of the last monster in the list, not of
  the closest one it picked.
* `borg_attack_aux_spell_bolt_reserve()`: the monster loop `break`s at the
  first live monster, so `near_monsters` is at most 1 and "should only be
  1 near monster" never fails; only that monster's hit points are checked.
* `borg_attack_aux_spell_dispel()` applies the "mana needed for the final
  teleport" penalty (`spell_power * 750`) twice, once with the class's
  reserve and once with 6 (both for a Mage).
* `borg_calculate_attack_effectiveness()`: `BF_ACT_ACID_BOLT3` looks for
  `act_acid_bolt2`, `BF_ACT_DISPEL_ALL` for `act_dispel_undead`;
  `BF_SPELL_BLIND_CREATURE` casts Frighten scored as confusion;
  `BF_ACT_WONDER` and `BF_ACT_STAFF_HOLY` keep `rad` 0 (bolts);
  `BF_DRAGON_RED` uses an arc of 2 degrees where the other dragon armours
  use 20.
* Rust (`borg_fight_attack.rs`): all kept; the out-of-bounds reads and
  writes are panics.
* Milestone5 ([Rust#13](https://github.com/diegoami/borg-rs/issues/13)): source256 returns the best Maim Foe score `b_d`.
* Milestone4 ([Rust#11](https://github.com/diegoami/borg-rs/issues/11)): source247 stops the blow-name search before the NULL
  end entry; source252 includes the south/east edges in the ball and arc
  blast loops.
* Milestone3 ([Rust#8](https://github.com/diegoami/borg-rs/issues/8)): corrected in C and Rust — Dispel Life hurts only
  living monsters (source248); dispels/beams no longer collect or break
  as `rad == 10` bolts and a no-target launch returns 0 (source326); the
  arc tests the scanned grid's angle (source253); a launch with no chosen
  missile returns before using `b_k` (source254); Vampire Strike guards
  `best_dist` (source257); the reserve-bolt veto scans every near monster
  (source258); the final-teleport penalty applies once (source259);
  ACID_BOLT3/DISPEL_ALL use their own activations (source260); Blind
  Creature scores fear (source261); Staff Holy is an area dispel
  (source262). `BF_ACT_WONDER`'s radius and the blast-loop edges remain.
* Tracker: one lead per slip, in the order above (the last bullet is
  four):
  diegoami/neo-angband-tracker#247 to diegoami/neo-angband-tracker#263.

## 52. Multibyte screen text can be unterminated (io/messages)

* `borg-io.c`, `borg_what_text()`, observed in M4 D's synthetic saved-character
  replay and checked against the source at `e473720ed`.
* `wcstombs(s, screen_str, ABS(n) + 1)` limits **bytes**, although `n` is
  the number of wide screen characters. A 79-column line containing one
  two-byte UTF-8 character needs 81 bytes including its NUL. With the
  limit 80, `wcstombs` writes 80 bytes and no NUL. The function ignores the
  conversion result and still reports success. `borg_get_messages()` uses
  an uninitialised `char buffer[1024]`, then `borg_trim`/`strlen` treat the
  converted prefix as a terminated string and can read stale stack bytes.
* Observed: `end-depth98`, fixture `depth98.save`, replay seed 6001,
  `menu,cheat`, at turn 27899/key 34102. The two C passes are identical,
  but C produces an extra `PAIN:)` and "Assuming player ghost ())" note.
  A `borg_verbose=TRUE` diagnostic shows C parsing
  `The lord of Carn Dûm (offscreen) wakes up.` followed by spaces and the
  stale suffix `) wakes up.`; Rust parses just the real message. Keys and
  player-state logs remain identical for this entire 100k-turn scenario;
  no gameplay consequence beyond the spurious parsing is claimed.
* `harness/regress/wcstombs/run.sh` is a native libc-only control: 79 ASCII
  columns terminate; replacing one with `û` fills the 80-byte limit without
  a NUL and `strlen` reads a deliberately initialised stale tail. The probe
  avoids undefined reads itself; it does not reproduce arbitrary stack
  contents or establish an independently verified native game defect.
* Rust zero-initialises its scratch buffer. Arbitrary uninitialised C
  stack contents are not reproducible safe state. No C correction, Rust
  special case, reference filtering or fixture/seed change is applied.
  This remains a strict message-parity blocker for step D.
* Tracker: diegoami/neo-angband-tracker#287 (reported lead).


Owner-authorized correction (2026-11-06): both implementations reserve NUL
and return empty text/error on invalid encoding. The initial byte limit
still used screen columns and truncated valid UTF-8; seeds1001/1003 then
parsed phantom Sméagol names. Commit `26a96bee4570` uses actual caller
byte capacity (explicit size in C, slice length in Rust). The real-function
full-message regression fails before and passes after. Both builds again
match all 45 original full cases, and all five unchanged endgame cases
match current corrected C/C/Rust in keys/messages/state within the six-case
`crash-fixes` suite. See `validation/2026-11-06-capacity/README.md`.
The original-source description above is historical; original references
are preserved and independent tracker verification remains explicit.
* Milestone3 ([Rust#8](https://github.com/diegoami/borg-rs/issues/8)): consumer patch0025 (`ABS(n) * MB_CUR_MAX` bound) is not
  imported; the local capacity-based correction above already covers it.
* Milestone7 ([Rust#17](https://github.com/diegoami/borg-rs/issues/17)): the consumer corrected its patch 0025 with 0068 (pass the
  buffer size); borg-rs takes it in C and Rust, superseding the local
  correction above. The only behavioural difference is the `wcstombs()`
  error path, which now terminates the buffer at 0 instead of returning early.

## 53. Queued detection can address a panel outside resized arrays (update)

* M4 C, natural Hobbit Rogue seed 4006, `menu,cheat`, 2M-turn limit:
  both unmodified C Borg runs segfault (exit 139), and Rust aborts on a
  bounds check (exit 134). No complete summary exists for this scenario;
  it is a failed run, not an accepted reference.
* A separate Rust replay with `RUST_BACKTRACE=1` reproduces
  `borg_update.rs:202`, outer detection-array length 2, index 2. The trace
  is `set_detect -> mark4` (the `q_y + 1` write) `-> borg_handle_self ->
  handle_messages -> borg_update`. This is the natural run behind the
  generic detection-panel bounds warning in section 46.
* C `borg_handle_self()` derives a panel from the **old** offsets
  `o_w_x`/`o_w_y` and writes four detection cells without checking their
  bounds. `borg_update()` first resizes the arrays for the current cave.
  Its stale-offset check only resets offsets **greater than** the cave
  width/height; equality is not reset. For a 22-row town and old offset
  22, `q_y = 1` survives, while the two-row detection array has indices
  0 and 1; the `q_y + 1` write uses 2. The guard is copied faithfully.
* This source path and the matching Rust backtrace are evidence for a C
  bounds-write lead. A native C crash backtrace and the actual failing C
  panel coordinates have not been captured; attribution of the two C
  segfaults to this exact write remains a reported lead, not independent
  C verification. Raw failed logs are buffered/incomplete and do not
  establish an exact final game turn.
* No clamp, enlarged array, changed reset guard, seed removal or shorter
  accepted run is applied. C stays unchanged; Rust's bounds panic stays.
  The `extended` suite cannot pass its full scenario set with this fault.
* Tracker: diegoami/neo-angband-tracker#290 (reported lead).

Owner-authorized correction (2026-11-06): C and Rust now reset old offsets
at equality with cave dimensions. The original C signal observer faults
in `borg_update`, at `borg_detect_wall`'s next-row write (q_y=1, q_x=0,
NULL next-row pointer); its raw trace was not transferred to this checkout.
At `26a96bee4570`, corrected seed4006 finishes 2,000,352 turns in C twice and Rust once,
with identical keys/messages/state. See
`validation/2026-11-06-capacity/README.md`. The original-source description
above is historical; independent tracker verification remains explicit.
* Milestone3 ([Rust#8](https://github.com/diegoami/borg-rs/issues/8)): the accepted C correction (patch0018) bounds-checks every
  queued detection mark and the "cackles" reset against the current panel
  count, dropping stale reactions. It supersedes the local equality reset:
  that reset made a stale reaction stamp the new cave's panel (0,0) as
  detected (detect-290 `stale` fixture). C and Rust return to the pristine
  `>` reset plus the consumer guard; the original seed4006 2M run is rerun.

## 54. A stale monster on the player's square can stop melee play (fight-attack)

* `borg_attack_aux_thrust()` requires distance at most one, accepting zero.
  If a stale tracked monster occupies the player's coordinates after a
  teleport/blindness transition, scoring can select it and queue `+5`.
  Five is the center square, not an attack direction; the empty queue then
  trips `borg_react_prompted`'s unexpected-direction abort.
* Observed in both corrected-C passes and Rust at `26a96bee4570`, unchanged
  `ext-long-warrior-2000`, seed2000, `menu,cheat`, 5M-turn limit: teleport
  to (11,70), blindness, a tracked giant tarantula still at (11,70), then
  `+5` and `borg_stopped` at game turn1,887,392/key456,028. Both strict
  extended comparisons are 12/12 identical, but that early stop does not
  establish a completed 5M-turn Warrior run.
* The native regression compiles the actual C thrust body unchanged into
  a controlled scoring/engine fixture. Before: center target scores10 and
  queues `+5`, assertion fails. After the narrow C/Rust correction (distance
  exactly one): center/distant targets are rejected and all eight adjacent
  grids remain valid, in apply and simulation modes. The prompt-abort guard
  remains intact. Full corrected extended C/C and Rust/C now match12/12
  with all original limits reached; Warrior2000 finishes5,000,001 turns
  at clevel42/depth44 in all three passes. See
  `validation/2026-11-06-melee/README.md`.
* Tracker: diegoami/neo-angband-tracker#322. This is supplied consumer evidence,
  not independent tracker verification or upstream source resolution.
* Milestone3 ([Rust#8](https://github.com/diegoami/borg-rs/issues/8)): consumer patch0027 is the same `!= 1` correction; the
  local one is retained and not counted twice.

## 55. Calculator pointer-array allocations are not released (reported leads)

These are source-reading leads found while building the native calculator
checker, not measured leaks in a game and not independent tracker validation.
The C source is unchanged; no correction is proposed in this pass.

* `calc_free()` frees every token and the `token_array` struct, but not
  `token_array->items`, which `borg_array_add()` allocated separately.
  Releasing a parsed calculation loses its backing pointer array.
* `adjust_order_ops_depth_block()` frees the temporary operator tokens and
  the `operators` struct, but not its separately allocated `items` array.
  Each populated temporary operator list loses that allocation.
* `calculations_free()` resets `calculations.count` to zero but retains
  `calculations.items`; `borg_array_add()` allocates a new `items` whenever
  count is zero, overwriting that retained pointer on the next parse.
  Clearing and reloading calculations therefore loses the prior backing
  allocation. This is a separate reset/reuse slip from the two destructors.
* Rust vectors free/reuse their own storage. Allocator lifetimes are not a
  claimed key/message/state parity property. No arbitrary malformed-input,
  out-of-memory or gameplay-consequence claim is made.
* Tracker: one finding per distinct slip, in the order above:
  diegoami/neo-angband-tracker#323, #324 and #325.
* Milestone3 ([Rust#8](https://github.com/diegoami/borg-rs/issues/8)): C carries all three releases (patches0026/0033/0034);
  ASan/LSan fixtures show the leaks gone. Rust needs no change.

## 56. Glyph reverse flow can stop at the origin (reported lead)

* `borg_flow_glyph()` calls `borg_flow_reverse(250, true, false, false,
  -1, false)`. The reverse helper clears costs, enqueues the player's grid
  at cost0, and passes `optimize=true` to `borg_flow_spread` despite its
  "do NOT optimize" comment. Spread's first cost is1; optimization compares
  that against the origin's0 and breaks before processing neighbors.
* The glyph scan consequently sees only the player's grid below cost75.
  Once the center is glyphed and the old-center branch skips that tile,
  nearby unglyphed floor candidates can remain unreachable to this search.
  This can prevent progress on outlying runes until some external action
  moves the character. It is not a demonstrated whole-game halt or win-loss
  consequence, and no tactic correction is applied here.
* Evidence: source reading plus the native C/Rust builder fixture. The
  controlled open-room case with an existing one-glyph center returns
  false and retains one tracked glyph in both implementations. Creation,
  completed rings, duplicate tracking and full-capacity controls match too.
  No ordinary-play reproduction or independent tracker verification is
  claimed. Existing section29 slips and tracker#118 concern other defects.
* Tracker: diegoami/neo-angband-tracker#345 (reported lead).
  [Additional fixture evidence](https://github.com/diegoami/neo-angband-tracker/issues/345#issuecomment-6698804562)
  links881764e and was read back: a character placed one cell off the
  center selects that outlying cell (controlled follow-step success),
  while the existing-center case still returns false. The1600-state
  corpus matches C/Rust, including already-glyphed/outlying and failed
  creation/commit/step controls. This is not an ordinary-play reproduction.

* Post-M4 owner-authorized FORK correction: only this caller passes
  optimize=false to reverse flow, in both C and Rust. Real engine construction
  from an empty room with40 known Rune scrolls now produces25 engine glyphs,
 24 around the player,25 tracked C glyphs and Morgoth position true at
  turn177, leaving15 scrolls. The reverted C caller fails to build a pattern
  within500 turns on the same engine save. Three construction/control cases
  match corrected C twice and Rust on all logs and complete final saves;
  the read-only native C stop observer preserves those bytes. This uses a
  wizard-equipped Warrior and a distant awake Morgoth to isolate preparation;
  it is not a naturally earned setup, combat/survival improvement or a win.
  See validation/2026-11-07-post-m4-investigation. Independent source-tracker
  qualification remains separate.
  [Published consumer correction/evidence](https://github.com/diegoami/neo-angband-tracker/issues/345#issuecomment-6709066618)
  links d0803c6 and was read back.
* Milestone7 ([Rust#17](https://github.com/diegoami/borg-rs/issues/17)): the consumer carried this as patch 0065; C takes the consumer
  blob and Rust its comment. Behaviour is unchanged (the local #345 call was
  already the same).

## 57. Explicit X11 window count can bypass saved preferences (reported lead)

* `init_x11()` initializes the `settings` filename only inside
  `if (num_term == -1)`. An explicit `-nN` sets that count before the block,
  leaving the static filename empty. `term_data_init()` still tries to read
  that filename, and the quit preference writer uses it too. Saved font,
  tile and position settings can consequently be bypassed along with the
  window-count default. Documentation describes `-n` as selecting the count.
* Observed during the owner's Rust-launcher layout request: the original
  native C X11 frontend with a28-pixel Courier override and saved6x13 tiles
  opened482x314 with clipped glyphs; explicit `-n1` opened1362x722 with
  font-derived metrics. The launcher follows the requested C launcher's
  existing option pattern. No C frontend or Borg source correction is made.
* Evidence is source inspection plus consumer native GUI startup observations
  in `docs/validation/2026-11-07-launcher/`. This is a preference-loading
  reported lead, not an ordinary-play halt/win-loss result or independent
  tracker verification. It is separate from wide-panel bounds findings.
* Tracker: diegoami/neo-angband-tracker#357 (reported lead).
  This concerns window preferences only; the separate launcher save failure
  was an argument-order error fixed in the wrapper, not a C savefile defect.

## 58. Ordinary engine glyphs fail the map-discovery predicate (existing finding)

* Existing tracker#295 identifies `borg-update.c:330-338`: glyph assignment
  is nested under `square_isdisarmabletrap`. That predicate requires a
  visible TRAP; ordinary warding glyphs have GLYPH|VISIBLE|FLOOR, without
  TRAP. Thus ordinary glyph-only tiles do not enter that discovery branch.
* M4 synthetic native-engine evidence: a depth100 preparation case creates
  a real glyph and consumes one scroll (20 to19). A prelaid24-glyph ring
  case finishes with25 engine glyphs,23 visible, but the native C Borg map
  has zero glyph tiles and tracks only its latest cast. The C observation
  reads fields only at stop; keys/messages/player state and complete saved
  bytes match the unobserved native C run. No TRAP flag or Borg glyph field
  is injected by the fixture constructor. Normal C/C and Rust/C all match.
* The six-case scenario suite and receipts are under
  `harness/scenarios/` and `docs/validation/2026-11-07-synthetic-tactics/`.
  At that tag this is consumer native evidence, not independent tracker
  verification or a source correction. It is distinct from section56's reverse-flow
  search and section29's rollback/counting slips. No full formation-building,
  combat improvement or win is claimed.
* Tracker: diegoami/neo-angband-tracker#295 (existing source finding;
  [additional consumer native evidence](https://github.com/diegoami/neo-angband-tracker/issues/295#issuecomment-6700494859)
  links184ed94 and was read back).

* Post-M4 FORK correction: after the owner-authorized m4 tag at3923a21,
  C/Rust recognize the GLYPH flag reported by map_info's remembered trap,
  separately from harmful player traps. Real engine before/after controls
  cover a known Rune, decoy, harmful pit, hidden Rune, empty floor and the
  prelaid ring. The corrected ring has map24/tracked24, Morgoth position
  true and all20 scrolls preserved. The hidden Rune remains unknown and
  the harmful pit remains a trap. C/C/Rust match all six logs/final saves;
  both C observers preserve all unobserved output/save bytes. Original M4
  references remain intact; separate corrected references are retained.
  See `docs/validation/2026-11-07-glyph-recognition/README.md`. This fixes
  recognition, not section56's reverse flow or autonomous construction.
  Published correction/evidence: `da39bf5`; [tracker update read back](https://github.com/diegoami/neo-angband-tracker/issues/295#issuecomment-6705894975).
* Milestone9 ([Rust#21](https://github.com/diegoami/borg-rs/issues/21)): the consumer carried source295 as patch 0098, which replaces
  the local #295 correction in C and Rust. As written, 0098 reads the cave's
  trap after taking the glyph test from the player's memory; a rune broken
  out of view leaves that trap NULL. borg-rs adds a narrow NULL guard
  (FORK-CORRECTIONS, fixture `harness/adoption/m9/local/staleglyph`) and
  reported it on better-borg-c#375.

## 59. Wizard cure-all leaves known curses; Remove Curse then reads NULL (engine)

* `cmd-wizard.c`, `do_cmd_wiz_cure_all()`, with `ui-curse.c` `curse_menu()`.
  Observed in the C milestone4 adoption, case `cfg-lunal-deep-warrior`
  (seed 2007, `menu,cheat,lunal`), at game turn about 20076, in C and Rust alike.
* Cure-all frees each worn item's `obj->curses` and sets it to NULL but leaves
  `obj->known->curses`. `curse_menu()` filters on the known list and reads
  the real one, so a later Remove Curse dereferences NULL. The Borg reaches
  cure-all through its cheat-death answer to "Die? [y/n]" (`borg_react_prompted`);
  normal play reaches it only through the debug command.
* Engine code, shared by both images (Rust calls it through FFI). Unchanged on
  official master `e473720ed`.
* Corrected locally (owner-directed narrow engine fix): cure-all also frees the
  known list. See `FORK-CORRECTIONS.md` and
  [validation/2026-11-08-c-m4](validation/2026-11-08-c-m4/README.md).
* Tracker: diegoami/neo-angband-tracker#474. Adoption: [Rust#11](https://github.com/diegoami/borg-rs/issues/11).
