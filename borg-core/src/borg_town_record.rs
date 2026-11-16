//! Phase 4 of the town inventory redesign (borg-rs#36), slice 1: the plan
//! record (P13, integer version 4) and its replay.
//!
//! The shape is the one accepted on 2026-11-13 in
//! `docs/TOWN-INVENTORY-PHASE4.md`, Part B, with the dated correction
//! recorded there: version 2's fields unchanged and in order, then the
//! plan's. One record per plan step - one solve, and the action taken from
//! it.
//!
//! **Replay** is the record's contract: from a v4 line alone, rebuild the
//! candidates, the limits, the urgent needs and the values, re-run the step, and
//! get the same plan, the same solve and the same runner-up. The facts a
//! record cannot recompute without the engine travel in it: the try-on
//! values (`tryon`) and, per ware, the facts the engine's item tables and
//! the stacking predicate decide (`wares`). Everything else - the deficits,
//! the weights, the caution, the groups, the solve - is recomputed by the
//! same code that produced it: [`step`] is the one core both sides call.
//!
//! The urgent pass is not replayed: its result (`urgent needs`) is the step's
//! input, and the pass has its own suite (phase 2).
//!
//! Production uses the same core for opt-in live execution and optional
//! comparison records. Historical v4 records retain their original model;
//! current v5 records use trip-relative utility.

use std::collections::BTreeMap;

use crate::borg_headers::borg_h::BORG_TOWN_PLAN;
use crate::borg_headers::borg_trait_h::BI_MAX;
use crate::borg_town_budget::{Budget, UrgentPurchase};
use crate::borg_town_dispose::{dispose, Disposition, Freed, KitItem, Place};
use crate::borg_town_solve::{groups, solve, Group, GroupKey, Limits, Solution};
use crate::borg_town_stock::{Candidate, Stock};
use crate::borg_town_usefulness::{usefulness, Need, Trip, Weights};
use crate::state::BorgState;

/// The plan record's schema version (v2: the decision record, v3: the
/// handover's state record).
pub const PLAN_SCHEMA: i64 = 11;

/// The 24 town settings, `borg_town_plan` through
/// `borg_town_power_scale`, contiguous; v4 stores the first 12.
pub const PARAMS: usize = 24;

/// The action codes: v2's, extended for the plan.
#[cfg_attr(not(test), expect(dead_code))]
pub mod action {
    pub const VISIT: i64 = 0;
    pub const BUY: i64 = 1;
    pub const TAKE_HOME: i64 = 2;
    pub const SELL: i64 = 3;
    pub const SELL_HOME: i64 = 4;
    pub const STORE: i64 = 5;
    pub const DROP: i64 = 6;
    pub const LEAVE: i64 = 7;
}

/// The `solve` flags, as bits.
pub mod flag {
    pub const OPTIMAL: i64 = 1;
    pub const GOLD_EXACT: i64 = 2;
    pub const SLOTS_MET: i64 = 4;
    pub const WEIGHT_MET: i64 = 8;
    pub const WEIGHT_REPAIRED: i64 = 16;
}

/// The engine-decided facts of one ware (the `wares` field).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WareFact {
    pub shop: u8,
    pub slot: u8,
    /// The object kind (`k_idx`); v2's tuple carries the tval.
    pub kind: u32,
    /// The stack class (phase 2).
    pub class: u16,
    /// 0 if a copy joins a carried stack, else 1.
    pub slots_cost: i32,
    /// The weight of one copy.
    pub weight: i32,
    /// The need it serves, as an index into [`Need::ALL`]; -1 for none.
    pub need: i8,
    /// What one copy closes of that need, in the need's own units.
    pub reduction: i64,
    /// The urgent needs it discharges, a bitmask in [`Need::ALL`] order.
    pub serves: u8,
}

/// A wearable's try-on (the `tryon` field): the equipment slot and the
/// power gained by wearing it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TryOn {
    pub shop: u8,
    pub slot: u8,
    pub eq_slot: u8,
    /// The power gained by wearing it, in `borg_power` points; [`step`]
    /// prices it through the power need's curve.
    pub value: i64,
}

/// One chosen option of the plan (the `plan` field), or the runner-up.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlanEntry {
    /// 0 a stack class, 1 an equipment slot.
    pub key_kind: u8,
    pub key: i64,
    pub option: usize,
    pub copies: i32,
    pub cost: i64,
    pub value: i64,
    pub slots: i32,
}

/// The v4 fields (after v2's).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanRecord {
    pub capability: Option<crate::borg_town_capability::Model>,
    /// 4 historical flat/capped model, 5 trip-relative utility.
    pub schema: i64,
    /// 0 shadow (computed, not executed), 1 live.
    pub mode: u8,
    pub visit: u64,
    pub step: u64,
    /// The game's turn (v2's `turn` is the Borg's clock).
    pub gturn: i64,
    /// `borg.power`: the power need reads it, and it is no trait.
    pub power: i64,
    /// The spell capabilities no trait counts, as bits: 1 a curing spell
    /// (Herbal Curing) is usable.
    pub spells: u8,
    /// Town settings in index order (v4: 12, v5: 24).
    pub params: [i64; PARAMS],
    /// `[gold, pack_slots, home_slots, weight]` after the urgent needs.
    pub limits: [i64; 4],
    /// `[pack_capacity, home_capacity]`.
    pub caps: [i64; 2],
    /// `(shop, slot, needs)` of each urgent purchase, `needs` a bitmask.
    pub urgent: Vec<(u8, u8, u8)>,
    /// The urgent needs the visit could not meet, a bitmask.
    pub unmet: u8,
    pub wares: Vec<WareFact>,
    pub tryon: Vec<TryOn>,
    /// The pack's and the Home's stacks, for the dispositions (slice 3).
    pub kit: Vec<KitItem>,
    /// The disposition of each `kit` stack: 0 keep, 1 store, 2 sell,
    /// 3 drop, 4 take.
    pub dispose: Vec<u8>,
    /// The chosen set, ranked.
    pub plan: Vec<PlanEntry>,
    /// `[value, bound, lambda, flags]`.
    pub solve: [i64; 4],
    /// The plan's action, written as `paction`: v2's `action` keeps its
    /// meaning (the Borg's goal), and one object cannot hold two keys of
    /// one name.
    pub action: i64,
    /// `[shop, slot_or_item, copies, price]`.
    pub act: [i64; 4],
    /// The need the action served (0..7), -1 for none.
    pub for_need: i64,
    /// 1 when the action was a urgent purchase.
    pub now: bool,
    /// The runner-up: the best option the solve did not take.
    pub displaced: Option<PlanEntry>,
    /// 0 none, 1 the 2000-turn give-up, 2 the anti-bounce give-up, 3 the
    /// solver refused.
    pub guard: u8,
}

// --- writing -------------------------------------------------------------

fn num(out: &mut String, v: impl std::fmt::Display) {
    out.push_str(&v.to_string());
}

fn arr<T>(out: &mut String, items: &[T], mut each: impl FnMut(&mut String, &T)) {
    out.push('[');
    for (i, x) in items.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        each(out, x);
    }
    out.push(']');
}

fn ints(out: &mut String, xs: &[i64]) {
    arr(out, xs, |o, &x| num(o, x));
}

fn entry(e: &PlanEntry) -> [i64; 7] {
    [
        i64::from(e.key_kind),
        e.key,
        e.option as i64,
        i64::from(e.copies),
        e.cost,
        e.value,
        i64::from(e.slots),
    ]
}

/// A kit stack as the record writes it: `[place, slot, kind, qty, weight,
/// need, reduction, power, proceeds, sell_shop, flags]`, `place` 0 pack 1
/// Home, `flags` bit 0 unknown, bit 1 unknown flavour, bit 2 just bought.
fn kit_row(k: &KitItem) -> [i64; 12] {
    [
        i64::from(k.place == Place::Home),
        i64::from(k.slot),
        i64::from(k.kind),
        i64::from(k.qty),
        i64::from(k.weight),
        i64::from(k.need),
        k.reduction,
        k.power,
        k.proceeds,
        i64::from(k.sell_shop),
        i64::from(k.unknown) | i64::from(k.unknown_flavour) << 1 | i64::from(k.just_bought) << 2 | i64::from(k.joins_pack) << 3,
        i64::from(k.serves),
    ]
}

/// The disposition codes.
pub fn dispose_code(d: Disposition) -> u8 {
    match d {
        Disposition::Keep => 0,
        Disposition::Store => 1,
        Disposition::Sell => 2,
        Disposition::Drop => 3,
        Disposition::Take => 4,
    }
}

impl PlanRecord {
    /// Append the v4 fields, each with its leading comma, in the fixed
    /// order: the caller writes v2's fields (with `"v":4`) before them and
    /// the closing brace after.
    pub fn push_fields(&self, out: &mut String) {
        let field = |out: &mut String, name: &str| {
            out.push_str(",\"");
            out.push_str(name);
            out.push_str("\":");
        };
        field(out, "mode");
        num(out, self.mode);
        field(out, "visit");
        num(out, self.visit);
        field(out, "step");
        num(out, self.step);
        field(out, "gturn");
        num(out, self.gturn);
        field(out, "power");
        num(out, self.power);
        field(out, "spells");
        num(out, self.spells);
        field(out, "params");
        ints(out, &self.params[..if self.schema == 4 { 12 } else { PARAMS }]);
        field(out, "limits");
        ints(out, &self.limits);
        field(out, "caps");
        ints(out, &self.caps);
        field(out, "urgent");
        arr(out, &self.urgent, |o, &(s, sl, n)| ints(o, &[i64::from(s), i64::from(sl), i64::from(n)]));
        field(out, "unmet");
        num(out, self.unmet);
        field(out, "wares");
        arr(out, &self.wares, |o, w| {
            ints(
                o,
                &[
                    i64::from(w.shop),
                    i64::from(w.slot),
                    i64::from(w.kind),
                    i64::from(w.class),
                    i64::from(w.slots_cost),
                    i64::from(w.weight),
                    i64::from(w.need),
                    w.reduction,
                    i64::from(w.serves),
                ],
            )
        });
        field(out, "tryon");
        arr(out, &self.tryon, |o, t| {
            ints(o, &[i64::from(t.shop), i64::from(t.slot), i64::from(t.eq_slot), t.value])
        });
        if let Some(model) = &self.capability {
            if self.schema >= 9 { field(out,"available"); arr(out,&model.available,|o,r| ints(o,&r[..])); }
            if let Some(p) = &model.packing {
                field(out, "packing"); ints(out, &[p.block,p.used]);
                field(out, "quiver_classes"); arr(out, &p.classes, |o,r| ints(o,&r[..]));
            }
            field(out, "capability_curves");
            arr(out, &model.curves, |o, r| ints(o, &r[..]));
            field(out, "capability_supplies");
            arr(out, &model.supplies, |o, r| ints(o, &r[..]));
        }
        field(out, "kit");
        arr(out, &self.kit, |o, k| ints(o, &kit_row(k)[..if self.schema == 4 { 11 } else { 12 }]));
        field(out, "dispose");
        arr(out, &self.dispose, |o, &d| num(o, d));
        field(out, "plan");
        arr(out, &self.plan, |o, e| ints(o, &entry(e)));
        field(out, "solve");
        ints(out, &self.solve);
        field(out, "paction");
        num(out, self.action);
        field(out, "act");
        ints(out, &self.act);
        field(out, "for");
        num(out, self.for_need);
        field(out, "now");
        num(out, i64::from(self.now));
        field(out, "displaced");
        match &self.displaced {
            Some(e) => ints(out, &entry(e)),
            None => ints(out, &[-1]),
        }
        field(out, "guard");
        num(out, self.guard);
    }
}

// --- reading -------------------------------------------------------------

/// The records' JSON: integers, arrays and objects only.
#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    Int(i64),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    /// Parse one record line.
    #[cfg_attr(not(test), expect(dead_code))]
    pub fn parse(text: &str) -> Result<Json, String> {
        let b = text.as_bytes();
        let mut at = 0;
        let v = parse_value(b, &mut at)?;
        skip_ws(b, &mut at);
        if at != b.len() {
            return Err(format!("trailing input at {at}"));
        }
        Ok(v)
    }

    fn get(&self, key: &str) -> Result<&Json, String> {
        match self {
            Json::Obj(kv) => kv.iter().find(|(k, _)| k == key).map(|(_, v)| v).ok_or(format!("no field {key}")),
            _ => Err(format!("not an object looking for {key}")),
        }
    }

    fn int(&self) -> Result<i64, String> {
        match self {
            Json::Int(i) => Ok(*i),
            _ => Err("not an integer".into()),
        }
    }

    fn arr(&self) -> Result<&[Json], String> {
        match self {
            Json::Arr(a) => Ok(a),
            _ => Err("not an array".into()),
        }
    }

    fn ints(&self) -> Result<Vec<i64>, String> {
        self.arr()?.iter().map(Json::int).collect()
    }

    /// The object's keys, in order.
    #[cfg_attr(not(test), expect(dead_code))]
    pub fn keys(&self) -> Vec<&str> {
        match self {
            Json::Obj(kv) => kv.iter().map(|(k, _)| k.as_str()).collect(),
            _ => Vec::new(),
        }
    }
}

fn skip_ws(b: &[u8], at: &mut usize) {
    while *at < b.len() && b[*at].is_ascii_whitespace() {
        *at += 1;
    }
}

fn parse_value(b: &[u8], at: &mut usize) -> Result<Json, String> {
    skip_ws(b, at);
    match b.get(*at) {
        Some(b'[') => {
            *at += 1;
            let mut v = Vec::new();
            skip_ws(b, at);
            if b.get(*at) == Some(&b']') {
                *at += 1;
                return Ok(Json::Arr(v));
            }
            loop {
                v.push(parse_value(b, at)?);
                skip_ws(b, at);
                match b.get(*at) {
                    Some(b',') => *at += 1,
                    Some(b']') => {
                        *at += 1;
                        return Ok(Json::Arr(v));
                    }
                    _ => return Err(format!("bad array at {at}")),
                }
            }
        }
        Some(b'{') => {
            *at += 1;
            let mut kv = Vec::new();
            skip_ws(b, at);
            if b.get(*at) == Some(&b'}') {
                *at += 1;
                return Ok(Json::Obj(kv));
            }
            loop {
                skip_ws(b, at);
                if b.get(*at) != Some(&b'"') {
                    return Err(format!("bad key at {at}"));
                }
                let start = *at + 1;
                let end = start + b[start..].iter().position(|&c| c == b'"').ok_or("unterminated key")?;
                let key = String::from_utf8_lossy(&b[start..end]).into_owned();
                *at = end + 1;
                skip_ws(b, at);
                if b.get(*at) != Some(&b':') {
                    return Err(format!("no colon at {at}"));
                }
                *at += 1;
                kv.push((key, parse_value(b, at)?));
                skip_ws(b, at);
                match b.get(*at) {
                    Some(b',') => *at += 1,
                    Some(b'}') => {
                        *at += 1;
                        return Ok(Json::Obj(kv));
                    }
                    _ => return Err(format!("bad object at {at}")),
                }
            }
        }
        Some(c) if *c == b'-' || c.is_ascii_digit() => {
            let start = *at;
            *at += 1;
            while *at < b.len() && b[*at].is_ascii_digit() {
                *at += 1;
            }
            std::str::from_utf8(&b[start..*at])
                .ok()
                .and_then(|s| s.parse().ok())
                .map(Json::Int)
                .ok_or(format!("bad integer at {start}"))
        }
        _ => Err(format!("unexpected input at {at}")),
    }
}

fn entry_of(xs: &[i64]) -> Result<PlanEntry, String> {
    if xs.len() != 7 {
        return Err("a plan entry has 7 fields".into());
    }
    Ok(PlanEntry {
        key_kind: xs[0] as u8,
        key: xs[1],
        option: xs[2] as usize,
        copies: xs[3] as i32,
        cost: xs[4],
        value: xs[5],
        slots: xs[6] as i32,
    })
}

fn fixed<const N: usize>(xs: Vec<i64>, what: &str) -> Result<[i64; N], String> {
    xs.try_into().map_err(|_| format!("{what} has the wrong length"))
}

impl PlanRecord {
    /// Read the v4 fields of a parsed record.
    #[cfg_attr(not(test), expect(dead_code))]
    pub fn from_json(j: &Json) -> Result<PlanRecord, String> {
        let schema = j.get("v")?.int()?;
        if schema != 4 && schema != 5 && schema != 6 && schema != 7 && schema != 8 && schema != 9 && schema != 10 && schema != PLAN_SCHEMA { return Err("not a supported plan record".into()); }
        let raw_params = j.get("params")?.ints()?;
        if raw_params.len() != if schema == 4 { 12 } else { PARAMS } { return Err("params has the wrong length".into()); }
        let mut params = [0; PARAMS];
        params[..raw_params.len()].copy_from_slice(&raw_params);
        let small = |k: &str| -> Result<i64, String> { j.get(k)?.int() };
        let rows = |k: &str| -> Result<Vec<Vec<i64>>, String> { j.get(k)?.arr()?.iter().map(Json::ints).collect() };
        let displaced = j.get("displaced")?.ints()?;
        let capability = if schema >= 7 {
            let model = crate::borg_town_capability::Model {
                available: if schema >= 9 { rows("available")?.into_iter().map(|r| fixed(r,"available ware")).collect::<Result<_,_>>()? } else { Vec::new() },
                packing: if schema >= 9 {
                    let header = fixed::<2>(j.get("packing")?.ints()?, "packing")?;
                    Some(crate::borg_town_packing::Packing { block: header[0], used: header[1],
                        classes: rows("quiver_classes")?.into_iter().map(|r| fixed(r,"quiver class")).collect::<Result<_,_>>()? })
                } else { None },
                curves: rows("capability_curves")?.into_iter().map(|r| fixed(r, "capability curve")).collect::<Result<_, _>>()?,
                supplies: rows("capability_supplies")?.into_iter().map(|r| fixed(r, "capability supply")).collect::<Result<_, _>>()?,
            };
            if !model.valid() { return Err("invalid capability facts".into()); }
            Some(model)
        } else { None };
        Ok(PlanRecord {
            capability,
            schema,
            mode: small("mode")? as u8,
            visit: small("visit")? as u64,
            step: small("step")? as u64,
            gturn: small("gturn")?,
            power: small("power")?,
            spells: small("spells")? as u8,
            params,
            limits: fixed(j.get("limits")?.ints()?, "limits")?,
            caps: fixed(j.get("caps")?.ints()?, "caps")?,
            urgent: rows("urgent")?
                .into_iter()
                .map(|r| match r[..] {
                    [s, sl, n] => Ok((s as u8, sl as u8, n as u8)),
                    _ => Err("an urgent purchase has 3 fields".to_string()),
                })
                .collect::<Result<_, _>>()?,
            unmet: small("unmet")? as u8,
            wares: rows("wares")?
                .into_iter()
                .map(|r| match r[..] {
                    [shop, slot, kind, class, slots_cost, weight, need, reduction, serves] => Ok(WareFact {
                        shop: shop as u8,
                        slot: slot as u8,
                        kind: kind as u32,
                        class: class as u16,
                        slots_cost: slots_cost as i32,
                        weight: weight as i32,
                        need: need as i8,
                        reduction,
                        serves: serves as u8,
                    }),
                    _ => Err("a ware has 9 fields".to_string()),
                })
                .collect::<Result<_, _>>()?,
            tryon: rows("tryon")?
                .into_iter()
                .map(|r| match r[..] {
                    [shop, slot, eq_slot, value] => {
                        Ok(TryOn { shop: shop as u8, slot: slot as u8, eq_slot: eq_slot as u8, value })
                    }
                    _ => Err("a try-on has 4 fields".to_string()),
                })
                .collect::<Result<_, _>>()?,
            kit: rows("kit")?
                .into_iter()
                .map(|r| {
                    if r.len() != if schema == 4 { 11 } else { 12 } { return Err("wrong kit row length".into()); }
                    let serves = if schema == 4 { 0 } else { r[11] as u8 };
                    match r[..11] {
                    [place, slot, kind, qty, weight, need, reduction, power, proceeds, sell_shop, flags] => Ok(KitItem {
                        joins_pack: schema >= 9 && flags & 8 != 0,
                        place: if place == 1 { Place::Home } else { Place::Pack },
                        slot: slot as u8,
                        kind: kind as u32,
                        qty: qty as i32,
                        weight: weight as i32,
                        need: need as i8,
                        reduction,
                        serves,
                        power,
                        proceeds,
                        sell_shop: sell_shop as i8,
                        unknown: flags & 1 != 0,
                        unknown_flavour: flags & 2 != 0,
                        just_bought: flags & 4 != 0,
                    }),
                    _ => Err("a kit stack has 11 fields (v4), 12 (v5)".to_string()),
                } })
                .collect::<Result<_, _>>()?,
            dispose: j.get("dispose")?.ints()?.into_iter().map(|d| d as u8).collect(),
            plan: rows("plan")?.iter().map(|r| entry_of(r)).collect::<Result<_, _>>()?,
            solve: fixed(j.get("solve")?.ints()?, "solve")?,
            action: small("paction")?,
            act: fixed(j.get("act")?.ints()?, "act")?,
            for_need: small("for")?,
            now: small("now")? != 0,
            displaced: if displaced == [-1] { None } else { Some(entry_of(&displaced)?) },
            guard: small("guard")? as u8,
        })
    }
}

// --- the step: the one core the producer and the replay share ------------

/// What a step reads.
pub struct StepInput<'a> {
    pub revision: i64,
    pub capability: Option<&'a crate::borg_town_capability::Model>,
    pub utility: bool,
    /// The candidates, in stock order.
    pub candidates: Vec<Candidate>,
    pub facts: &'a [WareFact],
    pub tryon: &'a [TryOn],
    pub limits: Limits,
    pub urgent: Vec<UrgentPurchase>,
    pub params: [i64; PARAMS],
    /// The whole trait vector.
    pub traits: &'a [i32],
    pub power: i64,
    /// The carried items (pack and equipment) as `(tval, quantity)`: the
    /// light need counts the light sources among them.
    pub carried: &'a [(u8, i32)],
    /// A curing spell is usable (no trait counts it).
    pub cure_spell: bool,
    /// The pack's and the Home's stacks, for the dispositions.
    pub kit: &'a [KitItem],
    /// The Home's free slots.
    pub home_slots: i32,
}

/// What a step produces.
#[derive(Clone, Debug)]
pub struct StepOutput {
    pub groups: Vec<Group>,
    pub solution: Solution,
    pub plan: Vec<PlanEntry>,
    pub solve: [i64; 4],
    pub displaced: Option<PlanEntry>,
    /// Stage 1: the disposition of each `StepInput::kit` stack.
    pub dispositions: Vec<Disposition>,
    /// What the dispositions freed for the purchases.
    #[cfg_attr(not(test), expect(dead_code))]
    pub freed: Freed,
}

/// A state the usefulness function can read: the traits, the power, and
/// the settings with the twelve town settings from `params`.
fn state_of(traits: &[i32], power: i64, params: &[i64; PARAMS], carried: &[(u8, i32)]) -> BorgState {
    let mut st = BorgState::default();
    st.items.items = carried
        .iter()
        .map(|&(tval, qty)| {
            let mut it = crate::borg_item::blank_item();
            it.tval = tval;
            it.iqty = qty.clamp(0, 255) as u8;
            it
        })
        .collect();
    st.core.cfg = crate::borg_init::BORG_SETTINGS.iter().map(|s| s.default_value).collect();
    for (i, &p) in params.iter().enumerate() {
        st.core.cfg[BORG_TOWN_PLAN as usize + i] = p as i32;
    }
    let mut t = vec![0; BI_MAX as usize];
    t[..traits.len().min(BI_MAX as usize)].copy_from_slice(&traits[..traits.len().min(BI_MAX as usize)]);
    st.borg.trait_ = t;
    st.borg.power = power as i32;
    st
}

fn plan_entry(g: &Group, option: usize) -> PlanEntry {
    let c = &g.options[option];
    let (key_kind, key) = match g.key {
        GroupKey::Class(k) => (0, i64::from(k)),
        GroupKey::Need(n) => (2, i64::from(n)),
        GroupKey::Slot(s) => (1, i64::from(s)),
    };
    PlanEntry {
        key_kind,
        key,
        option,
        copies: c.picks.iter().map(|&(_, k)| k).sum(),
        cost: c.cost,
        value: c.value,
        slots: c.slots,
    }
}

/// The runner-up: of the options that would raise a group above what the
/// solve chose, the one adding the most value; ties to less gold, then the
/// earlier group, then the lower option.
fn runner_up(groups: &[Group], s: &Solution) -> Option<PlanEntry> {
    let mut best: Option<(usize, usize, (i64, i64))> = None;
    for (g, &o) in s.chosen.iter().enumerate() {
        let cur = &groups[g].options[o];
        for (j, alt) in groups[g].options.iter().enumerate() {
            let gain = alt.value - cur.value;
            if gain <= 0 {
                continue;
            }
            let key = (gain, -(alt.cost - cur.cost));
            if best.is_none_or(|(_, _, k)| key > k) {
                best = Some((g, j, key));
            }
        }
    }
    best.map(|(g, j, _)| plan_entry(&groups[g], j))
}

/// One plan step: the groups, the solve, the ranked plan and the runner-up.
pub fn step(input: &StepInput) -> StepOutput {
    let mut st = state_of(input.traits, input.power, &input.params, input.carried);
    let mut trip = Trip::of(&st);
    trip.cure_spell = input.cure_spell;
    let weights = Weights::of(&st.core.cfg);

    // Stage 1 (owner, 2026-11-14: two stages): the dispositions of the
    // pack and the Home, and what they free.
    let mut utility = crate::borg_town_utility::Utility::of(&st, input.kit, input.cure_spell);
    utility.capability = input.capability.cloned();
    if input.utility && input.revision >= 9 {
        for purchase in &input.urgent {
            let c = &input.candidates[purchase.candidate];
            if c.shop == 7 {
                if let Some(k) = input.kit.iter().find(|k| k.place == Place::Home && k.slot == c.slot) {
                    let one = KitItem { qty: 1, ..*k };
                    if let Some(m) = &mut utility.capability { m.change_home(&one, -1); }
                    if let Ok(n) = usize::try_from(k.need) {
                        if let Some(v) = utility.home.get_mut(n) { *v = (*v - k.reduction).max(0); }
                    }
                }
            }
            if let Some(m) = &mut utility.capability {
                for (id, units) in m.units(i64::from(c.shop), i64::from(c.slot), 1) { m.curves[id][3] += units; }
            }
            if let Some(f) = input.facts.iter().find(|f| f.shop == c.shop && f.slot == c.slot) {
                utility.add(f.need, f.reduction);
                if purchase.needs.contains(&Need::Poison) && f.need != crate::borg_town_utility::index(Need::Poison) as i8 {
                    utility.add(crate::borg_town_utility::index(Need::Poison) as i8, crate::borg_town_utility::UNIT);
                }
            }
        }
    }
    let (dispositions, freed) = if input.utility {
        crate::borg_town_utility::dispositions(input, &mut utility, &st)
    } else { dispose(
        input.kit,
        &st,
        &trip,
        &weights,
        input.limits.pack_slots,
        input.home_slots,
        input.limits.weight,
    ) };
    // What they change in the needs the purchases see: a stack that leaves
    // the pack takes its copies out of its need, one taken from the Home
    // brings them in.
    for (it, d) in input.kit.iter().zip(&dispositions) {
        let sign = match (it.place, d) {
            (Place::Pack, Disposition::Store | Disposition::Sell | Disposition::Drop) => -1,
            (Place::Home, Disposition::Take) => 1,
            _ => 0,
        };
        if sign != 0 && !input.utility {
            apply_need(&mut st, it.need, it.reduction, it.qty, sign);
        }
    }
    if input.utility && input.revision < 9 {
        for purchase in &input.urgent {
            let c = &input.candidates[purchase.candidate];
            if let Some(m) = &mut utility.capability {
                for (id, units) in m.units(i64::from(c.shop), i64::from(c.slot), 1) { m.curves[id][3] += units; }
            }
            if let Some(f) = input.facts.iter().find(|f| f.shop == c.shop && f.slot == c.slot) {
                utility.add(f.need, f.reduction);
                if purchase.needs.contains(&Need::Poison) && f.need != crate::borg_town_utility::index(Need::Poison) as i8 {
                    utility.add(crate::borg_town_utility::index(Need::Poison) as i8, crate::borg_town_utility::UNIT);
                }
            }
        }
    }
    // Stage 2: the purchases, with what the dispositions freed.
    let limits = Limits {
        gold: input.limits.gold + freed.gold,
        pack_slots: input.limits.pack_slots + freed.pack_slots,
        weight: input.limits.weight + freed.weight,
    };
    let st = st;
    let facts: BTreeMap<(u8, u8), &WareFact> = input.facts.iter().map(|f| ((f.shop, f.slot), f)).collect();
    let tryon: BTreeMap<(u8, u8), &TryOn> = input.tryon.iter().map(|t| ((t.shop, t.slot), t)).collect();
    let mut candidates = input.candidates.clone();
    if input.utility {
        for c in &mut candidates {
            if input.revision >= 9 && c.shop == 7 { c.qty = 0; }
            if !tryon.contains_key(&(c.shop, c.slot)) && input.kit.iter().zip(&dispositions).any(|(it, d)|
                it.place == Place::Pack && it.kind == c.kind && *d != Disposition::Keep) {
                c.slots_cost = 1;
            }
        }
    }
    let stock = Stock { candidates, ..Stock::default() };
    let budget = Budget {
        gold: limits.gold,
        pack_slots: limits.pack_slots,
        home_slots: 0,
        weight: limits.weight,
        urgent: input.urgent.clone(),
        unmet: Vec::new(),
    };
    let caution = weights.caution;
    // Buy to identify (owner, 2026-11-14): a learnable ware - an unaware
    // kind, or unknown runes - is worth the knowledge weight for its first
    // copy; a second copy teaches nothing. Each unknown kind is worth the
    // whole weight on its own.
    let knowledge_bit = 1u8 << Need::ALL.iter().position(|&n| n == Need::Knowledge).unwrap_or(5);
    let learn = |c: &Candidate, n: i32| -> i64 {
        match facts.get(&(c.shop, c.slot)) {
            Some(f) if f.serves & knowledge_bit != 0 && n >= 1 => if input.utility { weights.knowledge * crate::borg_town_utility::UNIT } else { weights.knowledge },
            _ => 0,
        }
    };
    let groups = groups(
        &stock,
        &budget,
        caution,
        // A wearable's try-on gain is power points; the power need's curve
        // prices it on the same scale as every other need.
        |c| {
            tryon.get(&(c.shop, c.slot)).map(|t| {
                (t.eq_slot, if input.utility { utility.power(t.value)
                    + utility.capability.as_ref().map_or(0, |m| m.ware(c.shop, c.slot, 1))
                    + learn(c, 1) } else { usefulness(Need::Power, t.value, 1, &st, &trip, &weights) + learn(c, 1) })
            })
        },
        |c, n| {
            let need = if let Some(model) = &utility.capability { model.ware(c.shop, c.slot, n) } else if input.utility { crate::borg_town_utility::purchase_value(&utility, facts.get(&(c.shop, c.slot)), n) } else { match facts.get(&(c.shop, c.slot)) {
                Some(f) if f.need >= 0 => {
                    usefulness(Need::ALL[f.need as usize], f.reduction, i64::from(n), &st, &trip, &weights)
                }
                _ => 0,
            } };
            need + learn(c, n)
        },
    );
    let groups = if let Some(p) = utility.capability.as_ref().and_then(|m| m.packing.as_ref()) {
        p.bound(groups, &stock.candidates)
    } else { groups };
    let groups = if let Some(model) = &utility.capability {
        crate::borg_town_capability::shared_groups(groups, input, &stock.candidates, model, &limits,
            (utility.space_weight, i64::from(st.borg.tr(crate::borg_headers::borg_trait_h::BI_CARRY)).max(2) / 2, weights.knowledge))
    } else if input.utility { crate::borg_town_utility::shared_groups(groups, input, &stock.candidates, &utility, &limits, &st) } else { groups };
    let solution = if input.utility { crate::borg_town_solve::solve_cash(&groups, &limits, utility.gold_weight) } else { solve(&groups, &limits) };
    let plan = solution.ranked(&groups).into_iter().map(|(g, o)| plan_entry(&groups[g], o)).collect();
    let flags = [
        (solution.optimal, flag::OPTIMAL),
        (solution.gold_exact, flag::GOLD_EXACT),
        (solution.slots_met, flag::SLOTS_MET),
        (solution.weight_met, flag::WEIGHT_MET),
        (solution.weight_repaired, flag::WEIGHT_REPAIRED),
    ]
    .iter()
    .filter(|(on, _)| *on)
    .fold(0, |m, (_, b)| m | b);
    let solve_row = [solution.value, solution.bound, solution.lambda, flags];
    let displaced = runner_up(&groups, &solution);
    StepOutput { groups, solution, plan, solve: solve_row, displaced, dispositions, freed }
}

/// Add (`sign` 1) or remove (`sign` -1) `qty` copies' worth of a need in a
/// step's state copy: light sources to the carried lights, food to
/// `BI_FOOD`, wound cures to `BI_ACLW`, poison cures to `BI_ACUREPOIS`,
/// Phase Door to `BI_APHASE`. Other needs carry nothing countable.
pub fn apply_need(st: &mut BorgState, need: i8, reduction: i64, qty: i32, sign: i32) {
    use crate::borg_headers::borg_trait_h::{BI_ACLW, BI_ACUREPOIS, BI_APHASE, BI_FOOD, BI_RECALL};
    use crate::headers::list_tvals_h::TV_LIGHT;
    let n = sign * qty;
    let at = |st: &mut BorgState, t: u32, d: i32| {
        if let Some(v) = st.borg.trait_.get_mut(t as usize) {
            *v = (*v + d).max(0);
        }
    };
    match need {
        0 if sign > 0 => {
            let mut it = crate::borg_item::blank_item();
            it.tval = TV_LIGHT as u8;
            it.iqty = qty.clamp(0, 255) as u8;
            st.items.items.push(it);
        }
        0 => {
            let mut left = qty;
            for it in st.items.items.iter_mut().filter(|it| u32::from(it.tval) == TV_LIGHT) {
                let take = left.min(i32::from(it.iqty));
                it.iqty -= take as u8;
                left -= take;
            }
        }
        1 => at(st, BI_FOOD, n * reduction as i32),
        2 => at(st, BI_ACLW, n),
        3 => at(st, BI_ACUREPOIS, n),
        4 => at(st, BI_APHASE, n),
        7 => at(st, BI_RECALL, n),
        _ => {}
    }
}

/// The step's action: the urgent needs first (B3, Part A), then the first ranked
/// option of the plan, else leave. Returns `(paction, act, for, urgent need)`,
/// `act` = `[shop, slot, copies, price]`; a purchase takes its cheapest
/// ware first, as the cost table does.
pub fn first_action(
    candidates: &[Candidate],
    urgent: &[UrgentPurchase],
    out: &StepOutput,
    facts: &[WareFact],
    kit: &[KitItem],
) -> (i64, [i64; 4], i64, bool) {
    first_action_with_room(candidates, urgent, out, facts, kit, None, None)
}

/// Respect a full Home's prerequisite: free a Home slot before depositing.
/// Historical shadow records retain their original order through first_action.
pub fn first_executable_action(
    candidates: &[Candidate], urgent: &[UrgentPurchase], out: &StepOutput,
    facts: &[WareFact], kit: &[KitItem], home_slots: i32, pack_slots: i32,
) -> (i64, [i64; 4], i64, bool) {
    first_action_with_room(candidates, urgent, out, facts, kit, Some(home_slots), Some(pack_slots))
}

fn first_action_with_room(
    candidates: &[Candidate], urgent: &[UrgentPurchase], out: &StepOutput,
    facts: &[WareFact], kit: &[KitItem], home_slots: Option<i32>, pack_slots: Option<i32>,
) -> (i64, [i64; 4], i64, bool) {
    let need_of = |c: &Candidate| {
        facts.iter().find(|f| f.shop == c.shop && f.slot == c.slot).map_or(-1, |f| i64::from(f.need))
    };
    // A blocked new stack must not hide a later emergency that joins stock
    // already carried. Historical shadow selection has no pack-room filter.
    let feasible_urgent = urgent.iter().find(|f| pack_slots.is_none_or(|n| n >= candidates[f.candidate].slots_cost));
    let blocked_urgent = !urgent.is_empty() && feasible_urgent.is_none();
    if let Some(f) = feasible_urgent {
        let c = &candidates[f.candidate];
        let need = f.needs.first().and_then(|n| Need::ALL.iter().position(|x| x == n)).map_or(-1, |i| i as i64);
        return (if c.shop == 7 { action::TAKE_HOME } else { action::BUY }, [i64::from(c.shop), i64::from(c.slot), 1, i64::from(c.price)], need, true);
    }
    // The dispositions before the purchases, in the order that frees room
    // first: the pack's sales and drops, then its stores, then the Home's
    // sales, then what comes back from the Home.
    let order = if home_slots.is_some_and(|n| n <= 0) { [
        (Place::Pack, Disposition::Sell),
        (Place::Pack, Disposition::Drop),
        (Place::Home, Disposition::Sell),
        (Place::Pack, Disposition::Store),
        (Place::Home, Disposition::Take),
    ] } else { [
        (Place::Pack, Disposition::Sell),
        (Place::Pack, Disposition::Drop),
        (Place::Pack, Disposition::Store),
        (Place::Home, Disposition::Sell),
        (Place::Home, Disposition::Take),
    ] };
    for (place, d) in order {
        if let Some((k, _)) = kit.iter().zip(&out.dispositions).find(|(k, &x)| k.place == place && x == d
            && !(place == Place::Pack && d == Disposition::Store && home_slots.is_some_and(|n| n <= 0))
            && !(place == Place::Home && pack_slots.is_some_and(|n| n <= 0) && !k.joins_pack)) {
            let (code, at) = match (place, d) {
                (Place::Pack, Disposition::Sell) => (action::SELL, i64::from(k.sell_shop)),
                (Place::Pack, Disposition::Drop) => (action::DROP, -1),
                (Place::Pack, Disposition::Store) => (action::STORE, 7),
                (Place::Home, Disposition::Sell) => (action::SELL_HOME, i64::from(k.sell_shop)),
                _ => (action::TAKE_HOME, 7),
            };
            return (code, [at, i64::from(k.slot), i64::from(k.qty), k.proceeds], i64::from(k.need), false);
        }
    }
    if blocked_urgent { return (action::LEAVE, [-1; 4], -1, false); }
    if let Some(&(g, o)) = out.solution.ranked(&out.groups).first() {
        if let Some(&(i, copies)) = out.groups[g].options[o].picks.first() {
            let c = &candidates[i];
            return (
                action::BUY,
                [i64::from(c.shop), i64::from(c.slot), i64::from(copies), i64::from(c.price)],
                need_of(c),
                false,
            );
        }
    }
    (action::LEAVE, [-1, -1, -1, -1], -1, false)
}

// --- replay --------------------------------------------------------------

/// Replay a v4 line: rebuild the step's inputs from the line alone and run
/// it. The caller compares the output with the record's `plan`, `solve`
/// and `displaced` ([`replays`] does).
#[cfg_attr(not(test), expect(dead_code))]
pub fn replay(line: &str) -> Result<(PlanRecord, StepOutput), String> {
    let (rec, parts) = replay_parts(line)?;
    let out = step(&parts.input(&rec));
    Ok((rec, out))
}

/// A step's inputs as a replay rebuilds them from a v4 line (the
/// borrowed parts of [`StepInput`] owned here).
pub struct ReplayParts {
    pub candidates: Vec<Candidate>,
    pub urgent: Vec<UrgentPurchase>,
    pub traits: Vec<i32>,
    pub carried: Vec<(u8, i32)>,
    pub limits: Limits,
    pub tryon: Vec<TryOn>,
    pub power: i64,
}

impl ReplayParts {
    /// The step input over these parts and the record's facts.
    pub fn input<'a>(&'a self, rec: &'a PlanRecord) -> StepInput<'a> {
        StepInput {
            revision: rec.schema,
            capability: rec.capability.as_ref(),
            utility: rec.schema >= 5,
            candidates: self.candidates.clone(),
            facts: &rec.wares,
            tryon: &self.tryon,
            limits: self.limits,
            urgent: self.urgent.clone(),
            params: rec.params,
            traits: &self.traits,
            power: self.power,
            carried: &self.carried,
            cure_spell: rec.spells & 1 != 0,
            kit: &rec.kit,
            home_slots: rec.limits[2] as i32,
        }
    }
}

/// Rebuild a step's inputs from a v4 line alone.
pub fn replay_parts(line: &str) -> Result<(PlanRecord, ReplayParts), String> {
    let j = Json::parse(line)?;
    let rec = PlanRecord::from_json(&j)?;

    // v2's shops: [shop, [[slot, tval, sval, quantity, price, flags], ...]].
    let mut shop_rows: BTreeMap<(u8, u8), Vec<i64>> = BTreeMap::new();
    for s in j.get("shops")?.arr()? {
        let s = s.arr()?;
        let shop = s.first().ok_or("an empty shop")?.int()? as u8;
        for w in s.get(1).ok_or("a shop without wares")?.arr()? {
            let w = w.ints()?;
            shop_rows.insert((shop, *w.first().ok_or("an empty ware")? as u8), w);
        }
    }
    // The candidates, in the producer's (shop, slot) order.
    let mut facts = rec.wares.clone();
    facts.sort_by_key(|f| (f.shop, f.slot));
    let candidates: Vec<Candidate> = facts
        .iter()
        .map(|f| {
            let w = shop_rows.get(&(f.shop, f.slot)).ok_or(format!("ware {}:{} not in shops", f.shop, f.slot))?;
            match w[..] {
                [_, tval, sval, qty, price, _] => Ok(Candidate {
                    shop: f.shop,
                    slot: f.slot,
                    kind: f.kind,
                    tval: tval as u8,
                    sval: sval as u8,
                    price: price as i32,
                    qty: rec.capability.as_ref().and_then(|m| m.available.iter().find(|r| r[0]==i64::from(f.shop) && r[1]==i64::from(f.slot))).map_or(qty as i32, |r| r[2] as i32),
                    weight: f.weight,
                    slots_cost: f.slots_cost,
                    stack_count: 0,
                    class: f.class,
                }),
                _ => Err("a v2 ware has 6 fields".to_string()),
            }
        })
        .collect::<Result<_, String>>()?;
    let index: BTreeMap<(u8, u8), usize> =
        candidates.iter().enumerate().map(|(i, c)| ((c.shop, c.slot), i)).collect();
    let urgent_buys = rec
        .urgent
        .iter()
        .map(|&(s, sl, n)| {
            let candidate = *index.get(&(s, sl)).ok_or("an urgent ware not in wares")?;
            let needs = Need::ALL.iter().enumerate().filter(|(b, _)| n & (1 << b) != 0).map(|(_, &x)| x).collect();
            Ok(UrgentPurchase { candidate, needs })
        })
        .collect::<Result<_, String>>()?;

    // v2's traits, sparse.
    let mut traits = vec![0i32; BI_MAX as usize];
    for t in j.get("traits")?.arr()? {
        if let [i, v] = t.ints()?[..] {
            // Versions 4..10 froze the M12 projection. Version 11 keeps
            // those IDs and appends M13 traits, independently of engine IDs.
            if rec.schema < 11 && i >= 265 { continue; }
            if let Some(slot) = crate::borg_town_traits::engine_index(i as usize).and_then(|n| traits.get_mut(n)) {
                *slot = v as i32;
            }
        }
    }

    // v2's pack and equipment: [slot, tval, sval, quantity, value, flags].
    let mut carried = Vec::new();
    for key in ["pack", "equip"] {
        for it in j.get(key)?.arr()? {
            if let [_, tval, _, qty, ..] = it.ints()?[..] {
                carried.push((tval as u8, qty as i32));
            }
        }
    }

    let limits = Limits { gold: rec.limits[0], pack_slots: rec.limits[1] as i32, weight: rec.limits[3] };
    let parts = ReplayParts {
        candidates,
        urgent: urgent_buys,
        traits,
        carried,
        limits,
        tryon: rec.tryon.clone(),
        power: rec.power,
    };
    Ok((rec, parts))
}

/// Does a v4 line replay to its own plan, solve and runner-up?
#[cfg_attr(not(test), expect(dead_code))]
pub fn replays(line: &str) -> Result<bool, String> {
    let (rec, out) = replay(line)?;
    let dispose: Vec<u8> = out.dispositions.iter().map(|&d| dispose_code(d)).collect();
    Ok(rec.plan == out.plan && rec.solve == out.solve && rec.displaced == out.displaced && rec.dispose == dispose)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::borg_headers::borg_trait_h::{BI_CARRY, BI_FOOD, BI_LIGHT, BI_MAXDEPTH};
    use crate::borg_town_budget::Facts;
    use crate::borg_town_stock::tests::{build, item, town};

    // Test kinds (the helpers' tval is the kind).
    const TORCH: u32 = 1;
    const RATION: u32 = 2;
    const CLW: u32 = 3;
    const PHASE: u32 = 5;
    const ARMOUR: u32 = 7;
    const ARMOUR_2: u32 = 8;

    /// The need and reduction a test kind serves, and the urgent needs it
    /// discharges - the role the engine's tables play in production.
    fn role(kind: u32) -> (i8, i64, u8) {
        match kind {
            TORCH => (0, 1, 1 << 0),
            RATION => (1, 1, 1 << 1),
            CLW => (2, 1, 1 << 2),
            PHASE => (4, 1, 0),
            _ => (-1, 0, 0),
        }
    }

    fn params() -> [i64; PARAMS] {
        let d: Vec<i64> = crate::borg_init::BORG_SETTINGS.iter().map(|s| i64::from(s.default_value)).collect();
        let mut p = [0; PARAMS];
        p.copy_from_slice(&d[BORG_TOWN_PLAN as usize..BORG_TOWN_PLAN as usize + PARAMS]);
        p
    }

    /// The v2 part of a record, as the dump writes it: only the fields a
    /// replay reads are filled with meaning.
    fn v2_prefix(stock: &Stock, traits: &[i32], gold: i64) -> String {
        let mut out = String::from("{\"v\":4,\"seq\":0,\"turn\":0,\"depth\":0,\"level\":1,\"chp\":10,\"mhp\":10,");
        out.push_str(&format!("\"gold\":{gold},\"race\":0,\"class\":0,\"traits\":["));
        let tr: Vec<String> =
            traits.iter().enumerate().filter(|(_, &v)| v != 0).map(|(i, v)| format!("[{},{}]",crate::borg_town_traits::record_index(i),v)).collect();
        out.push_str(&tr.join(","));
        out.push_str("],\"action\":0,\"shop\":-1,\"ware\":-1,\"item\":-1,\"price\":-1,");
        // the carried light: one torch worn, of the two wanted
        out.push_str("\"pack\":[],\"equip\":[[29,19,0,1,0,1]],\"home\":[],\"shops\":[");
        let mut by_shop: BTreeMap<u8, Vec<String>> = BTreeMap::new();
        for c in &stock.candidates {
            by_shop
                .entry(c.shop)
                .or_default()
                .push(format!("[{},{},{},{},{},1]", c.slot, c.tval, c.sval, c.qty, c.price));
        }
        let shops: Vec<String> = by_shop.iter().map(|(s, w)| format!("[{s},[{}]]", w.join(","))).collect();
        out.push_str(&shops.join(","));
        out.push(']');
        out
    }

    /// A whole step as the producer will run it, written as a v4 line.
    fn produce(stock: &Stock, gold: i64, traits: &[i32], tryon: &[TryOn], unmet: &[Need]) -> (String, PlanRecord) {
        produce_kit(stock, gold, traits, tryon, unmet, &[])
    }

    fn produce_kit(
        stock: &Stock,
        gold: i64,
        traits: &[i32],
        tryon: &[TryOn],
        unmet: &[Need],
        kit: &[KitItem],
    ) -> (String, PlanRecord) {
        produce_ident(stock, gold, traits, tryon, unmet, kit, &[])
    }

    /// As produce_kit, with the wares of the kinds in `unseen` marked as
    /// unknown (`{unseen}` or `{??}`): a purchase teaches them.
    fn produce_ident(
        stock: &Stock,
        gold: i64,
        traits: &[i32],
        tryon: &[TryOn],
        unmet: &[Need],
        kit: &[KitItem],
        unseen: &[u32],
    ) -> (String, PlanRecord) {
        let facts = Facts { gold, pack_capacity: 23, home_capacity: 24, carry: traits[BI_CARRY as usize], weight: 0 };
        let budget = Budget::build(&facts, stock, unmet, |c| {
            let (_, _, serves) = role(c.kind);
            Need::ALL.iter().enumerate().filter(|(b, _)| serves & (1 << b) != 0).map(|(_, &n)| n).collect()
        });
        let wares: Vec<WareFact> = stock
            .candidates
            .iter()
            .map(|c| {
                let (need, reduction, mut serves) = role(c.kind);
                if unseen.contains(&c.kind) {
                    serves |= 1 << Need::ALL.iter().position(|&n| n == Need::Knowledge).unwrap();
                }
                WareFact {
                    shop: c.shop,
                    slot: c.slot,
                    kind: c.kind,
                    class: c.class,
                    slots_cost: c.slots_cost,
                    weight: c.weight,
                    need,
                    reduction,
                    serves,
                }
            })
            .collect();
        let limits = Limits::of(&budget);
        let out = step(&StepInput {
            revision: 4,
            capability: None,
            utility: false,
            candidates: stock.candidates.clone(),
            facts: &wares,
            tryon,
            limits,
            urgent: budget.urgent.clone(),
            params: params(),
            traits,
            power: 1000,
            carried: &carried(),
            cure_spell: false,
            kit,
            home_slots: budget.home_slots,
        });
        let mask = |ns: &[Need]| {
            ns.iter().fold(0u8, |m, n| m | (1 << Need::ALL.iter().position(|x| x == n).unwrap()))
        };
        let rec = PlanRecord {
            capability: None,
            schema: 4,
            mode: 0,
            visit: 3,
            step: 1,
            gturn: 12345,
            power: 1000,
            spells: 0,
            params: { let mut p = params(); p[12..].fill(0); p },
            limits: [budget.gold, i64::from(budget.pack_slots), i64::from(budget.home_slots), budget.weight],
            caps: [23, 24],
            urgent: budget
                .urgent
                .iter()
                .map(|f| (stock.candidates[f.candidate].shop, stock.candidates[f.candidate].slot, mask(&f.needs)))
                .collect(),
            unmet: mask(&budget.unmet),
            wares,
            tryon: tryon.to_vec(),
            kit: kit.to_vec(),
            dispose: out.dispositions.iter().map(|&d| dispose_code(d)).collect(),
            plan: out.plan.clone(),
            solve: out.solve,
            action: if out.plan.is_empty() { action::LEAVE } else { action::BUY },
            act: [-1, -1, -1, -1],
            for_need: -1,
            now: false,
            displaced: out.displaced,
            guard: 0,
        };
        let mut line = v2_prefix(stock, traits, gold);
        rec.push_fields(&mut line);
        line.push('}');
        (line, rec)
    }

    /// The carried items as the step reads them: one torch (the light
    /// tval), matching v2_prefix's equipment row.
    fn carried() -> Vec<(u8, i32)> {
        vec![(crate::headers::list_tvals_h::TV_LIGHT as u8, 1)]
    }

    fn traits() -> Vec<i32> {
        let mut t = vec![0; BI_MAX as usize];
        t[BI_MAXDEPTH as usize] = 10; // depth target 15: two lights wanted
        t[BI_LIGHT as usize] = 1;
        t[BI_FOOD as usize] = 3; // seven rations short of ten
        t[BI_CARRY as usize] = 2000;
        t
    }

    /// The general store, the alchemist, an armoury and the Black Market.
    fn stock() -> Stock {
        let t = town([
            &[item(TORCH, 5, 2), item(RATION, 20, 4), item(PHASE, 10, 15)],
            &[item(ARMOUR, 1, 200), item(ARMOUR_2, 1, 260)],
            &[],
            &[],
            &[item(CLW, 5, 20)],
            &[],
            &[item(RATION, 5, 12)],
            &[],
        ]);
        build(&t, &[])
    }

    fn armour_tryon(s: &Stock) -> Vec<TryOn> {
        s.candidates
            .iter()
            .filter(|c| c.kind == ARMOUR || c.kind == ARMOUR_2)
            .map(|c| TryOn {
                shop: c.shop,
                slot: c.slot,
                eq_slot: 6,
                value: if c.kind == ARMOUR { 60_000 } else { 80_000 },
            })
            .collect()
    }

    /// The contract: a record replays to its own plan, solve and
    /// runner-up, from the line alone.
    #[test]
    fn a_record_replays_to_its_own_plan() {
        let s = stock();
        for gold in [0, 9, 60, 300, 2_000] {
            let (line, rec) = produce(&s, gold, &traits(), &armour_tryon(&s), &[Need::Food, Need::Poison]);
            assert!(replays(&line).unwrap(), "gold {gold}: {line}");
            let (_, out) = replay(&line).unwrap();
            assert_eq!(out.plan, rec.plan);
        }
    }

    /// The plan is not empty and buys what the deficits want: rations
    /// for the food gap, a second light for the depth, and the better
    /// armour rather than both (Q1).
    #[test]
    fn the_step_plans_what_the_deficits_want() {
        let s = stock();
        let (_, rec) = produce(&s, 2_000, &traits(), &armour_tryon(&s), &[]);
        assert!(!rec.plan.is_empty());
        let slot_picks: Vec<&PlanEntry> = rec.plan.iter().filter(|e| e.key_kind == 1).collect();
        assert_eq!(slot_picks.len(), 1, "one armour for one slot");
        // the try-on's power gain goes through the power need's curve:
        // the gap is a trip's preparation, 40000 x 5 = 200000, w_power 50:
        // 80000 points are worth 50*80000/200000 = 20
        assert_eq!(slot_picks[0].value, 20);
        assert!(rec.solve[3] & flag::GOLD_EXACT != 0);
    }

    /// v5 records carry their new units and parameters and replay through
    /// the utility path, while all historical fixtures below remain v4.
    #[test]
    fn utility_records_replay_and_reject_changed_ambition() {
        use crate::borg_town_utility::UNIT;
        let s = stock();
        let mut t = traits();
        t[crate::borg_headers::borg_trait_h::BI_CLEVEL as usize] = 10;
        t[crate::borg_headers::borg_trait_h::BI_MAXHP as usize] = 100;
        let (line, _) = produce(&s, 300, &t, &armour_tryon(&s), &[]);
        let (mut rec, parts) = replay_parts(&line).unwrap();
        rec.schema = 5;
        rec.params = params();
        for f in &mut rec.wares {
            f.reduction *= if f.need == 2 { 20 * UNIT } else { UNIT };
        }
        let o = step(&parts.input(&rec));
        rec.plan = o.plan;
        rec.solve = o.solve;
        rec.displaced = o.displaced;
        rec.dispose = o.dispositions.iter().map(|&d| dispose_code(d)).collect();
        let mut v5 = v2_prefix(&s, &t, 300).replacen("\"v\":4", "\"v\":5", 1);
        rec.push_fields(&mut v5);
        v5.push('}');
        assert!(replays(&v5).unwrap());
        let parsed = PlanRecord::from_json(&Json::parse(&v5).unwrap()).unwrap();
        assert_eq!(parsed, rec);
        let before = format!("\"params\":[{}]", rec.params.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","));
        rec.params[13] = 100_000;
        let after = format!("\"params\":[{}]", rec.params.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","));
        assert!(!replays(&v5.replacen(&before, &after, 1)).unwrap());
    }

    /// A record whose inputs were tampered with does not replay to its
    /// plan: replay is a real check, not an echo.
    #[test]
    fn a_tampered_record_does_not_replay() {
        let s = stock();
        let (line, _) = produce(&s, 300, &traits(), &armour_tryon(&s), &[]);
        // the 1x rations now cost 40
        let tampered = line.replacen("[1,2,0,20,4,1]", "[1,2,0,20,40,1]", 1);
        assert_ne!(tampered, line, "the ware was found");
        assert!(!replays(&tampered).unwrap());
        // so does a different arm
        let (rec, _) = replay(&line).unwrap();
        let mut p = rec.params;
        p[1] = 0; // w_light
        let mut l2 = line.clone();
        let from = format!("\"params\":[{}]", rec.params[..12].iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","));
        l2 = l2.replacen(&from, &format!("\"params\":[{}]", p[..12].iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")), 1);
        assert!(!replays(&l2).unwrap());
    }

    /// The v4 fields round-trip through the reader exactly.
    #[test]
    fn the_fields_round_trip() {
        let s = stock();
        let (line, rec) = produce(&s, 300, &traits(), &armour_tryon(&s), &[Need::Food]);
        let back = PlanRecord::from_json(&Json::parse(&line).unwrap()).unwrap();
        assert_eq!(back, rec);
    }

    /// P13's stable field order: v2's fields first, unchanged, then the
    /// plan's, in this order; and no key appears twice.
    #[test]
    fn the_field_order_is_fixed() {
        let s = stock();
        let (line, _) = produce(&s, 300, &traits(), &armour_tryon(&s), &[]);
        let j = Json::parse(&line).unwrap();
        let keys = j.keys();
        let v4 = [
            "mode", "visit", "step", "gturn", "power", "spells", "params", "limits", "caps", "urgent", "unmet",
            "wares", "tryon", "kit", "dispose", "plan", "solve", "paction", "act", "for", "now", "displaced", "guard",
        ];
        assert_eq!(&keys[keys.len() - v4.len()..], v4);
        assert_eq!(&keys[..2], ["v", "seq"]);
        let mut sorted = keys.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), keys.len(), "a key appears twice");
    }

    /// The action codes: v2's five unchanged, the plan's three after them.
    #[test]
    fn the_action_codes_extend_v2s() {
        let codes = [
            action::VISIT,
            action::BUY,
            action::TAKE_HOME,
            action::SELL,
            action::SELL_HOME,
            action::STORE,
            action::DROP,
            action::LEAVE,
        ];
        assert_eq!(codes, [0, 1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn live_execution_frees_a_full_home_before_attempting_a_deposit() {
        let stock = stock();
        let (line, _) = produce(&stock, 300, &traits(), &[], &[]);
        let (_, mut out) = replay(&line).unwrap();
        let pack = KitItem { joins_pack: false, place: Place::Pack, slot: 0, kind: 1, qty: 1,
            weight: 1, need: -1, reduction: 0, power: 0, proceeds: 0,
            sell_shop: 4, unknown: false, unknown_flavour: false,
            just_bought: false, serves: 0 };
        let home = KitItem { joins_pack: false, place: Place::Home, slot: 5, ..pack };
        let kit = [pack, home];
        out.dispositions = vec![Disposition::Store, Disposition::Sell];
        assert_eq!(first_executable_action(&stock.candidates, &[], &out, &[], &kit, 0, 1).0, action::SELL_HOME);
        assert_eq!(first_executable_action(&stock.candidates, &[], &out, &[], &kit, 1, 1).0, action::STORE);
        assert_eq!(first_action(&stock.candidates, &[], &out, &[], &kit).0, action::STORE,
            "historical shadow action order remains reproducible");
    }

    /// The step's solution and groups are the ones its plan and solve
    /// rows were read from.
    #[test]
    fn the_step_output_agrees_with_itself() {
        let s = stock();
        let (line, _) = produce(&s, 300, &traits(), &armour_tryon(&s), &[]);
        let (rec, out) = replay(&line).unwrap();
        assert_eq!(out.solution.value, rec.solve[0]);
        assert_eq!(out.solution.bound, rec.solve[1]);
        let chosen = out.solution.chosen.iter().filter(|&&o| o != 0).count();
        assert_eq!(chosen, rec.plan.len());
        assert!(out.groups.len() >= chosen);
    }

    /// The step's action: an urgent need before anything else; else the plan's
    /// first ranked purchase; else leave.
    #[test]
    fn the_first_action_is_the_urgent_then_the_plan_then_leave() {
        let s = stock();
        let wares = |s: &Stock| -> Vec<WareFact> {
            s.candidates
                .iter()
                .map(|c| {
                    let (need, reduction, serves) = role(c.kind);
                    WareFact { shop: c.shop, slot: c.slot, kind: c.kind, class: c.class, slots_cost: 1, weight: 10, need, reduction, serves }
                })
                .collect()
        };
        let run = |gold: i64, unmet: &[Need]| {
            let (line, _) = produce(&s, gold, &traits(), &[], unmet);
            let (rec, out) = replay(&line).unwrap();
            let idx: BTreeMap<(u8, u8), usize> = s.candidates.iter().enumerate().map(|(i, c)| ((c.shop, c.slot), i)).collect();
            let urgent: Vec<UrgentPurchase> = rec
                .urgent
                .iter()
                .map(|&(sh, sl, n)| UrgentPurchase {
                    candidate: idx[&(sh, sl)],
                    needs: Need::ALL.iter().enumerate().filter(|(b, _)| n & (1 << b) != 0).map(|(_, &x)| x).collect(),
                })
                .collect();
            first_action(&s.candidates, &urgent, &out, &wares(&s), &rec.kit)
        };
        // the food urgent need unmet: the urgent purchase comes first
        let (a, act, need, now) = run(300, &[Need::Food]);
        assert_eq!((a, need, now), (action::BUY, 1, true));
        assert_eq!(act[0], 0, "the general store's ration");
        // no urgent need: the plan's first purchase
        let (a, _, _, now) = run(300, &[]);
        assert_eq!((a, now), (action::BUY, false));
        // nothing affordable: leave
        let (a, act, _, _) = run(0, &[]);
        assert_eq!((a, act), (action::LEAVE, [-1, -1, -1, -1]));
    }

    /// The light need counts light sources: the replay reads them from
    /// v2's pack and equipment rows, so a line with a spare torch in the
    /// pack prices one light less than one without.
    #[test]
    fn the_replay_counts_the_carried_lights() {
        let s = stock();
        let (line, _) = produce(&s, 2_000, &traits(), &[], &[]);
        let (_, out) = replay(&line).unwrap();
        let spare = line.replacen("\"pack\":[]", "\"pack\":[[0,19,0,1,0,1]]", 1);
        assert_ne!(spare, line);
        let (_, out2) = replay(&spare).unwrap();
        let light = |o: &StepOutput| o.plan.iter().find(|e| e.key_kind == 0 && e.key == 0).map(|e| e.value);
        assert_eq!(light(&out), Some(100), "one torch of two: the second is worth the whole gap");
        assert_eq!(light(&out2), None, "two carried: light is met");
    }

    /// Every record of a real run replays to its own plan. Run by hand on
    /// a file written with `BORG_TOWN_PLAN_RECORD`:
    /// `PLAN_RECORD_FILE=... cargo test --release -- --ignored --nocapture
    /// a_run_s_records_all_replay`.
    #[test]
    #[ignore]
    fn a_run_s_records_all_replay() {
        let path = std::env::var("PLAN_RECORD_FILE").expect("PLAN_RECORD_FILE");
        let text = std::fs::read_to_string(&path).unwrap();
        let (mut n, mut planned, mut urgent) = (0, 0, 0);
        for (i, line) in text.lines().enumerate() {
            let (rec, out) = replay(line).unwrap_or_else(|e| panic!("line {}: {e}", i + 1));
            assert_eq!(out.plan, rec.plan, "line {}: plan", i + 1);
            assert_eq!(out.solve, rec.solve, "line {}: solve", i + 1);
            assert_eq!(out.displaced, rec.displaced, "line {}: runner-up", i + 1);
            let dispose: Vec<u8> = out.dispositions.iter().map(|&d| dispose_code(d)).collect();
            assert_eq!(dispose, rec.dispose, "line {}: dispositions", i + 1);
            n += 1;
            planned += usize::from(!rec.plan.is_empty());
            urgent += usize::from(rec.now);
        }
        assert!(n > 0, "no records");
        println!("replayed {n} records: {planned} with a plan, {urgent} taking an urgent purchase first");
    }

    /// For the calibration demo: replay every record of a run and
    /// **simulate the visit** the plan would make - take the first ranked
    /// purchase, apply it, re-solve, until the plan says leave - so the
    /// demo shows what would be bought, not the one-solve surrogate set
    /// (which prices each kind against the whole gap, Q3). Applying a
    /// purchase: food adds to BI_FOOD, a light source to the carried
    /// lights, Phase Door to BI_APHASE, cures to their counts; a wearable
    /// adds its try-on gain to the power and drops the other try-ons of
    /// its slot (they were measured against the old gear). The gold, the
    /// copies, the slots and the weight are spent. Run by hand:
    /// `PLAN_RECORD_FILE=... PLAN_PICKS_OUT=... cargo test --release --
    /// --ignored export_visits`.
    #[test]
    #[ignore]
    fn export_visits() {
        use crate::borg_headers::borg_trait_h::{BI_ACLW, BI_ACUREPOIS, BI_APHASE};
        use std::fmt::Write as _;
        let path = std::env::var("PLAN_RECORD_FILE").expect("PLAN_RECORD_FILE");
        let out_path = std::env::var("PLAN_PICKS_OUT").expect("PLAN_PICKS_OUT");
        let mut out = String::new();
        for line in std::fs::read_to_string(&path).unwrap().lines() {
            let (rec, mut parts) = replay_parts(line).unwrap();
            assert_eq!(rec.schema, 4, "export_visits simulates legacy v4 only; use the v5 shadow records directly");
            let mut facts = rec.wares.clone();
            facts.sort_by_key(|f| (f.shop, f.slot));
            let first = step(&parts.input(&rec));
            let mut bought: Vec<String> = Vec::new();
            for _ in 0..40 {
                let o = step(&parts.input(&rec));
                let Some(&(g, opt)) = o.solution.ranked(&o.groups).first() else { break };
                let choice = o.groups[g].options[opt].clone();
                let wearable = matches!(o.groups[g].key, GroupKey::Slot(_));
                let mut ps = Vec::new();
                for &(ci, k) in &choice.picks {
                    let f = facts[ci];
                    parts.candidates[ci].qty -= k;
                    match f.need {
                        0 => parts.carried.push((crate::headers::list_tvals_h::TV_LIGHT as u8, k)),
                        1 => parts.traits[BI_FOOD as usize] += k * f.reduction as i32,
                        2 => parts.traits[BI_ACLW as usize] += k,
                        3 => parts.traits[BI_ACUREPOIS as usize] += k,
                        4 => parts.traits[BI_APHASE as usize] += k,
                        _ => {}
                    }
                    if wearable {
                        if let Some(t) = parts.tryon.iter().find(|t| t.shop == f.shop && t.slot == f.slot).copied() {
                            parts.power += t.value;
                            parts.tryon.retain(|x| x.eq_slot != t.eq_slot);
                        }
                    }
                    ps.push(format!("[{},{},{}]", f.shop, f.slot, k));
                }
                parts.limits.gold -= choice.cost;
                parts.limits.pack_slots -= choice.slots;
                parts.limits.weight -= choice.weight;
                bought.push(format!(
                    "{{\"kind\":{},\"picks\":[{}],\"cost\":{},\"value\":{}}}",
                    i32::from(wearable),
                    ps.join(","),
                    choice.cost,
                    choice.value
                ));
            }
            writeln!(
                out,
                "{{\"turn\":{},\"bought\":[{}],\"one_solve_value\":{},\"one_solve_options\":{}}}",
                rec.gturn,
                bought.join(","),
                first.solution.value,
                first.plan.len()
            )
            .unwrap();
        }
        std::fs::write(&out_path, out).unwrap();
    }

    /// Two stages (slice 3): the dispositions first, the purchases with
    /// what they free. A worthless stack is sold (its slot freed), the
    /// Home's rations are taken back (their food closes part of the gap,
    /// so the purchases buy less food), and the record replays its
    /// dispositions as well as its plan.
    #[test]
    fn the_step_disposes_then_buys() {
        let s = stock();
        let junk = KitItem { joins_pack: false,
            serves: 0,
            place: Place::Pack,
            slot: 4,
            kind: 99,
            qty: 1,
            weight: 50,
            need: -1,
            reduction: 0,
            power: 0,
            proceeds: 7,
            sell_shop: 0,
            unknown: false,
            unknown_flavour: false,
            just_bought: false,
        };
        let rations = KitItem { joins_pack: false, place: Place::Home, slot: 0, kind: RATION, qty: 5, weight: 10, need: 1, reduction: 1, proceeds: 0, ..junk };
        let (line, rec) = produce_kit(&s, 300, &traits(), &[], &[], &[junk, rations]);
        assert_eq!(rec.dispose, [2, 4], "the junk sold, the rations taken");
        assert!(replays(&line).unwrap(), "the dispositions replay");
        let (_, out) = replay(&line).unwrap();
        assert_eq!((out.freed.pack_slots, out.freed.gold), (0, 7), "one slot freed by the sale, one used by the take");
        // the rations taken close 5 of the 7-ration gap: the purchases'
        // food is 2 rations' worth, not 7
        let (_, no_kit) = produce(&s, 300, &traits(), &[], &[]);
        let food = |r: &PlanRecord| r.plan.iter().filter(|e| e.key == 1 || e.key == 6).map(|e| e.copies).sum::<i32>();
        assert!(food(&rec) < food(&no_kit), "{} rations planned with the Home's, {} without", food(&rec), food(&no_kit));
        // the first action is the sale, before any purchase
        let urgent: Vec<UrgentPurchase> = Vec::new();
        let (a, act, _, _) = first_action(&s.candidates, &urgent, &out, &rec.wares, &rec.kit);
        assert_eq!((a, act[1]), (action::SELL, 4));
    }

    /// Buy to identify: an unknown ware's first copy is worth the
    /// knowledge weight on top of its need, a second copy nothing more;
    /// the plan buys one of an otherwise unwanted unknown ware.
    #[test]
    fn an_unknown_ware_is_bought_once_to_learn_it() {
        let s = stock();
        let mut t = traits();
        t[BI_FOOD as usize] = 10; // no food gap: rations serve nothing
        let rations = |r: &PlanRecord| r.plan.iter().filter(|e| e.key == 1 || e.key == 6).map(|e| e.copies).sum::<i32>();
        let (_, known) = produce(&s, 2_000, &t, &[], &[]);
        assert_eq!(rations(&known), 0, "a known ration with no gap is not bought");
        let (line, unseen) = produce_ident(&s, 2_000, &t, &[], &[], &[], &[RATION]);
        assert_eq!(rations(&unseen), 1, "one, to learn the flavour: {:?}", unseen.plan);
        assert!(replays(&line).unwrap(), "the knowledge bonus replays");
    }

    #[test]
    fn historical_traits_replay_into_m13_without_changing_the_record() {
        let s=stock();let (line,_)=produce(&s,300,&traits(),&[],&[]);
        assert!(line.contains("[106,10]"));
        let (_,parts)=replay_parts(&line).unwrap();
        assert_eq!(parts.traits[BI_MAXDEPTH as usize],10);
        assert_eq!(parts.traits[crate::borg_headers::borg_trait_h::BI_CARRY as usize],2000);
        let added=line.replacen("\"traits\":[","\"traits\":[[265,1],",1);
        let (_,parts)=replay_parts(&added).unwrap();
        assert_eq!(parts.traits[crate::borg_headers::borg_trait_h::BI_PSTUN as usize],0,"legacy records cannot masquerade as new M13 fields");
    }

    /// A version-2 line is not read as a plan record.
    #[test]
    fn a_v2_line_is_refused() {
        let j = Json::parse("{\"v\":2,\"seq\":0}").unwrap();
        assert!(PlanRecord::from_json(&j).is_err());
    }

    /// The reader takes what the writer writes, and nothing else.
    #[test]
    fn the_reader_rejects_malformed_input() {
        assert!(Json::parse("{\"a\":[1,-2,[3]]}").is_ok());
        assert!(Json::parse("{\"a\":1,}").is_err());
        assert!(Json::parse("{\"a\":1} x").is_err());
        assert!(Json::parse("[1,2").is_err());
    }
    #[test]
    fn a_blocked_urgent_purchase_does_not_hide_a_feasible_cure() {
        let stock=stock();let (line,_)=produce(&stock,300,&traits(),&[],&[]);
        let (_,mut out)=replay(&line).unwrap();out.solution.chosen.fill(0);
        let mut cs=stock.candidates.clone();cs[0].slots_cost=1;cs[1].slots_cost=0;
        let urgent=[UrgentPurchase {candidate:0,needs:vec![Need::Food]},
            UrgentPurchase {candidate:1,needs:vec![Need::Poison]}];
        let chosen=first_executable_action(&cs,&urgent,&out,&[],&[],1,0);
        assert_eq!(chosen,(action::BUY,[i64::from(cs[1].shop),i64::from(cs[1].slot),1,i64::from(cs[1].price)],3,true));
        // When room exists the original urgent order still wins.
        assert_eq!(first_executable_action(&cs,&urgent,&out,&[],&[],1,1).1[1],i64::from(cs[0].slot));
        cs[1].shop=7;
        assert_eq!(first_executable_action(&cs,&urgent,&out,&[],&[],1,0).0,action::TAKE_HOME);
        cs[1].slots_cost=1;
        assert_eq!(first_executable_action(&cs,&urgent,&out,&[],&[],1,0).0,action::LEAVE);
    }

    #[test]
    fn full_pack_frees_room_before_urgent_buy_and_does_not_queue_an_impossible_buy() {
        let stock=stock();let (line,_)=produce(&stock,300,&traits(),&[],&[]);
        let (_,mut out)=replay(&line).unwrap();
        let pack=KitItem {joins_pack:false,place:Place::Pack,slot:0,kind:1,qty:1,weight:1,need:-1,reduction:0,power:0,
            proceeds:0,sell_shop:4,unknown:false,unknown_flavour:false,just_bought:false,serves:0};
        let kit=[pack];out.dispositions=vec![Disposition::Sell];
        let urgent=[UrgentPurchase {candidate:0,needs:vec![Need::Food]}];
        assert_eq!(first_executable_action(&stock.candidates,&urgent,&out,&[],&kit,1,0).0,action::SELL);
        out.dispositions[0]=Disposition::Keep;out.solution.chosen.fill(0);
        assert_eq!(first_executable_action(&stock.candidates,&urgent,&out,&[],&kit,1,0).0,action::LEAVE);
        assert_eq!(first_executable_action(&stock.candidates,&urgent,&out,&[],&kit,1,1).0,action::BUY);
        let mut cs=stock.candidates.clone();cs[0].shop=7;cs[0].slots_cost=0;
        assert_eq!(first_executable_action(&cs,&urgent,&out,&[],&kit,1,0).0,action::TAKE_HOME);
    }

}
