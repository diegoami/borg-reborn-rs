//! Buying-strategy selection for the post-M10 Borg family (borg-rs#25).
//!
//! `current` preserves the accepted impact-v10 behaviour. `reserve410` is an
//! experimental early escape/cure reserve inspired by better-borg-c#410, whose
//! strict version was rejected after failing its declared survival/progress
//! margins. It is opt-in and carries no claim of benefit.

use crate::borg_headers::borg_h::{
    BORG_BUY_STRATEGY_CURRENT_CLASSES, BORG_BUY_STRATEGY_CURRENT_MUNCHKIN_CLASSES,
    BORG_BUY_IDENTIFY, BORG_BUY_STRATEGY_DEFAULT, BORG_BUY_STRATEGY_MUNCHKIN,
    BORG_BUY_STRATEGY_RESERVE410_CLASSES, BORG_BUY_STRATEGY_RESERVE410_MUNCHKIN_CLASSES,
    BORG_MUNCHKIN_START,
};
use crate::borg_headers::borg_item_h::borg_item;
use crate::borg_headers::borg_trait_h::{
    BI_ACCW, BI_ACLW, BI_ACSW, BI_APHASE, BI_CLASS, BI_FOOD, BI_ISCUT, BI_ISPOISONED, BI_LIGHT,
    BI_MAXCLEVEL,
};
use crate::headers::list_tvals_h::{TV_POTION, TV_SCROLL};
use crate::state::BorgState;
use crate::engine;

/// `borg_buy_strategy_default = 0`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuyingStrategy {
    Current,
    Reserve410,
}

impl BuyingStrategy {
    fn from_setting(value: i32) -> Self {
        if value == 1 {
            Self::Reserve410
        } else {
            Self::Current
        }
    }
}

/// The four Phase Doors and three CLW/CSW/CCW potions evaluated by #410.
pub const RESERVE410_MAXCLEVEL: i32 = 7;
pub const RESERVE410_PHASE: i32 = 4;
pub const RESERVE410_CURE: i32 = 3;
/// The independent learning window of borg-rs#27.
pub const IDENTIFY_MAXCLEVEL: i32 = 7;
/// All nine classes, for configuration validation and documentation.
pub const CLASS_MASK_ALL: i32 = 0x1ff;

fn class_bit(class: i32) -> i32 {
    if (0..9).contains(&class) {
        1 << class
    } else {
        0
    }
}

/// Validate the seven family settings. A class may override a less specific
/// selection in either direction, but current and reserve masks at the same
/// precedence may not conflict.
pub fn config_error(cfg: &[i32]) -> Option<&'static str> {
    let get = |index: u32| cfg.get(index as usize).copied().unwrap_or(0);
    if !matches!(get(BORG_BUY_STRATEGY_DEFAULT), 0 | 1) {
        return Some("borg_buy_strategy_default must be 0 (current) or 1 (reserve410)");
    }
    if !matches!(get(BORG_BUY_STRATEGY_MUNCHKIN), -1..=1) {
        return Some("borg_buy_strategy_munchkin must be -1, 0 or 1");
    }
    if !matches!(get(BORG_BUY_IDENTIFY), 0 | 1) {
        return Some("borg_buy_identify must be 0 or 1");
    }
    for (index, name) in [
        (
            BORG_BUY_STRATEGY_CURRENT_CLASSES,
            "borg_buy_strategy_current_classes",
        ),
        (
            BORG_BUY_STRATEGY_RESERVE410_CLASSES,
            "borg_buy_strategy_reserve410_classes",
        ),
        (
            BORG_BUY_STRATEGY_CURRENT_MUNCHKIN_CLASSES,
            "borg_buy_strategy_current_munchkin_classes",
        ),
        (
            BORG_BUY_STRATEGY_RESERVE410_MUNCHKIN_CLASSES,
            "borg_buy_strategy_reserve410_munchkin_classes",
        ),
    ] {
        let value = get(index);
        if !(0..=CLASS_MASK_ALL).contains(&value) {
            return Some(name);
        }
    }
    if get(BORG_BUY_STRATEGY_CURRENT_CLASSES) & get(BORG_BUY_STRATEGY_RESERVE410_CLASSES) != 0 {
        return Some("current/reserve410 class masks conflict");
    }
    if get(BORG_BUY_STRATEGY_CURRENT_MUNCHKIN_CLASSES)
        & get(BORG_BUY_STRATEGY_RESERVE410_MUNCHKIN_CLASSES)
        != 0
    {
        return Some("current/reserve410 class+munchkin masks conflict");
    }
    None
}

/// Resolve class+munchkin, then class, then munchkin, then the global default.
/// Unspecified combinations select `current` through the zero-valued defaults.
pub fn selected(cfg: &[i32], class: i32, munchkin: bool) -> BuyingStrategy {
    let get = |index: u32| cfg.get(index as usize).copied().unwrap_or(0);
    let bit = class_bit(class);
    let mut strategy = BuyingStrategy::from_setting(get(BORG_BUY_STRATEGY_DEFAULT));

    if munchkin && get(BORG_BUY_STRATEGY_MUNCHKIN) >= 0 {
        strategy = BuyingStrategy::from_setting(get(BORG_BUY_STRATEGY_MUNCHKIN));
    }
    if get(BORG_BUY_STRATEGY_CURRENT_CLASSES) & bit != 0 {
        strategy = BuyingStrategy::Current;
    }
    if get(BORG_BUY_STRATEGY_RESERVE410_CLASSES) & bit != 0 {
        strategy = BuyingStrategy::Reserve410;
    }
    if munchkin {
        if get(BORG_BUY_STRATEGY_CURRENT_MUNCHKIN_CLASSES) & bit != 0 {
            strategy = BuyingStrategy::Current;
        }
        if get(BORG_BUY_STRATEGY_RESERVE410_MUNCHKIN_CLASSES) & bit != 0 {
            strategy = BuyingStrategy::Reserve410;
        }
    }
    strategy
}

pub fn selected_for(st: &BorgState) -> BuyingStrategy {
    selected(
        &st.core.cfg,
        st.borg.tr(BI_CLASS),
        st.cfg(BORG_MUNCHKIN_START) != 0,
    )
}

/// Whether the experimental early reserve is active and still has a deficit.
/// A radius-1 light (`BI_LIGHT == 1`) counts as urgent: the existing
/// spare-torch top-up keeps priority over reserve buying.
pub fn reserve410_unmet(st: &BorgState) -> bool {
    if selected_for(st) != BuyingStrategy::Reserve410
        || st.borg.tr(BI_MAXCLEVEL) > RESERVE410_MAXCLEVEL
        || st.borg.tr(BI_FOOD) == 0
        || st.borg.tr(BI_LIGHT) <= 1
        || st.borg.tr(BI_ISCUT) != 0
        || st.borg.tr(BI_ISPOISONED) != 0
    {
        return false;
    }
    st.borg.tr(BI_APHASE) < RESERVE410_PHASE
        || st.borg.tr(BI_ACLW) + st.borg.tr(BI_ACSW) + st.borg.tr(BI_ACCW) < RESERVE410_CURE
}

/// Whether the independent buy-to-identify policy may run now. Urgent states
/// keep the existing immediate paths; later levels use the old valuation. A
/// radius-1 light (`BI_LIGHT == 1`) is urgent too: the spare-torch top-up must
/// not lose its money to a learning purchase.
pub fn buy_identify_enabled(st: &BorgState) -> bool {
    st.cfg(BORG_BUY_IDENTIFY) != 0
        && st.borg.tr(BI_MAXCLEVEL) <= IDENTIFY_MAXCLEVEL
        && st.borg.tr(BI_FOOD) != 0
        && st.borg.tr(BI_LIGHT) > 1
        && st.borg.tr(BI_ISCUT) == 0
        && st.borg.tr(BI_ISPOISONED) == 0
}

/// The deliberately small #27 learning list. Phase Door and Recall are not
/// learning purchases: they belong to the separate survival-reserve policy.
pub fn is_learning_kind(st: &BorgState, item: &borg_item) -> bool {
    let tval = u32::from(item.tval);
    let sval = i32::from(item.sval);
    (tval == TV_POTION
        && (sval == st.item_val.sv_potion_cure_light
            || sval == st.item_val.sv_potion_cure_serious))
        || (tval == TV_SCROLL
            && (sval == st.item_val.sv_scroll_satisfy_hunger
                || sval == st.item_val.sv_scroll_detect_invis
                || sval == st.item_val.sv_scroll_monster_confusion))
}

/// Whether a learning-list item is still unlearned, given the player's
/// awareness of its kind. Split from the engine lookup so the gate is
/// unit-testable without an engine entry.
pub fn is_learning_candidate_kind(st: &BorgState, item: &borg_item, aware: bool) -> bool {
    is_learning_kind(st, item) && !aware
}

/// A learning-list kind the player has not already learned. Shop analysis
/// records the real kind index; the engine changes `kind->aware` after the
/// purchase, which makes the next copy ineligible.
pub fn is_learning_candidate(st: &BorgState, item: &borg_item) -> bool {
    is_learning_candidate_kind(st, item, engine::kind(item.kind as usize).aware)
}

/// An item counted toward, and therefore retained for, the #410 reserve.
pub fn is_reserve410_item(st: &BorgState, item: &borg_item) -> bool {
    let tval = u32::from(item.tval);
    let sval = i32::from(item.sval);
    (tval == TV_SCROLL && sval == st.item_val.sv_scroll_phase_door)
        || (tval == TV_POTION
            && (sval == st.item_val.sv_potion_cure_light
                || sval == st.item_val.sv_potion_cure_serious
                || sval == st.item_val.sv_potion_cure_critical))
}

/// Keep a carried reserve item out of the home-sell pass while it is still
/// needed. This is the anti-buy/store-loop half of the #410 experiment.
pub fn retain_reserve410_item(st: &BorgState, item: &borg_item) -> bool {
    item.iqty != 0 && reserve410_unmet(st) && is_reserve410_item(st, item)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::borg_headers::borg_trait_h::{BI_MAX, BI_MAXCLEVEL};
    use crate::borg_item::blank_item;

    fn cfg() -> Vec<i32> {
        let mut cfg = vec![0; crate::borg_headers::borg_h::BORG_MAX_SETTINGS as usize];
        cfg[BORG_BUY_STRATEGY_MUNCHKIN as usize] = -1;
        cfg
    }

    fn state(strategy: i32) -> BorgState {
        let mut st = BorgState::default();
        st.core.cfg = cfg();
        st.core.cfg[BORG_BUY_STRATEGY_DEFAULT as usize] = strategy;
        st.borg.trait_ = vec![0; BI_MAX as usize];
        st.borg.trait_[BI_FOOD as usize] = 1;
        // A carried torch: LIGHT_2 sets BI_LIGHT to 2.
        st.borg.trait_[BI_LIGHT as usize] = 2;
        st.borg.trait_[BI_MAXCLEVEL as usize] = 1;
        st
    }

    #[test]
    fn defaults_and_precedence_cover_class_and_mode() {
        let c = cfg();
        for class in 0..9 {
            for munchkin in [false, true] {
                assert_eq!(selected(&c, class, munchkin), BuyingStrategy::Current);
            }
        }

        let mut c = cfg();
        c[BORG_BUY_STRATEGY_DEFAULT as usize] = 1;
        assert_eq!(selected(&c, 0, false), BuyingStrategy::Reserve410);
        c[BORG_BUY_STRATEGY_MUNCHKIN as usize] = 0;
        assert_eq!(selected(&c, 0, true), BuyingStrategy::Current);
        c[BORG_BUY_STRATEGY_RESERVE410_CLASSES as usize] = 1;
        assert_eq!(selected(&c, 0, true), BuyingStrategy::Reserve410);
        c[BORG_BUY_STRATEGY_CURRENT_MUNCHKIN_CLASSES as usize] = 1;
        assert_eq!(selected(&c, 0, true), BuyingStrategy::Current);
        c[BORG_BUY_STRATEGY_CURRENT_MUNCHKIN_CLASSES as usize] = 0;
        c[BORG_BUY_STRATEGY_RESERVE410_MUNCHKIN_CLASSES as usize] = 1;
        assert_eq!(selected(&c, 0, true), BuyingStrategy::Reserve410);
        assert_eq!(selected(&c, 0, false), BuyingStrategy::Reserve410);
        assert_eq!(selected(&c, 1, true), BuyingStrategy::Current);
    }

    #[test]
    fn masks_can_describe_different_class_mode_pairs() {
        let mut c = cfg();
        // Warrior: reserve for both modes. Mage: current for both modes.
        c[BORG_BUY_STRATEGY_RESERVE410_CLASSES as usize] = 0b01;
        c[BORG_BUY_STRATEGY_CURRENT_CLASSES as usize] = 0b10;
        assert_eq!(selected(&c, 0, false), BuyingStrategy::Reserve410);
        assert_eq!(selected(&c, 0, true), BuyingStrategy::Reserve410);
        assert_eq!(selected(&c, 1, false), BuyingStrategy::Current);
        assert_eq!(selected(&c, 1, true), BuyingStrategy::Current);

        // Reverse Mage only for munchkin, without changing Warrior.
        c[BORG_BUY_STRATEGY_CURRENT_MUNCHKIN_CLASSES as usize] = 0b01;
        c[BORG_BUY_STRATEGY_RESERVE410_MUNCHKIN_CLASSES as usize] = 0b10;
        assert_eq!(selected(&c, 0, true), BuyingStrategy::Current);
        assert_eq!(selected(&c, 1, true), BuyingStrategy::Reserve410);
        assert_eq!(selected(&c, 1, false), BuyingStrategy::Current);
    }

    #[test]
    fn invalid_values_and_conflicts_are_rejected() {
        let mut c = cfg();
        assert_eq!(config_error(&c), None);
        c[BORG_BUY_STRATEGY_DEFAULT as usize] = 2;
        assert!(config_error(&c).is_some());
        c = cfg();
        c[BORG_BUY_STRATEGY_MUNCHKIN as usize] = -2;
        assert!(config_error(&c).is_some());
        c = cfg();
        c[BORG_BUY_IDENTIFY as usize] = 2;
        assert!(config_error(&c).is_some());
        c = cfg();
        c[BORG_BUY_STRATEGY_RESERVE410_CLASSES as usize] = 0x200;
        assert!(config_error(&c).is_some());
        c = cfg();
        c[BORG_BUY_STRATEGY_CURRENT_CLASSES as usize] = 1;
        c[BORG_BUY_STRATEGY_RESERVE410_CLASSES as usize] = 1;
        assert!(config_error(&c).is_some());
        c = cfg();
        c[BORG_BUY_STRATEGY_CURRENT_MUNCHKIN_CLASSES as usize] = 2;
        c[BORG_BUY_STRATEGY_RESERVE410_MUNCHKIN_CLASSES as usize] = 2;
        assert!(config_error(&c).is_some());
    }

    #[test]
    fn reserve_gate_has_window_deficit_urgent_and_opt_in_faces() {
        let mut st = state(0);
        assert!(!reserve410_unmet(&st), "current never enters the reserve");
        st.core.cfg[BORG_BUY_STRATEGY_DEFAULT as usize] = 1;
        assert!(reserve410_unmet(&st));
        st.borg.trait_[BI_APHASE as usize] = RESERVE410_PHASE;
        st.borg.trait_[BI_ACLW as usize] = RESERVE410_CURE;
        assert!(!reserve410_unmet(&st));
        st.borg.trait_[BI_ACSW as usize] = 1;
        st.borg.trait_[BI_ACLW as usize] = 0;
        st.borg.trait_[BI_MAXCLEVEL as usize] = RESERVE410_MAXCLEVEL + 1;
        assert!(!reserve410_unmet(&st));
        st.borg.trait_[BI_MAXCLEVEL as usize] = 1;
        assert!(reserve410_unmet(&st));
        // Both a dead light (0) and a radius-1 light (1) leave the spare-torch
        // top-up in front of the reserve.
        for light in [0, 1] {
            st.borg.trait_[BI_LIGHT as usize] = light;
            assert!(!reserve410_unmet(&st));
        }
        st.borg.trait_[BI_LIGHT as usize] = 2;
        for index in [BI_FOOD, BI_ISCUT, BI_ISPOISONED] {
            let old = st.borg.trait_[index as usize];
            st.borg.trait_[index as usize] = if index == BI_FOOD { 0 } else { 1 };
            assert!(!reserve410_unmet(&st));
            st.borg.trait_[index as usize] = old;
        }
    }

    #[test]
    fn reserve_items_include_only_phase_and_the_three_cure_tiers() {
        let mut st = state(1);
        st.item_val.sv_scroll_phase_door = 10;
        st.item_val.sv_potion_cure_light = 20;
        st.item_val.sv_potion_cure_serious = 21;
        st.item_val.sv_potion_cure_critical = 22;
        let mut item = blank_item();
        item.tval = TV_SCROLL as u8;
        item.sval = 10;
        assert!(is_reserve410_item(&st, &item));
        item.sval = 11;
        assert!(!is_reserve410_item(&st, &item));
        item.tval = TV_POTION as u8;
        for sval in [20, 21, 22] {
            item.sval = sval;
            assert!(is_reserve410_item(&st, &item));
        }
        item.sval = 23;
        assert!(!is_reserve410_item(&st, &item));
    }

    #[test]
    fn identify_gate_is_opt_in_early_and_nonurgent() {
        let mut st = state(0);
        assert!(!buy_identify_enabled(&st));
        st.core.cfg[BORG_BUY_IDENTIFY as usize] = 1;
        assert!(buy_identify_enabled(&st));
        st.borg.trait_[BI_MAXCLEVEL as usize] = IDENTIFY_MAXCLEVEL + 1;
        assert!(!buy_identify_enabled(&st));
        st.borg.trait_[BI_MAXCLEVEL as usize] = 1;
        // Light at radius 1 or below keeps the spare-torch top-up first.
        for light in [0, 1] {
            st.borg.trait_[BI_LIGHT as usize] = light;
            assert!(!buy_identify_enabled(&st));
        }
        st.borg.trait_[BI_LIGHT as usize] = 2;
        for index in [BI_FOOD, BI_ISCUT, BI_ISPOISONED] {
            let old = st.borg.trait_[index as usize];
            st.borg.trait_[index as usize] = if index == BI_FOOD { 0 } else { 1 };
            assert!(!buy_identify_enabled(&st));
            st.borg.trait_[index as usize] = old;
        }
    }

    #[test]
    fn learning_list_is_small_and_excludes_reserve_items() {
        let mut st = state(0);
        st.item_val.sv_potion_cure_light = 20;
        st.item_val.sv_potion_cure_serious = 21;
        st.item_val.sv_potion_cure_critical = 22;
        st.item_val.sv_scroll_satisfy_hunger = 30;
        st.item_val.sv_scroll_detect_invis = 31;
        st.item_val.sv_scroll_monster_confusion = 32;
        st.item_val.sv_scroll_phase_door = 40;
        st.item_val.sv_scroll_word_of_recall = 41;
        let mut item = blank_item();
        item.tval = TV_POTION as u8;
        for (sval, expected) in [(20, true), (21, true), (22, false)] {
            item.sval = sval;
            assert_eq!(is_learning_kind(&st, &item), expected);
        }
        item.tval = TV_SCROLL as u8;
        for (sval, expected) in [(30, true), (31, true), (32, true), (40, false), (41, false)] {
            item.sval = sval;
            assert_eq!(is_learning_kind(&st, &item), expected);
        }
    }

    #[test]
    fn learning_candidate_rejects_an_aware_kind() {
        let mut st = state(0);
        st.item_val.sv_scroll_detect_invis = 31;
        st.item_val.sv_scroll_satisfy_hunger = 30;
        let mut item = blank_item();
        item.tval = TV_SCROLL as u8;
        item.sval = 31;
        assert!(is_learning_candidate_kind(&st, &item, false));
        assert!(!is_learning_candidate_kind(&st, &item, true));
        /* Another learning-list kind follows the same gate. */
        item.sval = 30;
        assert!(is_learning_candidate_kind(&st, &item, false));
        assert!(!is_learning_candidate_kind(&st, &item, true));
        /* Not on the learning list, aware or not. */
        item.sval = 99;
        assert!(!is_learning_candidate_kind(&st, &item, false));
        assert!(!is_learning_candidate_kind(&st, &item, true));
    }

    #[test]
    fn home_sell_retention_requires_an_actual_unmet_reserve_item() {
        let mut st = state(1);
        st.item_val.sv_scroll_phase_door = 10;
        let mut item = blank_item();
        item.tval = TV_SCROLL as u8;
        item.sval = 10;
        item.iqty = 1;
        assert!(retain_reserve410_item(&st, &item));
        st.borg.trait_[BI_APHASE as usize] = RESERVE410_PHASE;
        st.borg.trait_[BI_ACLW as usize] = RESERVE410_CURE;
        assert!(!retain_reserve410_item(&st, &item));
        st.borg.trait_[BI_APHASE as usize] = 0;
        st.borg.trait_[BI_ACLW as usize] = 0;
        item.sval = 11;
        assert!(!retain_reserve410_item(&st, &item));
        item.sval = 10;
        item.iqty = 0;
        assert!(!retain_reserve410_item(&st, &item));
        item.iqty = 1;
        st.core.cfg[BORG_BUY_STRATEGY_DEFAULT as usize] = 0;
        assert!(!retain_reserve410_item(&st, &item));
    }
}
