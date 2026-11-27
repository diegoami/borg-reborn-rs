//! Recall-first equipment exchanges for the weighted town planner (#1239).
use crate::borg_headers::borg_trait_h::*;
use crate::borg_inventory::borg_wield_slot;
use crate::borg_io::{borg_keypress, borg_note};
use crate::borg_item::{inven_wield, BorgItem};
use crate::borg_store::BORG_HOME;
use crate::borg_town_budget::Facts;
use crate::borg_town_record::action;
use crate::borg_town_shadow::TownAction;
use crate::borg_town_utility::Room;
use crate::engine;
use crate::headers::list_object_flags_h::OF_STICKY;
use crate::headers::list_tvals_h::*;
use crate::params::{value as param, Param};
use crate::state::BorgState;

// Live executor only. The recorded shopping solve remains independently replayable.
pub(crate) const WEAR: i64 = -3;

#[derive(Clone, Copy)]
enum Source {
    Pack,
    Shop(usize),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Stage {
    Acquire,
    Wear,
    Store,
}

struct Swap {
    source: Source,
    slot: usize,
    replacement: BorgItem,
    displaced: BorgItem,
    stage: Stage,
    queued: bool,
    home_copies: u32,
}

#[derive(Default)]
pub(crate) struct Loadout {
    pending: Option<Swap>,
    attempted: bool,
    retained: Vec<BorgItem>,
}

impl Loadout {
    pub(crate) fn new_visit(&mut self) {
        self.pending = None;
        self.attempted = false;
    }
}

pub(crate) fn enabled(st: &BorgState) -> bool {
    crate::borg_town_live::active(st) && !crate::borg_town_live::shop_overloaded()
        && !crate::borg_town_live::unload_on_arrival(st)
        && param(Param::town_loadout_power_loss) > 0
}

pub(crate) fn pending(st: &BorgState) -> bool {
    enabled(st) && st.think.town_live.recall_loadout.pending.is_some()
}

pub(crate) fn retained_at_home(st: &BorgState, item: &BorgItem) -> bool {
    enabled(st) && st.think.town_live.recall_loadout.retained.iter().any(|saved| same_gear(saved, item))
}

// Stock numbers, article/prefix, inscriptions and the display value can
// change when a piece moves. Match its known equipment properties instead.
fn same_gear(a: &BorgItem, b: &BorgItem) -> bool {
    b.iqty != 0 && a.kind == b.kind && a.weight == b.weight
        && a.art_idx == b.art_idx && a.ego_idx == b.ego_idx && a.activ_idx == b.activ_idx
        && [a.to_h, a.to_d, a.to_a, a.ac, a.pval] == [b.to_h, b.to_d, b.to_a, b.ac, b.pval]
        && (a.dd, a.ds) == (b.dd, b.ds) && a.flags == b.flags && a.modifiers == b.modifiers
        && a.brands == b.brands && a.slays == b.slays
        && a.el_info.iter().zip(&b.el_info).all(|(a, b)| a.res_level == b.res_level)
}

fn pack(st: &BorgState) -> &[BorgItem] {
    &st.items.items[..inven_wield() as usize]
}

fn home(st: &BorgState) -> &[BorgItem] {
    &st.store.shops[BORG_HOME].ware
}

fn home_copies(st: &BorgState, item: &BorgItem) -> u32 {
    home(st).iter().filter(|ware| same_gear(item, ware)).map(|ware| u32::from(ware.iqty)).sum()
}

fn eligible(item: &BorgItem) -> bool {
    item.iqty > 0 && item.ident && !item.cursed
        && !engine::flag_has(&item.flags, OF_STICKY as i32)
        && !item.desc.windows(2).any(|s| s[0] as u8 == b'!' && matches!(s[1] as u8, b'*' | b'w' | b't' | b'd'))
        && matches!(u32::from(item.tval), TV_HAFTED | TV_SWORD | TV_POLEARM
            | TV_DRAG_ARMOR | TV_HARD_ARMOR | TV_SOFT_ARMOR | TV_SHIELD | TV_CLOAK
            | TV_CROWN | TV_HELM | TV_GLOVES | TV_BOOTS)
}

#[derive(Clone, Copy)]
#[cfg_attr(feature = "test-town-loadout", derive(Debug))]
struct Assessment {
    room: i64,
    speed: i32,
    power: i32,
    armor: i32,
    damage: i32,
    hit: i32,
    mana_penalty: i32,
}

impl Assessment {
    fn of(st: &BorgState) -> Self {
        let tr = |i| st.borg.tr(i);
        let wield = inven_wield() as usize;
        // engine/angband/src/player-calcs.c:calc_mana():1505-1532.
        // The Borg's observed max SP is live; compare projected armor penalties
        // together with the spell-stat adjustment below.
        let armor_weight: i32 = st.items.items[wield + 6..wield + 12].iter()
            .filter(|item| item.iqty > 0).map(|item| i32::from(item.weight)).sum();
        let mana_penalty = if engine::class_total_spells() > 0 {
            ((armor_weight - engine::class_spell_weight()) / 10).max(0)
        } else { 0 };
        Self {
            room: i64::from(tr(BI_CARRY) / 2) - i64::from(tr(BI_WEIGHT)),
            speed: tr(BI_SPEED),
            power: crate::borg_power::borg_power_equipment(st),
            armor: tr(BI_ARMOR),
            damage: (tr(BI_WDD) * (tr(BI_WDS) + 1) + 2 * (tr(BI_TODAM) + tr(BI_WTODAM))).max(0) * tr(BI_BLOWS),
            hit: tr(BI_TOHIT) + tr(BI_WTOHIT),
            mana_penalty,
        }
    }

    fn fits(self, before: Self, required: Room, free_slots: i32, restore: bool, power_loss: i32) -> bool {
        self.room >= required.weight && free_slots >= required.slots
            && self.speed >= before.speed && self.mana_penalty <= before.mana_penalty
            && 2 * i64::from(self.armor) >= i64::from(before.armor)
            && 2 * i64::from(self.damage) >= i64::from(before.damage)
            && (before.hit <= 0 || 2 * i64::from(self.hit) >= i64::from(before.hit))
            && if restore { self.power > before.power }
                else { self.room > before.room && i64::from(self.power) + i64::from(power_loss) >= i64::from(before.power) }
    }
}

fn preserves_traits(st: &BorgState, original: &[i32]) -> bool {
    // Keep the currently worn protection, a usable light and launcher, and
    // the stat/failure adjustments of the spells the unchanged pack carries.
    (BI_IFIRE..=BI_FRACT).chain([BI_SINV, BI_LIGHT, BI_SHOTS, BI_AMMO_POWER, BI_SP_ADJ, BI_FAIL1, BI_HP_ADJ])
        .all(|i| st.borg.tr(i) >= original[i as usize])
        && st.borg.tr(BI_FAIL2) <= original[BI_FAIL2 as usize]
}

/// Compare the actual post-deposit kit. The notice calculation uses the
/// replacement's STR and speed modifiers and the encumbrance from this kit
/// (engine/angband/src/player-calcs.c:weight_limit():1741-1751, calc_bonuses():2222-2229).
fn assess(st: &mut BorgState, slot: usize, replacement: BorgItem, pack_slot: Option<usize>) -> Option<Assessment> {
    let borg = st.borg.clone();
    let traits = st.traits.clone();
    let home = st.home.clone();
    let do_spell = st.think.do_spell;
    st.think.do_spell = false;
    let displaced = std::mem::replace(&mut st.items.items[slot], BorgItem { iqty: 1, ..replacement });
    if let Some(i) = pack_slot { st.items.items[i].iqty = 0; }
    crate::borg_trait::borg_notice(st, false);
    let result = preserves_traits(st, &borg.trait_).then(|| Assessment::of(st));
    st.items.items[slot] = displaced;
    if let Some(i) = pack_slot { st.items.items[i] = replacement; }
    st.borg = borg;
    st.traits = traits;
    st.home = home;
    st.think.do_spell = do_spell;
    result
}

pub(crate) fn start(st: &mut BorgState, urgent: Room, gold: i64) -> Option<TownAction> {
    if !enabled(st) || st.think.town_live.recall_loadout.attempted || engine::player_is_shapechanged()
        || st.borg.tr(BI_ISHUNGRY) != 0 || st.borg.tr(BI_ISWEAK) != 0
    {
        return None;
    }
    let before = Assessment::of(st);
    let departure = crate::borg_town_utility::departure_room();
    let restore = before.room >= urgent.weight;
    let desired = departure.weight + urgent.weight;
    if restore && (before.room < desired || st.think.town_live.recall_loadout.retained.is_empty()) {
        return None;
    }

    let facts = Facts::of(st);
    let free_slots = facts.pack_capacity - crate::borg_town_stock::pack_slots_used_of(pack(st));
    let wield = inven_wield() as usize;
    let mut offers = Vec::new();
    if !restore {
        offers.extend(pack(st).iter().copied().enumerate().map(|(i, item)| (Source::Pack, i, item)));
    }
    // A Black Market purchase has its own live gold constraints.
    for shop in (0..crate::borg_town_stock::BORG_BLACK_MARKET).chain([BORG_HOME]) {
        if restore && shop != BORG_HOME { continue; }
        offers.extend(st.store.shops[shop].ware.iter().copied().enumerate()
            .map(|(i, item)| (Source::Shop(shop), i, item)));
    }

    let mut best = None;
    for (source, index, item) in offers {
        if !eligible(&item) || restore && !retained_at_home(st, &item) { continue; }
        let slot = borg_wield_slot(&item) as usize;
        let old = st.items.items[slot];
        if !eligible(&old) || old.iqty != 1 || same_gear(&old, &item)
            || if restore { item.weight <= old.weight } else { item.weight >= old.weight }
            || old.activ_idx > 0 && old.activ_idx != item.activ_idx
        {
            continue;
        }
        let (pack_slot, price, opens_home) = match source {
            Source::Pack if item.iqty == 1 => (Some(index), 0, false),
            Source::Pack => continue,
            Source::Shop(shop) => {
                if free_slots <= 0 || pack(st).iter().any(|it| same_gear(&item, it))
                    || crate::borg_town_live::buy_rejected(st, shop as i16, index)
                    || crate::borg_town_live::acquisition_room(st, item.tval, item.kind) == 0
                {
                    continue;
                }
                let stock = engine::store_stock(&st.access, shop);
                let Some(offer) = stock.get(index).filter(|o| o.carry_capacity() > 0 && o.merge_capacity() == 0) else { continue; };
                let price = if shop == BORG_HOME { 0 } else { offer.purchase_quote(shop) };
                if i64::from(price) > gold { continue; }
                (None, price, shop == BORG_HOME && item.iqty == 1)
            }
        };
        if crate::borg_town_live::sale_rejected(st, BORG_HOME as i16, &old)
            || (!opens_home && !engine::equip_object(&st.access, slot - wield)
                .is_some_and(|it| it.store_accepts_quantity(BORG_HOME, 1)))
        {
            continue;
        }
        let slots_after = free_slots + i32::from(pack_slot.is_some());
        let Some(after) = assess(st, slot, item, pack_slot) else { continue; };
        let minimum = if restore { Room { weight: desired, slots: departure.slots + urgent.slots } } else { urgent };
        if !after.fits(before, minimum, slots_after, restore, param(Param::town_loadout_power_loss)) { continue; }
        let leaves_reserve = after.room >= desired && slots_after >= departure.slots + urgent.slots;
        let score = (after.speed, leaves_reserve, after.power, std::cmp::Reverse(price), after.room);
        if best.as_ref().is_none_or(|(key, _)| score > *key) {
            let stage = if pack_slot.is_some() { Stage::Wear } else { Stage::Acquire };
            best = Some((score, Swap { source, slot, replacement: item, displaced: old, stage, queued: false, home_copies: 0 }));
        }
    }
    let ((_, _, power, _, _), swap) = best?;
    let delta = i64::from(power) - i64::from(before.power);
    borg_note(st, &format!("# Town loadout: projected equipment power {} -> {power} ({delta:+})", before.power));
    crate::borg_item_wear::borg_clear_best(st);
    st.think.town_live.recall_loadout.attempted = true;
    st.think.town_live.recall_loadout.pending = Some(swap);
    advance(st)
}

fn shop_action(code: i64, shop: usize, slot: usize) -> TownAction {
    TownAction { code, at: [shop as i64, slot as i64, 1, 0] }
}

/// Use fresh pack/Home indices and advance only after the intended gear move
/// is observed. A rejected or lost item ends the attempt for this visit.
pub(crate) fn advance(st: &mut BorgState) -> Option<TownAction> {
    let mut swap = st.think.town_live.recall_loadout.pending.take()?;
    if swap.queued {
        swap.queued = false;
        swap.stage = match swap.stage {
            Stage::Acquire if pack(st).iter().any(|it| same_gear(&swap.replacement, it)) => {
                if matches!(swap.source, Source::Shop(BORG_HOME)) && home_copies(st, &swap.replacement) == 0 {
                    st.think.town_live.recall_loadout.retained.retain(|it| !same_gear(it, &swap.replacement));
                }
                Stage::Wear
            }
            Stage::Wear if same_gear(&swap.replacement, &st.items.items[swap.slot]) => Stage::Store,
            Stage::Store if home_copies(st, &swap.displaced) > swap.home_copies => {
                let saved = &mut st.think.town_live.recall_loadout.retained;
                if swap.displaced.weight > swap.replacement.weight
                    && !saved.iter().any(|it| same_gear(it, &swap.displaced))
                {
                    saved.push(swap.displaced);
                }
                borg_note(st, "# Town loadout: confirmed displaced gear at Home");
                return None;
            }
            _ => {
                borg_note(st, "# Town loadout: gear move failed to make progress");
                return None;
            }
        };
    }
    let act = match swap.stage {
        Stage::Acquire => {
            let Source::Shop(shop) = swap.source else { return None; };
            let slot = st.store.shops[shop].ware.iter().position(|it| same_gear(&swap.replacement, it))?;
            shop_action(if shop == BORG_HOME { action::TAKE_HOME } else { action::BUY }, shop, slot)
        }
        Stage::Wear => {
            let slot = pack(st).iter().position(|it| it.iqty == 1 && same_gear(&swap.replacement, it))?;
            let wield = inven_wield() as usize;
            if !same_gear(&swap.displaced, &st.items.items[swap.slot])
                || !engine::equip_object(&st.access, swap.slot - wield)
                    .is_some_and(|it| it.store_accepts_quantity(BORG_HOME, 1))
            {
                return None;
            }
            let shop = if st.borg.in_shop { i64::from(st.think.shop_num) } else { -1 };
            TownAction { code: WEAR, at: [shop, slot as i64, 1, 0] }
        }
        Stage::Store => {
            let slot = pack(st).iter().position(|it| same_gear(&swap.displaced, it))?;
            if crate::borg_town_live::sale_rejected(st, BORG_HOME as i16, &st.items.items[slot])
                || !engine::inven_object(&st.access, slot).is_some_and(|it| it.store_accepts_quantity(BORG_HOME, 1))
            {
                return None;
            }
            swap.home_copies = home_copies(st, &swap.displaced);
            shop_action(action::STORE, BORG_HOME, slot)
        }
    };
    st.think.town_live.recall_loadout.pending = Some(swap);
    Some(act)
}

pub(crate) fn observe_command(st: &mut BorgState, executed: bool) {
    if executed {
        if let Some(swap) = &mut st.think.town_live.recall_loadout.pending { swap.queued = true; }
    } else {
        st.think.town_live.recall_loadout.pending = None;
    }
}

pub(crate) fn wear(st: &mut BorgState, slot: i32) {
    // engine/angband/src/cmd-obj.c:do_cmd_wield():270-332 and
    // src/ui-store.c:store_process_command():825-862 accept the same keys.
    borg_note(st, "# Town loadout: wearing replacement");
    borg_keypress(st, u32::from(b'w'));
    borg_keypress(st, u32::from(engine::all_letters_nohjkl(slot)));
    observe_command(st, true);
}

#[cfg(feature = "test-town-loadout")]
pub(crate) fn fixture(st: &mut BorgState, case: &str) {
    use std::sync::atomic::{AtomicU8, Ordering::Relaxed};
    static SETUP: AtomicU8 = AtomicU8::new(0);
    // Native messaging may ask for keys while the synthetic kit is installed.
    if SETUP.load(Relaxed) == 1 { return; }
    let excess = match case {
        "full-home" | "full-pack" | "no-storage" | "restore" | "deposit-failed" => 104,
        "trim" => 188,
        _ => panic!("unknown town loadout fixture {case}"),
    };
    assert!(enabled(st));
    if st.borg.tr(BI_CDEPTH) != 0 {
        eprintln!("left town before resupplying in {case}");
        std::process::exit(1);
    }
    if SETUP.load(Relaxed) == 0 {
        SETUP.store(1, Relaxed);
        let has_whip = case != "no-storage";
        let gold = if has_whip { 24_223 } else { 0 };
        engine::setup_test_town_loadout(st, excess, has_whip, case == "full-pack", gold, false, true, false, false);
        crate::borg_item::borg_find_all_items(st);
        crate::borg_trait::borg_notice(st, true);
        SETUP.store(2, Relaxed);
        assert_eq!(st.borg.tr(BI_CLEVEL), 20);
        assert_eq!(Facts::of(st).allowance(), -i64::from(excess));
        assert_eq!(st.borg.tr(BI_AMISSILES), 57);
        if case == "trim" {
            let reserve = param(Param::town_quiver_trim_reserve);
            let trim = crate::borg_junk::town_quiver_surplus(st, reserve, 200);
            let quiver = || st.items.items[crate::borg_item::quiver_start() as usize..].iter()
                .filter(|it| it.iqty > 0).map(|it| {
                    let desc: Vec<_> = it.desc.iter().take_while(|&&b| b != 0).map(|&b| b as u8).collect();
                    (it.iqty, it.tval, it.ident, it.cursed, it.to_h, it.to_d, String::from_utf8_lossy(&desc).into_owned())
                }).collect::<Vec<_>>();
            assert_eq!(trim.map(|slot| st.items.items[slot].iqty), Some(37), "reserve {reserve}: {:?}", quiver());
        }
        let free = Facts::of(st).pack_capacity - crate::borg_town_stock::pack_slots_used_of(pack(st));
        assert_eq!(free == 0, case == "full-pack");
        if case == "no-storage" {
            crate::borg_town_live::choose(st);
            assert!(!pending(st), "no legal Home deposit exists");
            assert_eq!(st.items.items[inven_wield() as usize].weight, 180);
            assert_eq!(st.borg.tr(BI_RECALL), 0);
            println!("#1239 town loadout passed: {case}");
            std::process::exit(0);
        }
    }
    if case == "deposit-failed" && st.think.town_live.recall_loadout.pending.as_ref().is_some_and(|swap|
        swap.queued && swap.stage == Stage::Wear && same_gear(&swap.replacement, &st.items.items[swap.slot]))
    {
        let act = advance(st).expect("wearing the replacement should lead to a deposit");
        assert_eq!(act.code, action::STORE);
        let displaced = pack(st)[act.at[1] as usize];
        assert_eq!(displaced.weight, 180);
        // Simulate a rejected store command by leaving the observed pack and
        // Home stock unchanged after the command was queued.
        observe_command(st, true);
        assert!(advance(st).is_none());
        assert!(!pending(st));
        assert_eq!(home_copies(st, &displaced), 0);
        assert!(pack(st).iter().any(|it| same_gear(&displaced, it)));
        assert!(st.think.town_live.recall_loadout.retained.is_empty());
        println!("#1239 town loadout passed: {case}");
        std::process::exit(0);
    }
    if SETUP.load(Relaxed) == 3 {
        if st.items.items[inven_wield() as usize].weight == 180
            && !pending(st) && home(st).iter().any(|it| it.tval == TV_HAFTED as u8 && it.weight == 30)
        {
            assert!(Facts::of(st).allowance() >= crate::borg_town_utility::departure_room().weight);
            println!("#1239 town loadout passed: {case}");
            std::process::exit(0);
        }
        return;
    }
    if st.borg.tr(BI_RECALL) > 0 {
        assert_eq!(st.items.items[inven_wield() as usize].weight, 30);
        assert!(home(st).iter().any(|it| it.weight == 180 && retained_at_home(st, it)));
        assert!(Facts::of(st).allowance() >= 0);
        assert_eq!(st.borg.tr(BI_AMISSILES), if case == "trim" { 20 } else { 57 });
        if case == "restore" {
            SETUP.store(1, Relaxed);
            engine::strengthen_test_town_loadout(st);
            // The loadout state is scoped by the observed visit clock.
            st.borg.time.level -= 1;
            crate::borg_trait::borg_notice(st, true);
            let before = Assessment::of(st);
            let mace = *home(st).iter().find(|it| it.iqty > 0 && it.weight == 180).unwrap();
            let after = assess(st, inven_wield() as usize, mace, None).expect("the mace preserves critical traits");
            let reserve = crate::borg_town_utility::departure_room();
            assert!(after.fits(before, reserve, 10, true, 0), "{before:?} -> {after:?}");
            SETUP.store(3, Relaxed);
            return;
        }
        println!("#1239 town loadout passed: {case}");
        std::process::exit(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requires_at_least_half_of_odd_combat_values() {
        let before = Assessment { room: -1, speed: 110, power: 0, armor: 3, damage: 3, hit: 1, mana_penalty: 0 };
        let viable = Assessment { room: 0, armor: 2, damage: 2, hit: 1, ..before };
        let fits = |kit: Assessment| kit.fits(before, Room::default(), 0, false, 1);
        assert!(fits(viable));
        assert!(!fits(Assessment { armor: 1, ..viable }));
        assert!(!fits(Assessment { damage: 1, ..viable }));
        assert!(!fits(Assessment { hit: 0, ..viable }));
    }
}
