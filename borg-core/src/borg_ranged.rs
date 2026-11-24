//! Ranged supplies for the next trip and for the current dungeon level.
use std::fs::File;
use std::io::Write;
use std::sync::{Mutex, OnceLock};

use crate::borg_headers::borg_h::BORG_TOWN_NO_STORES;
use crate::borg_headers::borg_item_h::borg_item;
use crate::borg_headers::borg_magic_h::*;
use crate::borg_headers::borg_trait_h::*;
use crate::borg_item::{inven_wield, quiver_start};
use crate::borg_think_dungeon::GOAL_BORE;
use crate::borg_town_utility::Journey;
use crate::borg_trip::ranged_recovery;
use crate::headers::list_object_flags_h::{OF_EXPLODE, OF_NO_FUEL, OF_THROWING};
use crate::headers::list_options_h::{OPT_birth_force_descend, OPT_birth_no_recall};
use crate::headers::list_terrain_h::FEAT_LESS;
use crate::headers::list_tvals_h::*;
use crate::params::{Param, value};
use crate::{engine, state::BorgState};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub(crate) struct Coverage {
    pub missile: i32,
    pub thrown: i32,
    pub device: i32,
    pub spell_ready: bool,
    pub spell_recoverable: bool,
    pub activation_ready: i32,
    pub recharging: bool,
    pub can_aim: bool,
}

impl Coverage {
    fn finite(self) -> i32 {
        self.missile + self.thrown + self.device
    }

    pub fn recoverable(self) -> bool {
        self.spell_recoverable || self.recharging || self.activation_ready > 0
    }

    fn short(self, threshold: i32) -> bool {
        !self.recoverable() && self.finite() + self.activation_ready < threshold
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Need {
    pub target: i32,
    min_damage: i32,
    fuel_goal: i64,
}

impl Need {
    pub fn for_depth(st: &BorgState, depth: i64, journey: Journey) -> Self {
        Self {
            target: value(Param::ranged_readiness_target),
            min_damage: (depth as i32 / value(Param::ranged_readiness_damage_depth)).max(1),
            fuel_goal: crate::borg_light::fuel_goal(st, journey),
        }
    }

    pub fn backup_target(self, renewable: bool) -> i32 {
        if renewable {
            (self.target + 1) / 2
        } else {
            self.target
        }
    }

    fn fuel_units(self, st: &BorgState, oil: &borg_item) -> i32 {
        let light = &st.items.items[inven_wield() as usize + 5];
        if u32::from(light.tval) != TV_LIGHT
            || i32::from(light.sval) != st.item_val.sv_light_lantern
            || engine::flag_has(&light.flags, OF_NO_FUEL as i32)
            || oil.pval <= 0
        {
            return 0;
        }
        let short = (self.fuel_goal - i64::from(light.timeout)).max(0) as u64;
        short.div_ceil(oil.pval as u64) as i32
    }

    pub fn is_oil(st: &BorgState, item: &borg_item) -> bool {
        u32::from(item.tval) == TV_FLASK
            && i32::from(item.sval) == st.item_val.sv_flask_oil
            && !item.cursed
    }

    pub fn preceding_oil(st: &BorgState, end: usize) -> i32 {
        st.items
            .items
            .iter()
            .take(end)
            .filter(|i| i.aware && Self::is_oil(st, i))
            .map(|i| i32::from(i.iqty))
            .sum()
    }

    pub fn missile(self, st: &BorgState, item: &borg_item) -> bool {
        let power = st.borg.tr(BI_AMMO_POWER);
        let damage = (i32::from(item.dd) * (i32::from(item.ds) + 1)
            + 2 * (i32::from(item.to_d) + st.borg.tr(BI_BTODAM)))
            * power;
        i32::from(item.tval) == st.borg.tr(BI_AMMO_TVAL)
            && power > 0
            && damage >= 2 * self.min_damage
    }

    fn thrown(self, item: &borg_item) -> bool {
        let throwing = engine::flag_has(&item.flags, OF_THROWING as i32);
        let explode = engine::flag_has(&item.flags, OF_EXPLODE as i32);
        crate::borg_fight_attack::thrown_damage(
            i32::from(item.dd),
            i32::from(item.ds),
            i32::from(item.to_d),
            i32::from(item.weight),
            throwing,
            explode,
        ) >= self.min_damage
    }

    pub fn floor_item(self, st: &BorgState, index: u32) -> bool {
        let kind = engine::kind(index as usize);
        if !matches!(kind.tval as u32, TV_FLASK | TV_ARROW | TV_SHOT | TV_BOLT) {
            return false;
        }
        let mut item = crate::borg_item::blank_item();
        item.kind = index;
        item.iqty = 1;
        item.aware = true;
        item.tval = kind.tval as u8;
        item.sval = kind.sval as u8;
        item.dd = kind.dd as u8;
        item.ds = kind.ds as u8;
        item.weight = kind.weight as i16;
        item.flags = kind.flags;
        item.pval = engine::kind_value_average(kind.pval) as i16;
        let previous_oil = if Self::is_oil(st, &item) {
            Self::preceding_oil(st, inven_wield() as usize)
        } else {
            0
        };
        self.item(st, &item, previous_oil, true) > 0
    }

    pub fn item(self, st: &BorgState, item: &borg_item, oil_before: i32, offer: bool) -> i32 {
        if item.iqty == 0 || item.cursed || !item.aware || item.kind == 0 {
            return 0;
        }
        let copies = if offer { 1 } else { i32::from(item.iqty) };
        match u32::from(item.tval) {
            TV_FLASK if Self::is_oil(st, item) && self.thrown(item) => {
                copies - (self.fuel_units(st, item) - oil_before).clamp(0, copies)
            }
            TV_ARROW | TV_BOLT | TV_SHOT if self.missile(st, item) || self.thrown(item) => copies,
            TV_WAND | TV_STAFF | TV_ROD if self.attack_device(st, item) => {
                if offer && u32::from(item.tval) == TV_ROD {
                    1
                } else if offer {
                    i32::from(item.pval).max(0) / i32::from(item.iqty)
                } else {
                    i32::from(item.pval).max(0)
                }
            }
            _ => 0,
        }
    }

    fn attack_device(self, st: &BorgState, item: &borg_item) -> bool {
        crate::borg_item_use::ranged_device_reliable(st, item)
            && self.effect_damage(
                st,
                &engine::kind_effect_values(item.kind as usize, st.borg.tr(BI_CLEVEL)),
            )
    }

    fn effect_damage(self, st: &BorgState, effects: &[(u16, i32, i64)]) -> bool {
        crate::borg_town_capability::ranged_effect_damage(
            effects,
            i64::from(st.borg.tr(BI_MAXHP)).max(1),
        ) >= i64::from(self.min_damage)
    }

    pub fn coverage(self, st: &BorgState, skip: Option<usize>) -> Coverage {
        let can_aim = [BI_ISBLIND, BI_ISCONFUSED, BI_ISIMAGE]
            .iter()
            .all(|&trait_| st.borg.tr(trait_) == 0);
        let mut total = Coverage {
            can_aim,
            ..Coverage::default()
        };
        let pack = inven_wield() as usize;
        let quiver = quiver_start() as usize;
        let mut previous_oil = 0;
        for (slot, item) in st.items.items.iter().enumerate() {
            if Some(slot) == skip || item.iqty == 0 || item.cursed || !item.aware || item.kind == 0
            {
                continue;
            }
            if (pack..quiver).contains(&slot) {
                if item.activ_idx > 0
                    && self.effect_damage(
                        st,
                        &engine::activation_effect_values(
                            item.activ_idx,
                            item.kind as usize,
                            st.borg.tr(BI_CLEVEL),
                        ),
                    )
                {
                    total.recharging |= item.timeout > 0;
                    total.activation_ready += i32::from(item.timeout == 0);
                }
                continue;
            }
            let n = self.item(st, item, previous_oil, false);
            match u32::from(item.tval) {
                TV_FLASK if Self::is_oil(st, item) => {
                    previous_oil += i32::from(item.iqty);
                    total.thrown += n;
                }
                TV_ARROW | TV_BOLT | TV_SHOT => {
                    if slot >= quiver && self.missile(st, item) {
                        total.missile += n;
                    } else if slot < pack && self.thrown(item) {
                        total.thrown += n;
                    }
                }
                TV_ROD => {
                    total.device += n;
                    total.recharging |= self.attack_device(st, item);
                }
                TV_WAND | TV_STAFF => total.device += n,
                _ => {}
            }
        }
        let fail = value(Param::attack_spell_fail);
        for spell in [
            MAGIC_MISSILE,
            FROST_BOLT,
            STINKING_CLOUD,
            FIRE_BALL,
            ICE_STORM,
            METEOR_SWARM,
            RIFT,
            MANA_STORM,
            SHOCK_WAVE,
            EXPLOSION,
            ORB_OF_DRAINING,
            ANNIHILATE,
            ELECTRIC_ARC,
            ACID_SPRAY,
            MANA_BOLT,
            THRUST_AWAY,
            LIGHTNING_STRIKE,
            EARTH_RISING,
            VOLCANIC_ERUPTION,
            RIVER_OF_LIGHTNING,
            SPEAR_OF_OROME,
            LIGHT_OF_VARDA,
            NETHER_BOLT,
            DISENCHANT,
        ] {
            use crate::borg_magic::{
                borg_get_spell_number, borg_spell_legal_fail, borg_spell_okay_fail,
            };
            if !borg_spell_legal_fail(st, spell, fail) {
                continue;
            }
            let index = borg_get_spell_number(st, spell);
            if self.effect_damage(
                st,
                &engine::spell_effect_values(index, st.borg.tr(BI_CLEVEL)),
            ) {
                total.spell_recoverable = true;
                total.spell_ready |= can_aim && borg_spell_okay_fail(st, spell, fail);
            }
        }
        total
    }
}

#[derive(Default)]
pub(crate) struct Trip {
    in_town: bool,
    departure_short: bool,
    departure_gold: i32,
    departure_ammo: i32,
    offers: Vec<(i32, borg_item)>,
    coverage: Coverage,
    last_record: Option<(i32, Coverage)>,
}

fn here(st: &BorgState) -> Need {
    let journey = Journey::of(st);
    Need::for_depth(st, i64::from(st.borg.tr(BI_CDEPTH)).max(1), journey)
}

pub(crate) fn pickup_need(st: &BorgState) -> Option<Need> {
    if !ranged_recovery() {
        return None;
    }
    let need = here(st);
    let current = st.ranged.coverage;
    (current.finite() < need.backup_target(current.recoverable())).then_some(need)
}

pub(crate) fn keep_stack(st: &BorgState, slot: usize) -> bool {
    if !ranged_recovery() {
        return false;
    }
    let need = here(st);
    let item = &st.items.items[slot];
    let oil = if Need::is_oil(st, item) {
        Need::preceding_oil(st, slot)
    } else {
        0
    };
    if need.item(st, item, oil, false) <= 0
        && !(u32::from(item.tval) == TV_ROD
            && item.aware
            && item.iqty > 0
            && !item.cursed
            && item.kind != 0
            && need.attack_device(st, item))
    {
        return false;
    }
    let before = need.coverage(st, None);
    let after = need.coverage(st, Some(slot));
    (before.finite() > after.finite() || before.recoverable() && !after.recoverable())
        && after.finite() < need.backup_target(after.recoverable())
}

/// Leave an ordinary swap to the danger check if it makes the grid safer.
/// Otherwise hold a working ranged kit until the new equipment can replace it.
pub(crate) fn worsens(before: Coverage, after: Coverage) -> bool {
    !before.short(1) && after.short(1)
}

pub(crate) fn wear_coverage(st: &BorgState, slot: usize, new: &borg_item) -> Option<Coverage> {
    if !ranged_recovery() {
        return None;
    }
    let wield = inven_wield() as usize;
    let old = &st.items.items[slot];
    (slot == wield + 1 || slot == wield + 5 || old.activ_idx > 0 || new.activ_idx > 0)
        .then(|| here(st).coverage(st, None))
}

pub(crate) fn coverage_here(st: &BorgState) -> Coverage {
    here(st).coverage(st, None)
}

fn record_file() -> Option<&'static Mutex<File>> {
    static FILE: OnceLock<Option<Mutex<File>>> = OnceLock::new();
    FILE.get_or_init(|| {
        let path = std::env::var_os("BORG_RANGED_RECORD")
            .map(std::path::PathBuf::from)
            .or_else(|| {
                if value(Param::ranged_readiness_trace) == 0 {
                    return None;
                }
                std::env::var_os("BORG_RUN_DIR")
                    .map(|dir| std::path::PathBuf::from(dir).join("ranged-readiness.jsonl"))
            })?;
        match File::create(path) {
            Ok(file) => Some(Mutex::new(file)),
            Err(error) => {
                eprintln!("BORG_RANGED_RECORD: {error}");
                None
            }
        }
    })
    .as_ref()
}

pub(crate) fn observe(st: &mut BorgState) {
    let record = record_file();
    if !ranged_recovery() && record.is_none() {
        return;
    }
    let depth = st.borg.tr(BI_CDEPTH);
    let journey = Journey::of(st);
    let needs = Need::for_depth(
        st,
        if depth == 0 {
            journey.depth
        } else {
            i64::from(depth)
        },
        journey,
    );
    let coverage = needs.coverage(st, None);
    if let Some(file) = record.filter(|_| st.ranged.last_record != Some((depth, coverage))) {
        let row = serde_json::json!({
            "schema": "borg-ranged-readiness@1", "game_turn": engine::game_turn(),
            "depth": depth, "target": needs.target, "min_damage": needs.min_damage,
            "coverage": coverage, "ready": coverage.can_aim &&
                (coverage.finite() + coverage.activation_ready > 0 || coverage.spell_ready),
        });
        let _ = writeln!(file.lock().unwrap_or_else(|e| e.into_inner()), "{row}");
        st.ranged.last_record = Some((depth, coverage));
    }
    st.ranged.coverage = coverage;
    if depth != 0 {
        st.ranged.in_town = false;
        return;
    }
    let gold = st.borg.tr(BI_GOLD);
    let refresh = !st.ranged.in_town || st.ranged.departure_gold != gold;
    st.ranged.departure_short = coverage.short(needs.backup_target(true));
    st.ranged.departure_gold = gold;
    st.ranged.departure_ammo = st.borg.tr(BI_AMMO_TVAL);
    if !ranged_recovery() || !refresh || st.cfg(BORG_TOWN_NO_STORES) != 0 {
        return;
    }
    st.ranged.in_town = true;
    st.ranged.offers.clear();
    for (shop, stock) in st.store.shops.iter().enumerate() {
        // Ordinary stores and known Home stock. The Black Market's gold
        // reserve greatly reduces the part of its stock we can buy.
        if shop == crate::borg_town_stock::BORG_BLACK_MARKET {
            continue;
        }
        let native = engine::store_stock(&st.access, shop);
        for (slot, item) in stock.ware.iter().enumerate().filter(|(_, i)| {
            i.iqty > 0
                && i.aware
                && !i.cursed
                && matches!(
                    u32::from(i.tval),
                    TV_FLASK | TV_SHOT | TV_ARROW | TV_BOLT | TV_WAND | TV_ROD | TV_STAFF
                )
        }) {
            let Some(offer) = native.get(slot) else {
                continue;
            };
            let price = if shop == crate::borg_store::BORG_HOME {
                0
            } else {
                offer.purchase_quote(shop)
            };
            st.ranged.offers.push((price, *item));
        }
    }
}

fn restock_cost(st: &BorgState, need: Need, missing: i32) -> Option<i32> {
    let oil = Need::preceding_oil(st, inven_wield() as usize);
    st.ranged
        .offers
        .iter()
        .filter_map(|(price, item)| {
            let copies = if Need::is_oil(st, item) && need.thrown(item) {
                (need.fuel_units(st, item) - oil).max(0) + missing
            } else {
                let uses = need.item(st, item, oil, true);
                if uses <= 0 {
                    return None;
                }
                (missing + uses - 1) / uses
            };
            (copies <= i32::from(item.iqty)).then_some(price.saturating_mul(copies))
        })
        .min()
}

pub(crate) fn return_to_town(st: &mut BorgState) -> bool {
    if !ranged_recovery()
        || st.cfg(BORG_TOWN_NO_STORES) != 0
        || !crate::borg_trip::acting(st)
        || st.borg.tr(BI_CDEPTH) == 0
        || st.borg.goal.fleeing
        || st.borg.mon.scary
        || st.borg.near.unique != 0
        || st.borg.goal.recalling != 0
        || engine::player_opt(OPT_birth_force_descend)
    {
        return false;
    }
    let need = here(st);
    let threshold = need.backup_target(true);
    if !st.ranged.coverage.short(threshold) {
        return false;
    }
    let gold = st.borg.tr(BI_GOLD);
    let ammo = st.borg.tr(BI_AMMO_TVAL);
    let Some(cost) = restock_cost(st, need, threshold - st.ranged.coverage.finite()) else {
        return false;
    };
    if gold < cost
        || st.ranged.departure_short
            && ammo == st.ranged.departure_ammo
            && gold - st.ranged.departure_gold < cost.max(1)
    {
        return false;
    }

    let depth = st.borg.tr(BI_CDEPTH);
    let up = st.borg.c;
    let on_stair = u32::from(st.cave.grid(up.x, up.y).feat) == FEAT_LESS;
    let recall = depth >= crate::borg_trip::RECALL_MIN_DEPTH
        && !engine::player_opt(OPT_birth_no_recall)
        && st.borg.tr(BI_RECALL) > 0
        && crate::borg_escape::borg_recall(st);
    if recall
        || on_stair
        || st.stairs.less.num > 0
            && crate::borg_flow_stairs::borg_flow_stair_less(st, GOAL_BORE, true)
    {
        st.borg.goal.rising = true;
        st.borg.stair_more = false;
        st.borg.stair_less = !recall;
        if on_stair && !recall && !crate::borg_flow_stairs::borg_prep_leave_level_spells(st) {
            crate::borg_io::borg_keypress(st, b'<' as u32);
        }
        crate::borg_io::borg_note(
            st,
            if recall {
                "# Ranged readiness: restock via Recall."
            } else {
                "# Ranged readiness: restock via up stairs."
            },
        );
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_or_exhausted_attack_starts_ranged_recovery() {
        assert!(Coverage::default().short(3));
        assert!(
            Coverage {
                missile: 2,
                ..Coverage::default()
            }
            .short(3)
        );
        assert!(
            !Coverage {
                missile: 2,
                thrown: 1,
                ..Coverage::default()
            }
            .short(3)
        );
    }

    #[test]
    fn resting_restores_a_spell_or_rod_without_a_town_trip() {
        assert!(
            !Coverage {
                spell_recoverable: true,
                ..Coverage::default()
            }
            .short(3)
        );
        assert!(
            !Coverage {
                recharging: true,
                ..Coverage::default()
            }
            .short(3)
        );
    }

    #[test]
    fn an_ordinary_swap_can_trade_sources_but_cannot_lose_the_only_source() {
        let launcher = Coverage {
            missile: 5,
            ..Coverage::default()
        };
        let device = Coverage {
            device: 1,
            ..Coverage::default()
        };
        assert!(!worsens(launcher, device));
        assert!(worsens(launcher, Coverage::default()));
    }
}
