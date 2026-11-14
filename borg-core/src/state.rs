//! All Borg state in one place (the C globals of `angband/src/borg/*.c`).
//!
//! Each module owns a sub-struct here. The one instance is created and
//! handed out by `engine::enter` (the engine's callbacks into the Borg);
//! Borg code receives `&mut BorgState` explicitly.

use crate::borg::{Borg, CoreState};
use crate::borg_cave::CaveState;
use crate::borg_cave_light::LightState;
use crate::borg_cave_view::ViewState;
use crate::borg_danger::DangerState;
use crate::borg_fight_attack::FightAttackState;
use crate::borg_fight_defend::FightDefendState;
use crate::borg_flow::FlowState;
use crate::borg_flow_glyph::GlyphState;
use crate::borg_flow_take::TakeState;
use crate::borg_flow_kill::KillState;
use crate::borg_flow_misc::FlowMiscState;
use crate::borg_flow_stairs::StairsState;
use crate::borg_formulas::FormulasState;
use crate::borg_formulas_calc::CalcState;
use crate::borg_init::InitState;
use crate::borg_home_notice::HomeState;
use crate::borg_inventory::InventoryState;
use crate::borg_io::IoState;
use crate::borg_item::ItemState;
use crate::borg_item_activation::ActivationIds;
use crate::borg_item_use::ItemUseState;
use crate::borg_item_val::ItemValState;
use crate::borg_item_wear::WearState;
use crate::borg_magic::MagicState;
use crate::borg_messages::MessagesState;
use crate::borg_messages_react::MsgState;
use crate::borg_prepared::PreparedState;
use crate::borg_projection::ProjectionState;
use crate::borg_store::StoreState;
use crate::borg_store_sell::StoreSellState;
use crate::borg_store_buy::BuyState;
use crate::borg_think::ThinkState;
use crate::borg_think_dungeon::ThinkDungeonState;
use crate::borg_trait::TraitState;
use crate::borg_trait_swap::TraitSwapState;
use crate::borg_update::UpdateState;

#[derive(Default)]
pub struct BorgState {
    /// The token engine object handles borrow (`engine::Access`): a handle
    /// made from `&st.access` cannot be kept across an engine call that
    /// takes `&mut BorgState` (docs/SELF-REVIEW.md L4).
    pub access: crate::engine::Access,
    /// borg.c: `borg` (what the Borg knows about itself)
    pub borg: Borg,
    /// borg.c: the other globals (`borg_cfg`, `borg_rand_local`, ...)
    pub core: CoreState,
    /// borg-cave.c
    pub cave: CaveState,
    /// borg-cave-light.c: `borg_light_*`, `borg_glow_*`
    pub light: LightState,
    /// borg-cave-view.c: `borg_view_n`, `borg_view_x`, `borg_view_y`
    pub view: ViewState,
    /// borg-danger.c: `borg_fear_region`, `borg_fear_monsters`,
    /// `borg_danger_wipe`
    pub danger: DangerState,
    /// borg-fight-attack.c: `successful_target`, `target_closest`,
    /// `borg_tp_other_*`
    pub fight_attack: FightAttackState,
    /// borg-fight-defend.c: `borg_attempting_refresh_resist`
    pub fight_defend: FightDefendState,
    /// borg-flow.c: the flow data (`borg_data_*`), the flow queue,
    /// `borg_temp_*` and `track_step`/`track_door`/`track_closed`
    pub flow: FlowState,
    /// borg-flow-glyph.c: `track_glyph`, `borg_needs_new_sea` and the
    /// file statics (`glyph_x`, `glyph_y`, `glyph_*_center`)
    pub glyph: GlyphState,
    /// borg-flow-take.c: `borg_takes`, `borg_takes_cnt`, `borg_takes_nxt`
    pub takes: TakeState,
    /// borg-flow-kill.c: `borg_kills` (+ `_cnt`, `_nxt`),
    /// `borg_race_count`, `borg_race_death`, the unique/normal monster name
    /// tables and `borg_nasties*`
    pub kills: KillState,
    /// borg-flow-misc.c: `track_shop_x/y`, `track_vein` and the file
    /// statics `spastic_x`/`spastic_y`
    pub flow_misc: FlowMiscState,
    /// borg-flow-stairs.c: `track_less`, `track_more`
    pub stairs: StairsState,
    /// borg-formulas.c: `borg_formulas`
    pub formulas: FormulasState,
    /// borg-formulas-calc.c: `calculations`
    pub formulas_calc: CalcState,
    /// borg-home-notice.c: the home counters (`num_*`, `home_*`)
    pub home: HomeState,
    /// borg-init.c: `borg_initialized`, `game_closed`, `borg_init_failure`
    pub init: InitState,
    /// borg-inventory.c: `borg_do_crush_junk`
    pub inventory: InventoryState,
    /// borg-io.c
    pub io: IoState,
    /// borg-item.c: `borg_items`, `safe_items`
    pub items: ItemState,
    /// borg-item-activation.c: `act_*`
    pub act: ActivationIds,
    /// borg-item-use.c: `when_last_quaff` (static in `borg_quaff_crit`)
    pub item_use: ItemUseState,
    /// borg-item-val.c: `sv_*`, `kv_*`
    pub item_val: ItemValState,
    /// borg-item-wear.c: `track_worn_*`
    pub wear: WearState,
    /// borg-magic.c: `borg_magics`, `borg_spell_ratings`
    pub magic: MagicState,
    /// borg-messages.c / borg-messages-react.c: the message queue
    /// (`borg_msg_*`)
    pub msgs: MsgState,
    /// borg-messages.c: `borg_match`, the message tables (`suffix_pain`,
    /// `suffix_hit_by`, `spell_msgs`, `spell_invis_msgs`) and the
    /// `borg_parse` buffer
    pub messages: MessagesState,
    /// borg-prepared.c: `borg_numb_live_unique`, `borg_first_living_unique`,
    /// `borg_depth_hunted_unique`
    pub prepared: PreparedState,
    /// borg-projection.c: `borg_target_loc`
    pub projection: ProjectionState,
    /// borg-store.c: `borg_shops`, `borg_safe_shops`
    pub store: StoreState,
    /// borg-store-sell.c: `sold_item_*`, `test_item`, `best_item`
    pub store_sell: StoreSellState,
    /// borg-store-buy.c: `bought_item_*` and the statics
    /// `borg_money_scum_who`/`borg_money_scum_ware`
    pub buy: BuyState,
    /// borg-think.c: `shop_num`, `borg_do_spell` and the statics of
    /// `borg_check_save` (`svSavefile`, `justSaved`)
    pub think: ThinkState,
    /// borg-think-dungeon.c: the level feelings (`borg_feeling_danger`,
    /// `borg_feeling_stuff`)
    pub think_dungeon: ThinkDungeonState,
    /// borg-trait-swap.c: `weapon_swap`, `armour_swap`, their values,
    /// enchantment needs and flags (`weapon_swap_*`, `armour_swap_*`)
    pub trait_swap: TraitSwapState,
    /// borg-update.c: the detection arrays (`borg_detect_*`, `old_panels`),
    /// `borg_failure`, `borg_do_update_view/lite`, the monster/object
    /// tracking list (`borg_wanks`) and the old panel and location
    pub update: UpdateState,
    /// borg-trait.c: `borg_simulate`, `borg_attacking`, the defence flags
    /// (`borg_on_glyph`, `borg_create_door`, the `borg_*_spell` flags) and
    /// `borg_game_ratio`
    pub traits: TraitState,
}
