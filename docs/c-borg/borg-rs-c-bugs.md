# Known C Borg bugs (reproduced faithfully)

The Rust port keeps each of these exactly as the C behaves. They are recorded
here so nobody "fixes" one by accident.

Format: file, function, (seed, turn when observed), explanation.

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
* Rust: `crate::bridge::st().borg.has[BI_CURHP as usize]` (same index into the
  same table).

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
  clippy rejects an `if` with the same condition twice; a comment marks it).
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
* Rust: kept (`st.msgs.dont_react` is only ever set; `st.borg.dont_react`
  is reset) in `borg_messages_react.rs` / `borg.rs`.
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
* Rust (`borg_inventory.rs`): both kept.
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
* Rust: all kept in `borg_danger.rs`.

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
  `borg_item_analyze.rs`.
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
* Rust (`borg_flow_take.rs`): all kept.

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
* Rust: all kept in `borg_item_use.rs`.
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
* Rust (`borg_item_wear.rs`): all kept (`swap_letter(-1)` gives the 0 key).

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
* Rust (`borg_flow_kill.rs`): all kept.

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
* Rust (`borg_flow_stairs.rs`): kept (`no_rest as i16` truncates the same
  way; `b_stair` is only checked against the allocated size).

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
* Rust (`borg_fight_perm.rs`): all kept (`as i16` truncation for the
  level timer).
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
* Rust (`borg_escape.rs`): all kept.
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
