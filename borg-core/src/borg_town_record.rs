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
/// handover's state record). borg-reborn-rs (#651): 12, what a visit
/// bought is not stored in the Home (`borg_town_utility::dispositions`).
/// borg-reborn-rs (#957, P7 review): 13, the record carries the storing
/// settings (`storing`: the departure room, `town_store_keep_min` and
/// `town_store_only_overweight`), so a replay stores as the visit did.
/// #1048: v14 records the Home stack limit and the eligible kit kinds.
/// V15 records native Home room and disposal of known unwanted stock without a buyer.
/// V16 permits partial surplus disposal. V17 separates Home cleanup from
/// purchase history while the carried reserve is covered. V18 defers a
/// surplus withdrawal if a shop purchase would block the pack sale.
/// V19 defers protected Detect Invisible stock unless overflow can sell
/// every copy withdrawn.
/// V20 (#1151) splits a fuel stack at the light reserve, lets a known-cursed
/// spare that adds no power go while the pack is over its allowance, and
/// does not charge the weight of a Remove Curse read in town.
/// V21 overload records (nine ware fields) and Home records (ten) were
/// produced by parallel branches. V22–23 are Home-only. V24 combines both.
pub const PLAN_SCHEMA: i64 = 24;

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
    pub const DROP_HOME: i64 = 8;
    pub const BUY_FOR_HOME: i64 = 9;
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
    /// Value as a reserve, or -1 for a kind just cleared as surplus.
    pub home_value: i64,
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
    /// Schema 21 was used by both the independent Home and loadout branches.
    pub legacy_home_21: bool,
    /// 0 shadow (computed, not executed), 1 live.
    /// 0 is shadow, 1 is live shopping, and 2 is live departure.
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
    pub black_market_reserve: i64,
    pub black_market_percentage: i64,
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
    /// v13 (#957) stores six values; v14 adds the per-kind Home limit.
    pub storing: Option<[i64; 7]>,
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
/// Bit 8 marks the stack reserved for a class book; older records leave it clear.
fn kit_row(k: &KitItem, schema: i64) -> [i64; 12] {
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
        i64::from(k.unknown) | i64::from(k.unknown_flavour) << 1 | i64::from(k.just_bought) << 2 | i64::from(k.joins_pack) << 3 | i64::from(k.protect_disposal) << 6 | i64::from(k.home_limited) << 4 | i64::from(k.home_unwanted) << 5 | i64::from(k.cursed_spare) << 7 | i64::from(k.keep_last_book) << 8
            | (if schema >= 21 { k.home_role as i64 } else { 0 }) << 9,
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
        if let Some(storing) = self.storing.filter(|_| self.schema >= 13) {
            field(out, "storing");
            ints(out, &storing[..if self.schema == 13 { 6 } else { 7 }]);
        }
        if self.black_market_reserve != 0 {
            field(out, "black_market_reserve");
            num(out, self.black_market_reserve);
        }
        if self.black_market_percentage >= 0 {
            field(out, "black_market_percentage");
            num(out, self.black_market_percentage);
        }
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
                    w.home_value,
                ][..if self.schema >= 22 || self.legacy_home_21 { 10 } else { 9 }],
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
        arr(out, &self.kit, |o, k| {
            let row = kit_row(k, self.schema);
            if self.schema >= 15 {
                let room = k.home_room.unwrap_or([-1; 3]).map(i64::from);
                let mut fields = [0; 16];
                fields[..12].copy_from_slice(&row);
                fields[12..15].copy_from_slice(&room);
                // borg-reborn-rs (#1151): v20 adds the physical fuel per copy.
                fields[15] = k.fuel;
                ints(o, &fields[..if self.schema >= 20 { 16 } else { 15 }]);
            } else {
                ints(o, &row[..if self.schema == 4 { 11 } else { 12 }]);
            }
        });
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
        if !(4..=PLAN_SCHEMA).contains(&schema) {
            return Err("not a supported plan record".into());
        }
        let raw_params = j.get("params")?.ints()?;
        if raw_params.len() != if schema == 4 { 12 } else { PARAMS } { return Err("params has the wrong length".into()); }
        let mut params = [0; PARAMS];
        params[..raw_params.len()].copy_from_slice(&raw_params);
        let small = |k: &str| -> Result<i64, String> { j.get(k)?.int() };
        let rows = |k: &str| -> Result<Vec<Vec<i64>>, String> { j.get(k)?.arr()?.iter().map(Json::ints).collect() };
        let wares = rows("wares")?;
        let kit = rows("kit")?;
        let legacy_home_21 = schema == 21 && (wares.first().is_some_and(|row| row.len() == 10)
            || wares.is_empty() && kit.iter().any(|row| row.get(10).is_some_and(|flags| flags & (7 << 9) != 0)));
        let home_facts = schema >= 22 || legacy_home_21;
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
            legacy_home_21,
            mode: small("mode")? as u8,
            visit: small("visit")? as u64,
            step: small("step")? as u64,
            gturn: small("gturn")?,
            power: small("power")?,
            spells: small("spells")? as u8,
            params,
            storing: match schema {
                13 => {
                    let [a, b, c, d, e, f] = fixed(j.get("storing")?.ints()?, "storing")?;
                    Some([a, b, c, d, e, f, 0])
                }
                14.. => Some(fixed(j.get("storing")?.ints()?, "storing")?),
                _ => None,
            },
            black_market_reserve: j.get("black_market_reserve").map_or(Ok(0), Json::int)?,
            black_market_percentage: j.get("black_market_percentage").map_or(Ok(-1), Json::int)?,
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
            wares: wares
                .into_iter()
                .map(|r| {
                    if r.len() != if home_facts { 10 } else { 9 } {
                        return Err("wrong ware row length".to_string());
                    }
                    match r[..9] {
                    [shop, slot, kind, class, slots_cost, weight, need, reduction, serves] => Ok(WareFact {
                        home_value: r.get(9).copied().unwrap_or(0),
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
                    _ => unreachable!(),
                    }
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
            kit: kit
                .into_iter()
                .map(|r| {
                    if r.len() != match schema { 4 => 11, 5..=14 => 12, 15..=19 => 15, _ => 16 } { return Err("wrong kit row length".into()); }
                    let serves = if schema == 4 { 0 } else { r[11] as u8 };
                    match r[..11] {
                    [place, slot, kind, qty, weight, need, reduction, power, proceeds, sell_shop, flags] => Ok(KitItem {
                        home_role: if home_facts {
                            crate::borg_town_home::Role::from_code((flags >> 9) & 7)
                        } else { Default::default() },
                        home_room: if schema >= 15 && r[12] >= 0 {
                            Some([r[12] as i32, r[13] as i32, r[14] as i32])
                        } else { None },
                        keep_last_book: flags & 256 != 0,
                        protect_disposal: schema >= 14 && flags & 64 != 0,
                        home_limited: schema >= 14 && flags & 16 != 0,
                        home_unwanted: schema >= 14 && flags & 32 != 0,
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
                        cursed_spare: schema >= 20 && flags & 128 != 0,
                        fuel: if schema >= 20 { r[15] } else { 0 },
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
    /// Shopping is finished. Keep urgent needs available while shedding surplus.
    pub departure: bool,
    pub shop_overloaded: bool,
    pub revision: i64,
    pub capability: Option<&'a crate::borg_town_capability::Model>,
    pub utility: bool,
    /// The candidates, in stock order.
    pub candidates: Vec<Candidate>,
    pub facts: &'a [WareFact],
    pub tryon: &'a [TryOn],
    pub limits: Limits,
    pub black_market_reserve: i64,
    pub black_market_percentage: i64,
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
    /// borg-reborn-rs (#1151, schema 20): town enchanting would read a
    /// scroll now (`borg_item_enchant::reads_scroll_in_town`); recorded as
    /// `spells` bit 1.
    pub reads_now: bool,
    /// The pack's and the Home's stacks, for the dispositions.
    pub kit: &'a [KitItem],
    /// The Home's free slots.
    pub home_slots: i32,
    /// #957: the storing settings a record carries (v13); None reads the
    /// process's (a live solve, an older record).
    pub storing: Option<crate::borg_town_utility::Storing>,
}

/// What a step produces.
#[derive(Clone, Debug)]
pub struct StepOutput {
    revision: i64,
    /// The first planned purchase that fills an outstanding ranged reserve.
    pub ranged_purchase: Option<(usize, i32)>,
    pub home_purchase: Option<usize>,
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
    /// Per kit row, explicit quantities for split and short-kit moves.
    /// A zero or absent quantity denotes the whole stack.
    pub copies: Vec<i32>,
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
    utility.recall_reserve = step_recall_reserve(input.storing, input.limits.gold);
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
    let (mut dispositions, mut freed, copies) = if input.utility {
        crate::borg_town_utility::dispositions(input, &mut utility, &st)
    } else {
        let (d, f) = dispose(
            input.kit,
            &st,
            &trip,
            &weights,
            input.limits.pack_slots,
            input.home_slots,
            input.limits.weight,
        );
        (d, f, Vec::new())
    };
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
    // Stage 2: the purchases, with what the dispositions freed. They leave
    // the departure room free (#859; zero when off).
    let room = if input.utility {
        input.storing.map_or_else(crate::borg_town_utility::departure_room, |s| s.room)
    } else { Default::default() };
    let limits = Limits {
        gold: input.limits.gold + freed.gold,
        pack_slots: input.limits.pack_slots + freed.pack_slots - room.slots,
        weight: if input.shop_overloaded { i64::MAX } else { input.limits.weight + freed.weight - room.weight },
    };
    let st = st;
    let facts: BTreeMap<(u8, u8), &WareFact> = input.facts.iter().map(|f| ((f.shop, f.slot), f)).collect();
    let tryon: BTreeMap<(u8, u8), &TryOn> = input.tryon.iter().map(|t| ((t.shop, t.slot), t)).collect();
    let mut candidates = input.candidates.clone();
    if input.utility {
        let saturated = crate::borg_town_utility::saturated_home_kinds(input, &utility, &dispositions, &copies);
        for c in &mut candidates {
            if input.departure || (input.revision >= 9 && c.shop == 7) || saturated.contains(&c.kind)
                || input.revision >= 21 && facts.get(&(c.shop, c.slot)).is_some_and(|w| w.home_value < 0)
            { c.qty = 0; }
            // borg-reborn-rs (#1151, plan schema 20): a Remove Curse scroll
            // bought for a curse the Borg has is read in town, so the weight
            // allowance does not price it. Overweight, the solver's weight
            // repair dropped even this 0.5 lb scroll while a sickliness curse
            // took 5 STR off the allowance. Review of #1155: only while it can
            // be read in town (`reads_now`: `borg_enchanting`'s and
            // `borg_read_scroll`'s gates), and only the copies read now; spares are bought by a later
            // solve, at their carried weight.
            if input.revision >= 20 && input.reads_now && u32::from(c.tval) == crate::headers::list_tvals_h::TV_SCROLL {
                let now = input.capability.map_or(0, |model| model.decurse_copies(c.shop, c.slot));
                if now > 0 {
                    c.weight = 0;
                    c.qty = c.qty.min(now);
                }
            }
            if !tryon.contains_key(&(c.shop, c.slot)) && input.kit.iter().zip(&dispositions).enumerate().any(|(i, (it, d))|
                it.place == Place::Pack && it.kind == c.kind && *d != Disposition::Keep
                    && !((input.capability.is_some_and(|model| model.restore_life_keep(it) > 0
                            || input.revision >= 20 && model.reserve_keep(it) > 0)
                        || it.home_limited && input.storing.is_some_and(|storing| storing.home_stack_limit > 0))
                        && copies.get(i).is_some_and(|&qty| qty > 0 && qty < it.qty))) {
                c.slots_cost = 1;
            }
        }
    }
    let stock = Stock { candidates, ..Stock::default() };
    let budget = Budget {
        gold: limits.gold,
        black_market_reserve: input.black_market_reserve,
        black_market_percentage: input.black_market_percentage,
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
    let ranked = solution.ranked(&groups);
    let plan: Vec<PlanEntry> = ranked.iter().map(|&(g, o)| plan_entry(&groups[g], o)).collect();
    let ranged_purchase = utility.capability.as_ref().and_then(|model| {
        ranked.iter().flat_map(|&(g, o)| &groups[g].options[o].picks).find_map(|&(i, copies)| {
            let c = &input.candidates[i];
            model.ranged_reserve_copies(c.shop, c.slot, copies).map(|n| (i, n))
        })
    });
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
    let mut home_purchase = if input.revision >= 21 && plan.is_empty()
        && (input.home_slots > 0 || input.revision >= 23) && input.limits.pack_slots > 0
    {
        reserve_purchase(input, &stock.candidates, limits.gold)
    } else { None };
    if input.revision >= 23 && input.home_slots <= 0 {
        if let Some(i) = home_purchase.take().filter(|_| freed.home_slots <= 0 && input.urgent.is_empty()) {
            let c = &stock.candidates[i];
            let rank = crate::borg_town_home::Role::offer_rank(facts[&(c.shop, c.slot)].home_value);
            // Make room before buying. A fresh solve will check the actual
            // vacancy, pack room, purse and offer after the withdrawal.
            if rank >= crate::borg_town_home::Role::Scarce.rank() {
                if let Some(j) = crate::borg_town_utility::home_replacement(input, &utility, &dispositions, c.kind, rank, 0) {
                    let item = &input.kit[j];
                    dispositions[j] = if item.sell_shop >= 0 { Disposition::Sell } else { Disposition::Drop };
                    freed.home_slots += 1;
                    freed.gold += item.proceeds;
                }
            }
        }
    }
    StepOutput { revision: input.revision, ranged_purchase, home_purchase, groups, solution, plan, solve: solve_row, displaced, dispositions, freed, copies }
}

/// Reserve purchases use the ordinary purse and Black Market cap. They are
/// considered after supplies for this trip and require real temporary room.
fn reserve_purchase(input: &StepInput, candidates: &[Candidate], gold: i64) -> Option<usize> {
    candidates.iter().enumerate().filter_map(|(i, c)| {
        let price = i64::from(c.price);
        if c.shop >= 7 || c.qty == 0 || price <= 0 || price > gold
            || i64::from(c.weight) > input.limits.weight
            || !crate::borg_town_budget::urgent_market_allows(
                c, gold, input.black_market_reserve, input.black_market_percentage)
        {
            return None;
        }
        let fact = input.facts.iter().find(|w| w.shop == c.shop && w.slot == c.slot)?;
        (fact.home_value > 0).then_some(((fact.home_value, -c.weight, -c.price), i))
    }).max_by_key(|&(score, i)| (score, std::cmp::Reverse(i))).map(|(_, i)| i)
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
    first_action_with_room(candidates, urgent, out, facts, kit, None, None, None)
}

/// Respect a full Home's prerequisite: free a Home slot before depositing.
/// Historical shadow records retain their original order through first_action.
/// `weight_room`: the tenth-pounds still free before the encumbrance starts
/// (`Facts::allowance`, `BI_CARRY / 2 - BI_WEIGHT`).
pub fn first_executable_action(
    candidates: &[Candidate], urgent: &[UrgentPurchase], out: &StepOutput,
    facts: &[WareFact], kit: &[KitItem], home_slots: i32, pack_slots: i32, weight_room: i64,
) -> (i64, [i64; 4], i64, bool) {
    first_action_with_room(candidates, urgent, out, facts, kit, Some(home_slots), Some(pack_slots), Some(weight_room))
}

/// One copy of an urgent ware fits the pack's free slots and the weight
/// still free (#955); `None` is not checked (historical selection).
fn urgent_fits(c: &Candidate, pack_slots: Option<i32>, weight_room: Option<i64>) -> bool {
    pack_slots.is_none_or(|n| n >= c.slots_cost) && weight_room.is_none_or(|w| w >= i64::from(c.weight))
}

fn planned_purchase(out: &StepOutput) -> Option<(usize, i32)> {
    out.ranged_purchase.or_else(|| {
        let &(g, o) = out.solution.ranked(&out.groups).first()?;
        out.groups[g].options[o].picks.first().copied()
    })
}

/// If the offered purchase can already be carried, finish acquiring stock
/// before doing the weight-driven dispositions. A blocked urgent offer
/// keeps its slot-opening prerequisite.
pub(crate) fn acquisition_before_disposal(
    candidates: &[Candidate], urgent: &[UrgentPurchase], out: &StepOutput,
    facts: &[WareFact], kit: &[KitItem], pack_slots: i32,
) -> Option<(i64, [i64; 4], i64, bool)> {
    if !urgent.is_empty() { return None; }
    for (i, item) in kit.iter().enumerate() {
        if item.place != Place::Home || out.dispositions.get(i) != Some(&Disposition::Take) { continue; }
        let qty = out.copies.get(i).copied().filter(|&n| n > 0).unwrap_or(item.qty);
        let joins = item.home_room.map_or(item.joins_pack, |[_, merge, _]| qty <= merge);
        if pack_slots > 0 || joins {
            return Some((action::TAKE_HOME, [7, i64::from(item.slot), i64::from(qty), 0], i64::from(item.need), false));
        }
    }
    let (i, copies) = planned_purchase(out)?;
    let c = &candidates[i];
    if c.slots_cost > pack_slots { return None; }
    let need = facts.iter().find(|f| f.shop == c.shop && f.slot == c.slot).map_or(-1, |f| i64::from(f.need));
    Some((action::BUY, [i64::from(c.shop), i64::from(c.slot), i64::from(copies), i64::from(c.price)], need, false))
}

/// The live selection (#939 re-review, P2). The urgent pass picks its set by
/// gold, with the urgent Recall first (#956), before anything checks what
/// fits; the selector then skips an urgent copy that does not fit (#955) and
/// makes room for it when it can. When that leaves the visit with nothing to
/// do (`LEAVE`) because an urgent ware does not fit and no room can be made,
/// the urgent needs are chosen again without every ware that does not fit
/// now (`rebudget(unfit)`, review of #973: not only the blocked ones, or the
/// second pass can pick another ware that does not fit), so an affordable
/// cure that fits is bought rather than lost to a Recall the pack cannot
/// take. When the room can be made, or the blocked ware fits, nothing
/// changes. One retry at most.
#[allow(clippy::too_many_arguments)]
pub fn live_selection(
    candidates: &[Candidate], urgent: &[UrgentPurchase], out: &StepOutput, facts: &[WareFact], kit: &[KitItem],
    home_slots: i32, pack_slots: i32, weight_room: i64,
    rebudget: impl FnOnce(&[usize]) -> Vec<UrgentPurchase>,
) -> (i64, [i64; 4], i64, bool) {
    let first = first_executable_action(candidates, urgent, out, facts, kit, home_slots, pack_slots, weight_room);
    if first.0 != action::LEAVE { return first; }
    let fits = |c: &Candidate| urgent_fits(c, Some(pack_slots), Some(weight_room));
    if urgent.iter().all(|f| fits(&candidates[f.candidate])) { return first; }
    let unfit: Vec<usize> = candidates.iter().enumerate()
        .filter(|(_, c)| c.qty > 0 && !fits(c)).map(|(i, _)| i).collect();
    let others = rebudget(&unfit);
    if others.is_empty() { return first; }
    let second = first_executable_action(candidates, &others, out, facts, kit, home_slots, pack_slots, weight_room);
    if second.0 == action::LEAVE { first } else { second }
}

#[allow(clippy::too_many_arguments)]
fn first_action_with_room(
    candidates: &[Candidate], urgent: &[UrgentPurchase], out: &StepOutput,
    facts: &[WareFact], kit: &[KitItem], home_slots: Option<i32>, pack_slots: Option<i32>,
    weight_room: Option<i64>,
) -> (i64, [i64; 4], i64, bool) {
    let need_of = |c: &Candidate| {
        facts.iter().find(|f| f.shop == c.shop && f.slot == c.slot).map_or(-1, |f| i64::from(f.need))
    };
    // A blocked new stack must not hide a later emergency that joins stock
    // already carried. Historical shadow selection has no pack-room filter.
    // borg-reborn-rs (#955, P7 review): nor over the carry allowance. The
    // solver already priced the room the urgent copy needs (Budget::weight
    // is the allowance less the urgent picks), so when one copy does not fit
    // yet, the room-making dispositions below run first and the purchase
    // follows on a later solve.
    let feasible_urgent = urgent.iter().find(|f| urgent_fits(&candidates[f.candidate], pack_slots, weight_room));
    let blocked_urgent = !urgent.is_empty() && feasible_urgent.is_none();
    if let Some(f) = feasible_urgent {
        let c = &candidates[f.candidate];
        let need = f.needs.first().and_then(|n| Need::ALL.iter().position(|x| x == n)).map_or(-1, |i| i as i64);
        return (if c.shop == 7 { action::TAKE_HOME } else { action::BUY }, [i64::from(c.shop), i64::from(c.slot), 1, i64::from(c.price)], need, true);
    }
    // The dispositions before the purchases, in the order that frees room
    // first: the pack's sales and drops, then its stores, then the Home's
    // disposals, then what comes back from the Home to keep.
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
        let home_disposal = place == Place::Home && d == Disposition::Sell;
        let selected = kit.iter().enumerate().filter(|(i, k)| {
            let disposition = out.dispositions.get(*i);
            let copies = out.copies.get(*i).copied().filter(|&qty| qty > 0);
            let joins = k.home_room.zip(copies).map_or(k.joins_pack, |([_, merge, _], qty)| qty <= merge);
            k.place == place
                && (disposition == Some(&d) || home_disposal && disposition == Some(&Disposition::Drop))
                && !(place == Place::Pack && d == Disposition::Store && home_slots.is_some_and(|n| n <= 0))
                && !(place == Place::Home && pack_slots.is_some_and(|n| n <= 0) && !joins)
        }).min_by_key(|(i, k)| {
            let rank = if out.revision >= 23 && d == Disposition::Store {
                u8::MAX - k.home_role.rank()
            } else if out.revision >= 23 && home_disposal {
                k.home_role.rank()
            } else { 0 };
            (!(home_disposal && k.home_unwanted), rank, *i)
        });
        if let Some((i, k)) = selected {
            let (code, at) = match (place, out.dispositions[i]) {
                (Place::Pack, Disposition::Sell) => (action::SELL, i64::from(k.sell_shop)),
                (Place::Pack, Disposition::Drop) => (action::DROP, -1),
                (Place::Pack, Disposition::Store) => (action::STORE, 7),
                (Place::Home, Disposition::Sell) => (action::SELL_HOME, i64::from(k.sell_shop)),
                (Place::Home, Disposition::Drop) => (action::DROP_HOME, 7),
                _ => (action::TAKE_HOME, 7),
            };
            // Split storage, disposal and short-kit withdrawals use the
            // quantity the utility pass priced.
            let copies = out.copies.get(i).copied().filter(|&n| n > 0);
            let qty = i64::from(copies.unwrap_or(k.qty));
            let proceeds = if code == action::SELL_HOME && copies.is_some() {
                k.proceeds * qty / i64::from(k.qty)
            } else { k.proceeds };
            return (code, [at, i64::from(k.slot), qty, proceeds], i64::from(k.need), false);
        }
    }
    if blocked_urgent { return (action::LEAVE, [-1; 4], -1, false); }
    if let Some((i, copies)) = planned_purchase(out) {
        let c = &candidates[i];
        return (action::BUY, [i64::from(c.shop), i64::from(c.slot), i64::from(copies), i64::from(c.price)], need_of(c), false);
    }
    if let Some(i) = out.home_purchase {
        let c = &candidates[i];
        return (action::BUY_FOR_HOME, [i64::from(c.shop), i64::from(c.slot), 1, i64::from(c.price)], -1, false);
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

/// #957 (reviews of #962): the Word of Recall reserve a step values, with
/// the purse after the urgent purchases: a v13 record's reserve and gold
/// floor, else the process's settings (a live solve, an older record).
fn step_recall_reserve(storing: Option<crate::borg_town_utility::Storing>, gold: i64) -> i64 {
    match storing {
        Some(s) => crate::borg_town_capability::eligible_recall_reserve(gold, s.recall_floor, s.recall_reserve),
        None => crate::borg_town_capability::recall_reserve_for_budget(gold),
    }
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
        let shop_overloaded = rec.schema >= 24 || rec.schema == 21 && !rec.legacy_home_21;
        StepInput {
            departure: shop_overloaded && rec.mode == 2,
            shop_overloaded,
            revision: rec.schema,
            capability: rec.capability.as_ref(),
            utility: rec.schema >= 5,
            candidates: self.candidates.clone(),
            facts: &rec.wares,
            tryon: &self.tryon,
            limits: self.limits,
            black_market_reserve: rec.black_market_reserve,
            black_market_percentage: rec.black_market_percentage,
            urgent: self.urgent.clone(),
            params: rec.params,
            traits: &self.traits,
            power: self.power,
            carried: &self.carried,
            cure_spell: rec.spells & 1 != 0,
            reads_now: rec.schema >= 20 && rec.spells & 2 != 0,
            kit: &rec.kit,
            home_slots: rec.limits[2] as i32,
            storing: rec.storing.map(crate::borg_town_utility::Storing::from_array),
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
                    home_value: 0, shop: c.shop,
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
        let out = step(&StepInput { departure: false, shop_overloaded: false, storing: None,
            revision: 4,
            capability: None,
            utility: false,
            candidates: stock.candidates.clone(),
            facts: &wares,
            tryon,
            limits,
            black_market_reserve: 0,
            black_market_percentage: -1,
            urgent: budget.urgent.clone(),
            params: params(),
            traits,
            power: 1000,
            carried: &carried(),
            cure_spell: false, reads_now: false,
            kit,
            home_slots: budget.home_slots,
        });
        let mask = |ns: &[Need]| {
            ns.iter().fold(0u8, |m, n| m | (1 << Need::ALL.iter().position(|x| x == n).unwrap()))
        };
        let rec = PlanRecord {
            storing: None,
            capability: None,
            schema: 4,
            legacy_home_21: false,
            mode: 0,
            visit: 3,
            step: 1,
            gturn: 12345,
            power: 1000,
            spells: 0,
            params: { let mut p = params(); p[12..].fill(0); p },
            limits: [budget.gold, i64::from(budget.pack_slots), i64::from(budget.home_slots), budget.weight],
            black_market_reserve: 0,
            black_market_percentage: -1,
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

    /// #957 (P7 review): a v13 record writes the storing settings and a
    /// replay uses them, not the process's; an older record has none and
    /// keeps reading the process's settings, as before.
    #[test]
    fn a_recorded_reserve_and_floor_decide_the_step_s_recall_reserve() {
        use crate::borg_town_utility::Storing;
        let recorded = |floor| Some(Storing { recall_reserve: 2, recall_floor: floor, ..Storing::default() });
        assert_eq!(super::step_recall_reserve(recorded(200), 250), 2, "the purse reaches the recorded floor");
        assert_eq!(super::step_recall_reserve(recorded(400), 250), 0, "it does not: no reserve");
        // Without recorded settings, the process's (the registry's defaults
        // in a test): the same as before #957.
        assert_eq!(super::step_recall_reserve(None, 250), crate::borg_town_capability::recall_reserve_for_budget(250));
    }

    #[test]
    fn the_storing_settings_travel_with_a_v14_record() {
        use crate::borg_town_utility::{Room, Storing};
        let storing = Storing { only_overweight: false, room: Room { weight: 50, slots: 2 }, keep_min: 3, recall_reserve: 2, recall_floor: 200, home_stack_limit: 2 };
        assert_eq!(Storing::from_array(storing.to_array()), storing);
        let stock = stock();
        let (_, mut rec) = produce(&stock, 300, &traits(), &[], &[]);
        let mut line = String::new();
        rec.push_fields(&mut line);
        assert!(!line.contains("\"storing\""), "an older record carries none");
        rec.schema = PLAN_SCHEMA;
        rec.storing = Some(storing.to_array());
        let mut line = String::new();
        rec.push_fields(&mut line);
        assert!(line.contains("\"storing\":[50,2,3,0,2,200,2]"), "{line}");
        let parts = ReplayParts { candidates: Vec::new(), urgent: Vec::new(), traits: Vec::new(), carried: Vec::new(),
            limits: Limits { gold: 0, pack_slots: 0, weight: 0 }, tryon: Vec::new(), power: 0 };
        assert_eq!(parts.input(&rec).storing, Some(storing));
        rec.storing = None;
        assert_eq!(parts.input(&rec).storing, None);
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

    #[test]
    fn both_schema_21_ware_layouts_replay_their_original_weight_policy() {
        let stock = stock();
        let (_, mut record) = produce(&stock, 300, &traits(), &[], &[]);
        record.schema = 21;
        record.storing = Some(crate::borg_town_utility::Storing::default().to_array());
        record.capability = Some(crate::borg_town_capability::Model {
            available: Vec::new(),
            packing: Some(crate::borg_town_packing::Packing { block: 40, used: 0, classes: Vec::new() }),
            curves: (0..53).map(|id| [id, 0, 0, 0, 0]).collect(),
            supplies: Vec::new(),
        });
        for legacy_home_21 in [false, true] {
            record.legacy_home_21 = legacy_home_21;
            let mut line = v2_prefix(&stock, &traits(), 300).replacen("\"v\":4", "\"v\":21", 1);
            record.push_fields(&mut line);
            line.push('}');
            let (parsed, parts) = replay_parts(&line).unwrap();
            assert_eq!(parsed.legacy_home_21, legacy_home_21);
            assert_eq!(parts.input(&parsed).shop_overloaded, !legacy_home_21);
        }
    }

    #[test]
    fn reserve_offer_respects_the_purse_home_and_temporary_pack_room() {
        let stock = stock();
        let (_, mut record) = produce(&stock, 300, &traits(), &[], &[]);
        record.schema = PLAN_SCHEMA;
        record.storing = Some(crate::borg_town_utility::Storing::default().to_array());
        let mut candidates = stock.candidates.clone();
        candidates.truncate(1);
        let offer = &mut candidates[0];
        offer.shop = 6;
        offer.price = 100;
        offer.weight = 30;
        offer.qty = 1;
        let facts = [WareFact { shop: offer.shop, slot: offer.slot, kind: offer.kind,
            class: offer.class, slots_cost: 1, weight: offer.weight, need: -1,
            reduction: 0, serves: 0, home_value: 9000 }];
        let parts = ReplayParts { candidates, urgent: Vec::new(), traits: traits(),
            carried: carried().to_vec(), limits: Limits { gold: 300, pack_slots: 1, weight: 30 },
            tryon: Vec::new(), power: 0 };
        let mut input = parts.input(&record);
        input.facts = &facts;
        input.home_slots = 1;
        let available = step(&input);
        assert_eq!(available.home_purchase, Some(0));
        assert_eq!(first_executable_action(&input.candidates, &[], &available, &facts, &[], 1, 1, 30).0,
            action::BUY_FOR_HOME);
        input.home_slots = 0;
        assert_eq!(step(&input).home_purchase, None);
        let ranked = [WareFact { home_value: crate::borg_town_home::Role::Gear.offer_value(9000), ..facts[0] }];
        let expendable = [KitItem { home_role: crate::borg_town_home::Role::Common, home_room: Some([3, 0, 3]),
            keep_last_book: false, protect_disposal: false, home_unwanted: false, home_limited: false,
            joins_pack: false, place: Place::Home, slot: 4, kind: 50, qty: 3, weight: 5, need: -1,
            reduction: 0, serves: 0, power: 0, proceeds: 10, sell_shop: 4, unknown: false,
            unknown_flavour: false, just_bought: false, cursed_spare: false, fuel: 0 }];
        input.facts = &ranked;
        input.kit = &expendable;
        let crowded = step(&input);
        assert_eq!(crowded.home_purchase, None);
        assert_eq!(first_executable_action(&input.candidates, &[], &crowded, &ranked, &expendable, 0, 1, 30).0,
            action::SELL_HOME);
        input.facts = &facts;
        input.kit = &[];
        input.home_slots = 1;
        input.limits.pack_slots = 0;
        assert_eq!(step(&input).home_purchase, None);
        input.limits.pack_slots = 1;
        input.limits.weight = 29;
        assert_eq!(step(&input).home_purchase, None);
        input.limits.weight = 30;
        input.black_market_percentage = 0;
        assert_eq!(step(&input).home_purchase, None);
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
        let pack = KitItem { home_role: Default::default(), home_room: None, keep_last_book: false, protect_disposal: false, home_unwanted: false, home_limited: false, joins_pack: false, place: Place::Pack, slot: 0, kind: 1, qty: 1,
            weight: 1, need: -1, reduction: 0, power: 0, proceeds: 0,
            sell_shop: 4, unknown: false, unknown_flavour: false,
            just_bought: false, cursed_spare: false, fuel: 0, serves: 0 };
        let home = KitItem { joins_pack: false, place: Place::Home, slot: 5, ..pack };
        let kit = [pack, home];
        out.dispositions = vec![Disposition::Store, Disposition::Sell];
        assert_eq!(first_executable_action(&stock.candidates, &[], &out, &[], &kit, 0, 1, i64::MAX).0, action::SELL_HOME);
        assert_eq!(first_executable_action(&stock.candidates, &[], &out, &[], &kit, 1, 1, i64::MAX).0, action::STORE);
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
                    WareFact { home_value: 0, shop: c.shop, slot: c.slot, kind: c.kind, class: c.class, slots_cost: 1, weight: 10, need, reduction, serves }
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
        let junk = KitItem { home_role: Default::default(), home_room: None, keep_last_book: false, protect_disposal: false, home_unwanted: false, home_limited: false, joins_pack: false,
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
            cursed_spare: false,
            fuel: 0,
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
        let chosen=first_executable_action(&cs,&urgent,&out,&[],&[],1,0, i64::MAX);
        assert_eq!(chosen,(action::BUY,[i64::from(cs[1].shop),i64::from(cs[1].slot),1,i64::from(cs[1].price)],3,true));
        // When room exists the original urgent order still wins.
        assert_eq!(first_executable_action(&cs,&urgent,&out,&[],&[],1,1, i64::MAX).1[1],i64::from(cs[0].slot));
        cs[1].shop=7;
        assert_eq!(first_executable_action(&cs,&urgent,&out,&[],&[],1,0, i64::MAX).0,action::TAKE_HOME);
        cs[1].slots_cost=1;
        assert_eq!(first_executable_action(&cs,&urgent,&out,&[],&[],1,0, i64::MAX).0,action::LEAVE);
    }

    #[test]
    fn overweight_priest_can_buy_recall_and_a_worthwhile_upgrade() {
        let stock = stock();
        let (line, _) = produce(&stock, 24_223, &traits(), &[], &[]);
        let (mut rec, mut parts) = replay_parts(&line).unwrap();
        let recall = 0;
        let armour = parts.candidates.iter().position(|c| c.kind == ARMOUR).unwrap();
        let candidate = &parts.candidates[armour];
        let tryon = TryOn { shop: candidate.shop, slot: candidate.slot, eq_slot: 6, value: 500_000 };
        for (i, candidate) in parts.candidates.iter_mut().enumerate() {
            candidate.qty = i32::from(i == armour || i == recall);
        }
        parts.tryon = vec![tryon];
        parts.limits = Limits { gold: 24_098, pack_slots: 6, weight: -109 };
        parts.urgent = vec![UrgentPurchase { candidate: recall, needs: vec![Need::Trip] }];
        rec.schema = 20;
        let old = step(&parts.input(&rec));
        assert!(planned_purchase(&old).is_none());
        assert_eq!(first_executable_action(&parts.candidates, &parts.urgent, &old, &rec.wares, &[], 0, 7, -104).0, action::LEAVE);

        rec.schema = 21;
        let new = step(&parts.input(&rec));
        assert_eq!(live_selection(&parts.candidates, &parts.urgent, &new, &rec.wares, &[], 0, 7, i64::MAX, |_| Vec::new()).0, action::BUY);
        parts.urgent.clear();
        parts.candidates[recall].qty = 0;
        let new = step(&parts.input(&rec));
        assert_eq!(planned_purchase(&new), Some((armour, 1)));
        let ordinary = acquisition_before_disposal(&parts.candidates, &[], &new, &rec.wares, &[], 6).unwrap();
        assert_eq!((ordinary.0, ordinary.1[1]), (action::BUY, i64::from(parts.candidates[armour].slot)));
        rec.mode = 2;
        assert!(planned_purchase(&step(&parts.input(&rec))).is_none(), "departure cannot buy the surplus back");
    }

    #[test]
    fn full_pack_frees_room_before_urgent_buy_and_does_not_queue_an_impossible_buy() {
        let stock=stock();let (line,_)=produce(&stock,300,&traits(),&[],&[]);
        let (_,mut out)=replay(&line).unwrap();
        let pack=KitItem { home_role: Default::default(), home_room: None, keep_last_book: false, protect_disposal: false, home_unwanted: false, home_limited: false, cursed_spare: false, fuel: 0,joins_pack:false,place:Place::Pack,slot:0,kind:1,qty:1,weight:1,need:-1,reduction:0,power:0,
            proceeds:0,sell_shop:4,unknown:false,unknown_flavour:false,just_bought:false,serves:0};
        let kit=[pack];out.dispositions=vec![Disposition::Sell];
        let urgent=[UrgentPurchase {candidate:0,needs:vec![Need::Food]}];
        assert_eq!(first_executable_action(&stock.candidates,&urgent,&out,&[],&kit,1,0, i64::MAX).0,action::SELL);
        out.dispositions[0]=Disposition::Keep;out.solution.chosen.fill(0);
        assert_eq!(first_executable_action(&stock.candidates,&urgent,&out,&[],&kit,1,0, i64::MAX).0,action::LEAVE);
        assert_eq!(first_executable_action(&stock.candidates,&urgent,&out,&[],&kit,1,1, i64::MAX).0,action::BUY);
        let mut cs=stock.candidates.clone();cs[0].shop=7;cs[0].slots_cost=0;
        assert_eq!(first_executable_action(&cs,&urgent,&out,&[],&kit,1,0, i64::MAX).0,action::TAKE_HOME);
    }

    /// #939 re-review (P2): an urgent Recall the pack cannot carry, with no
    /// room to make, does not hide an affordable urgent cure that fits.
    #[test]
    fn a_blocked_urgent_recall_does_not_hide_a_cure_that_fits() {
        let stock=stock();let (line,_)=produce(&stock,300,&traits(),&[],&[]);
        let (_,mut out)=replay(&line).unwrap();
        out.dispositions.clear();out.solution.chosen.fill(0);out.ranged_purchase=None;
        let mut cs=stock.candidates.clone();
        let recall=0;let cure=cs.iter().position(|c| c.kind==CLW).unwrap();
        cs[recall].weight=5;cs[recall].slots_cost=1;cs[cure].weight=4;cs[cure].slots_cost=1;
        let urgent=[UrgentPurchase {candidate:recall,needs:vec![Need::Trip]}];
        let cure_set=|unfit:&[usize]| { assert!(unfit.contains(&recall) && !unfit.contains(&cure)); vec![UrgentPurchase {candidate:cure,needs:vec![Need::Poison]}] };
        // Four tenths free: the Recall does not fit, the cure does.
        let (a,act,_,now)=live_selection(&cs,&urgent,&out,&[],&[],1,1,4,cure_set);
        assert_eq!((a,act[1],now),(action::BUY,i64::from(cs[cure].slot),true));
        // Five tenths free: the Recall fits and keeps its priority.
        let (a,act,_,_)=live_selection(&cs,&urgent,&out,&[],&[],1,1,5,|_:&[usize]| panic!("no fallback when it fits"));
        assert_eq!((a,act[1]),(action::BUY,i64::from(cs[recall].slot)));
        // Nothing else urgent and affordable: the visit still leaves.
        let (a,..)=live_selection(&cs,&urgent,&out,&[],&[],1,1,4,|_:&[usize]| Vec::new());
        assert_eq!(a,action::LEAVE);
    }

    /// Review of #973: the retry leaves out every ware that does not fit, not
    /// only the blocked one, so it cannot pick a second ware that does not
    /// fit either. The real urgent budgeter: 150 gold, the Recall (125 gold,
    /// 5 tenths) first; a 75-gold cure at 4 tenths and a 140-gold cure at 1
    /// tenth; one tenth free. Leaving out only the Recall, the budgeter picks
    /// the cheaper, heavier cure and the visit leaves; the fix buys the light
    /// one.
    #[test]
    fn the_urgent_retry_leaves_out_every_ware_that_does_not_fit() {
        use crate::borg_town_budget::Facts;
        let stock=stock();let (line,_)=produce(&stock,300,&traits(),&[],&[]);
        let (_,mut out)=replay(&line).unwrap();
        out.dispositions.clear();out.solution.chosen.fill(0);out.ranged_purchase=None;
        let ware=|slot:u8,kind:u32,price:i32,weight:i32| Candidate { shop:5,slot,kind,tval:0,sval:0,price,qty:1,
            weight,slots_cost:1,stack_count:0,class:u16::from(slot) };
        let (recall,light)=(0usize,2usize);
        let mut shop=Stock::default();
        shop.candidates=vec![ware(0,900,125,5),ware(1,901,75,4),ware(2,902,140,1)];
        let serves=|c:&Candidate| if c.kind==900 { vec![Need::Trip] } else { vec![Need::Poison] };
        let facts=Facts { gold:150, pack_capacity:23, home_capacity:24, carry:2000, weight:0 };
        let needs=[Need::Poison,Need::Trip];
        let first=Budget::build(&facts,&shop,&needs,serves);
        assert_eq!(first.urgent.iter().map(|f| f.candidate).collect::<Vec<_>>(),[recall]);
        let rebudget=|unfit:&[usize]| { let mut s=shop.clone(); for &i in unfit { s.candidates[i].qty=0; } Budget::build(&facts,&s,&needs,serves).urgent };
        let (a,act,_,now)=live_selection(&shop.candidates,&first.urgent,&out,&[],&[],1,1,1,rebudget);
        assert_eq!((a,act[1],now),(action::BUY,i64::from(shop.candidates[light].slot),true));
        // The old retry (the blocked Recall only) picks the heavy cure: LEAVE.
        let only_blocked=|_:&[usize]| { let mut s=shop.clone(); s.candidates[recall].qty=0; Budget::build(&facts,&s,&needs,serves).urgent };
        let (a,..)=live_selection(&shop.candidates,&first.urgent,&out,&[],&[],1,1,1,only_blocked);
        assert_eq!(a,action::LEAVE);
    }

    /// #955 (P7 review): an urgent copy that does not fit the weight still
    /// free waits for the room-making dispositions; once it fits, it is bought.
    #[test]
    fn urgent_buy_over_the_weight_allowance_frees_weight_first() {
        let stock=stock();let (line,_)=produce(&stock,300,&traits(),&[],&[]);
        let (_,mut out)=replay(&line).unwrap();
        let pack=KitItem { home_role: Default::default(), home_room: None, keep_last_book: false, protect_disposal: false, home_unwanted: false, home_limited: false, cursed_spare: false, fuel: 0,joins_pack:false,place:Place::Pack,slot:0,kind:1,qty:1,weight:50,need:-1,reduction:0,power:0,
            proceeds:0,sell_shop:4,unknown:false,unknown_flavour:false,just_bought:false,serves:0};
        let kit=[pack];out.dispositions=vec![Disposition::Store];
        let urgent=[UrgentPurchase {candidate:0,needs:vec![Need::Food]}];
        let w=i64::from(stock.candidates[0].weight);
        assert!(w > 0);
        assert_eq!(first_executable_action(&stock.candidates,&urgent,&out,&[],&kit,1,1,w-1).0,action::STORE, "room first");
        assert_eq!(first_executable_action(&stock.candidates,&urgent,&out,&[],&kit,1,1,w).0,action::BUY, "fits: bought first");
        out.dispositions[0]=Disposition::Keep;
        assert_eq!(first_executable_action(&stock.candidates,&urgent,&out,&[],&kit,1,1,w-1).0,action::LEAVE, "no room to make: never overweight");
    }

}
