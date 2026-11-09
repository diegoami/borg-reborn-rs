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
- **S**: the seam itself.

## Functions

| Symbol | Reason | What for |
| --- | --- | --- |
| `map_info` | B | what the player sees and remembers at a panel grid (`borg_update_map`) |
| `square_in_bounds_fully` | B | bounds check before every grid read |
| `square_object` | B | the real top object on a seen grid (`borg_get_top_object`) |
| `square_isvault` | B | vault at the player's grid (danger, retreat) |
| `square_iswebbed` | B | webbed panel grid (`borg_update_map`) |
| `square_shopnum` | B | shop entrance number (`borg_update_map`) |
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

## Constants

| Symbol | Reason | What for |
| --- | --- | --- |
| `TMD_MAX` | B | the number of timed effects |
| `TMD_FOOD` | B | the food timer |
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
