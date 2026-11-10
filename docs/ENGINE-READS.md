# Engine reads (the D3 list)

*docs/SPEC.md §5: the normative list of every engine symbol the Rust Borg
uses, and why. `engine/tests/test_reads.py` fails when `engine.rs` uses a
symbol that is not here, or when a symbol here is no longer used. The
generated bindings (`borg-reborn-rs/src/bindings.rs`, docs/SPEC.md §3.4)
hold exactly the declarations these need.*

Reasons, from the C Borg specification (`reference/c-borg/BORG_SPEC.md`
§1.1) and the spec:

- **A**: channel A, the screen;
- **B**: channel B, a direct read the C Borg makes too;
- **K**: static game data (`r_info`, `k_info`, …), which the C Borg reads
  for danger and item scoring;
- **X**: a declared extra (docs/SPEC.md §5): an engine value the C Borg
  computes for itself, read because the engine's figure is the right one or
  to check our own;
- **W**: a write: the key hook, the options the Borg sets, the RNG swap;
- **S**: the seam itself;
- **T**: a read of the engine-truth check (task T2.5, feature
  `test-perception`): test builds only, never compiled into the Borg that
  plays. It reads the engine through other routes than the observation's,
  to check it (`engine.rs`, below the `ENGINE TRUTH` line; `test_reads.py`
  checks they are used nowhere else).

## Functions

| Symbol | Reason | What for |
| --- | --- | --- |
| `map_info` | B | what the player sees and remembers at a panel grid (`borg_update_map`) |
| `square_in_bounds_fully` | B | bounds check before every grid read |
| `square_object` | B | the real top object on a seen grid (`borg_get_top_object`) |
| `square_isvault` | B | vault at the player's grid (danger, retreat) |
| `square_iswebbed` | B | webbed panel grid (`borg_update_map`), only on a grid the player knows as floor, as the C Borg guards it (`borg-update.c:542`; M2 review, finding 1) |
| `square_shopnum` | B | shop entrance number (`borg_update_map`), only on a grid the player knows as a shop |
| `feat_is_shop` | B | the C Borg's guard before the shop number: the known feature is a shop (`borg-update.c:317`) |
| `cave_monster` | B | the real monster on a seen grid: race, HP, timers (`borg_update_kill_old`) |
| `cmd_get_nrepeats` | B | the game is repeating a command (spec §2.7) |
| `get_com` | S | reads the `^Z` command key |
| `option_set` | W | the game options the Borg sets |
| `spell_by_index` | B | the class's spells (`borg_cheat_spells`) |
| `store_stock_list` | B | every store's stock (`borg_notice_store`) |
| `price_item` | X | a ware's price as the shop shows it; the C Borg computes its own (`borg_price_item`) |
| `Term_inkey` | S | a real key the user pressed stops the Borg (spec §2.2) |
| `object_desc` | B | item descriptions (`borg_item_analyze`) |
| `object_flags_known` | B | known object flags (`borg_item_analyze`) |
| `object_flavor_is_aware` | B | flavour awareness (`borg_item_analyze`) |
| `object_fully_known` | B | fully known (`borg_item_analyze`) |
| `object_runes_known` | B | runes known (`borg_item_analyze`) |
| `object_weight_one` | B | item weight (`borg_item_analyze`) |
| `pack_slots_used` | X | the engine's own pack count, quiver included (REDESIGN-LESSONS L8) |
| `square` | T | the player's map memory (`player->cave`), against the world's map |
| `panel_contains` | T | the engine's own panel test (`textui_panel_contains`), against the observed panel |
| `equipped_item_slot` | T | where an object of the gear list is worn, without the body slots perception reads |
| `object_is_in_quiver` | T | whether an object of the gear list is in the quiver |
| `monster_is_visible` | B | `map_info`'s own visibility test (`cave-map.c`): tells a monster the player sees from the `m_idx = 1` a hallucination puts on a grid where monster 1 stands unseen (T2.5); the truth check also uses it on the whole level |

## Globals

| Symbol | Reason | What for |
| --- | --- | --- |
| `player` | B | the character: status, timed effects, gear, spells, options |
| `cave` | B | the current level, for the grid reads above |
| `turn` | B | the game turn |
| `z_info` | B | the engine's sizes (pack, quiver, stores, data counts, move energy) |
| `Term` | A | the screen: message line, store header, map panel offset |
| `row_top_map` | A | map panel geometry |
| `row_bottom_map` | A | map panel geometry |
| `col_map` | A | map panel geometry |
| `tile_width` | A | map panel geometry |
| `tile_height` | A | map panel geometry |
| `stores` | B | the stores, for their stock |
| `r_info` | K | monster races |
| `k_info` | K | object kinds |
| `e_info` | K | ego items |
| `a_info` | K | artifacts |
| `classes` | K | player classes and their spells |
| `races` | K | player races |
| `extract_energy` | K | energy per game turn by speed |
| `adj_str_blow` | K | the blows table's strength term, read to check the lifted copy |
| `adj_str_hold` | K | the wield-weight table, read to check the lifted copy |
| `f_info` | T | terrain features, to show a remembered feature as `map_info` does (mimics) |
| `Rand_quick` | W | the RNG swap: the Borg's reads use its own RNG (spec §1.2) |
| `Rand_value` | W | the RNG swap |
| `state_i` | W | the RNG swap: saved and restored |
| `STATE` | W | the RNG swap: saved and restored |
| `inkey_hack` | S | the key hook; declared in `engine.rs` (upstream declares it in no header) |

## Types read field by field

| Symbol | Reason | What for |
| --- | --- | --- |
| `chunk` | B | the level (`cave`) |
| `grid_data` | B | `map_info`'s result |
| `loc` | B | grid coordinates |
| `object` | B | items: gear, floor, stock |
| `object_kind` | K | an item's kind |
| `element_info` | B | resistances of items and races |
| `random_value` | K | dice of the static data |
| `keypress` | S | the key the hook returns |
| `ui_event` | S | `Term_inkey`'s result |
| `player` | B | the character (the global's type): grid, depth, levels, HP, SP, stats, timed effects, gear lists, spells, options |
| `player_race` | B | the race played (`ridx`); K for the static race table |
| `player_class` | B | the class played (`cidx`, its magic); K for the static class table |
| `player_state` | B | computed stats, speed, blows, AC |
| `player_options` | X | `birth_percent_damage` |
| `player_upkeep` | B | resting, running, the pack and quiver arrays |
| `player_body` | B | the equipment slots |
| `equip_slot` | B | a slot's type and object |
| `class_magic` | K | a class's books and spells |
| `class_book` | K | a book's spells |
| `class_spell` | B | a spell's book, level, mana and failure (`borg_cheat_spells`) |
| `term` | A | screen size, sidebar mode, the map panel offset and the screen window (`scr`) |
| `term_win` | A | the screen buffer (`c`): the message line and the store header |
| `random` | K | dice of the static data (`random_value` is its alias): base, dice, sides, bonus |
| `monster` | B | a seen monster: index, race, grid, HP, timers, speed, colour, mimicry |
| `monster_race` | K | monster races |
| `monster_base` | K | a race's base (its name) |
| `monster_blow` | K | a race's blows |
| `blow_method` | K | a blow's method |
| `blow_effect` | K | a blow's effect |
| `ego_item` | K | ego items |
| `artifact` | K | artifacts |
| `store` | B | a store, for its stock and prices; K for its always and normal tables |
| `angband_constants` | B | `z_info`: the engine's sizes |
| `square` | T | a remembered grid's feature (`player->cave`) |
| `feature` | T | a feature's mimic |

## Constants

| Symbol | Reason | What for |
| --- | --- | --- |
| `TMD_MAX` | B | the number of timed effects |
| `TMD_FOOD` | B | the food timer |
| `FEAT_FLOOR` | B | the web guard: a known floor (`borg-update.c:542`) |
| `TMD_IMAGE` | B | hallucination (the RNG-isolation test build only) |
| `OPT_birth_percent_damage` | X | a birth option the blows formula needs |
| `OBJ_NOTICE_IGNORE` | B | an ignored object is not the top object (`borg_get_top_object`) |
| `ODESC_FULL` | B | description mode (`borg_item_analyze`) |
| `ODESC_STORE` | B | description mode for wares (`borg_notice_store`) |
| `EVT_KBRD` | S | a keyboard event |
| `EVT_NONE` | S | no key: let the game's rest or repeat run |
| `EVT_DISCONNECT` | S | the terminal went away |

## Not engine reads

The harness frontend's `borgh_decision` (declared in `engine.rs`, harness
builds only) and the Borg's own export `do_cmd_borg` belong to the seam,
not to the game's state.
