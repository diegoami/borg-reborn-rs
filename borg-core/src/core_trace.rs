//! borg-reborn-rs (#222): the vendored core's decision trace.
//!
//! Nothing here is read by policy. Snapshots use the core's own state after
//! a selected check and before its queued keys are consumed. Danger is a
//! read-only evaluation (no RNG); full-suite parity checks guard that seam.
use crate::borg_headers::borg_trait_h::{BI_CDEPTH, BI_CURHP, BI_CURSP, BI_MAXHP, BI_MAXSP};
use crate::state::BorgState;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Default)]
struct Selection {
    healing_potions: Option<i32>,
    name: Option<&'static str>,
    source: Option<&'static str>,
    attacks: Vec<AttackRanking>,
    prepared: Vec<PreparedCheck>,
    store_exit: Option<StoreExit>,
}

/// borg-reborn-rs (#1395): a town planner action as the trace records it:
/// the store it targets, the action code and its four arguments (the
/// `TownAction` of borg_town_shadow.rs).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct PlannedAction {
    pub(crate) shop: i32,
    pub(crate) code: i64,
    pub(crate) at: [i64; 4],
}

/// borg-reborn-rs (re-review of #1414): one attempt inside a store visit:
/// the action chosen there before execution could clear it, and its
/// outcome (`queued`, a reason it was not, or what settlement then found).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct StoreAttempt {
    pub(crate) action: PlannedAction,
    pub(crate) outcome: String,
}

/// borg-reborn-rs (#1395): a store left without a transaction, and why. Only
/// the live town planner records it, when it leaves a store with no command
/// queued. `requested` is the store action chosen outside that brought the
/// Borg in, kept for the whole visit; `attempts` each action tried inside
/// during the visit, follow-up re-entries of the same store included
/// (reviews of #1414); `planned_shop` and `planned_action` are the goal
/// still held at exit (-1 and `None` once cleared).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct StoreExit {
    shop: i32,
    reason: String,
    requested: Option<PlannedAction>,
    attempts: Vec<StoreAttempt>,
    planned_shop: i32,
    planned_action: Option<i64>,
}

/// Record why the store of this think is left without a transaction.
pub(crate) fn record_store_exit(
    shop: i16, reason: &str, requested: Option<PlannedAction>, attempts: Vec<StoreAttempt>,
    planned_shop: i16, planned_action: Option<i64>,
) {
    FRAMES.with(|frames| {
        if let Some(frame) = frames.borrow_mut().last_mut() {
            frame.store_exit = Some(StoreExit {
                shop: i32::from(shop), reason: reason.into(), requested, attempts,
                planned_shop: i32::from(planned_shop), planned_action,
            });
        }
    });
}

/// One native attack-selector invocation. Multiple attempts in one think are
/// retained; nested thinks use their own Selection frames. Data never feeds policy.
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct AttackRanking {
    candidates: Vec<AttackCandidate>,
    winner: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct AttackCandidate {
    attack_id: i32,
    attack_name: Option<String>,
    family: Option<String>,
    class_id: i32,
    class_name: Option<String>,
    // Only the aggregate result is available here. Do not claim a nonpositive
    // return identifies a specific immunity, mana or charge rejection.
    eligibility: String,
    raw_utility: i32,
    parameter_name: Option<String>,
    effective_weight: i32,
    parameter_origin: String,
    adjusted_utility: i32,
    incumbent_changed: bool,
}

impl AttackRanking {
    pub(crate) fn candidate(&mut self, class: i32, id: i32, raw: i32,
        result: &crate::attack_preference::Candidate, changed: bool) {
        self.candidates.push(AttackCandidate {
            attack_id: id,
            attack_name: result.attack.map(|a| a.name.into()),
            family: result.attack.map(|a| a.family.name().into()),
            class_id: class,
            class_name: crate::attack_preference::class_name(class).map(str::to_owned),
            eligibility: if result.attack.is_none() { "unknown_attack" }
                else if raw <= 0 { "nonpositive_return" } else { "positive_return" }.into(),
            raw_utility: raw,
            parameter_name: result.parameter.map(|p| crate::params::SPECS[p as usize].name.into()),
            effective_weight: result.weight,
            parameter_origin: result.origin.into(),
            adjusted_utility: result.adjusted,
            incumbent_changed: changed,
        });
    }
}

/// Allocate candidate records only when a trace frame is active. No frame borrow
/// survives simulation (which may reenter thinking); no engine reads or RNG.
pub(crate) fn attack_ranking() -> Option<AttackRanking> {
    FRAMES.with(|frames| (!frames.borrow().is_empty()).then(AttackRanking::default))
}

pub(crate) fn record_attack_ranking(trace: Option<AttackRanking>, winner: Option<i32>) {
    if let Some(mut trace) = trace {
        trace.winner = winner;
        FRAMES.with(|frames| {
            if let Some(frame) = frames.borrow_mut().last_mut() {
                frame.attacks.push(trace);
            }
        });
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct PreparedCheck {
    current_depth: i32,
    proposed_depth: i32,
    source: String,
    requirement: Option<FailedRequirement>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct FailedRequirement {
    name: String,
    actual: i32,
    required: i32,
    relation: String,
    message: String,
}

pub(crate) fn record_prepared(
    st: &BorgState, depth: i32, result: Option<&crate::borg_prepared::Reason>,
    source: &std::panic::Location<'_>,
) {
    FRAMES.with(|frames| {
        let mut frames = frames.borrow_mut();
        let Some(frame) = frames.last_mut() else { return };
        frame.prepared.push(PreparedCheck {
            current_depth: st.borg.tr(BI_CDEPTH),
            proposed_depth: depth,
            source: source.to_string(),
            requirement: result.map(|reason| FailedRequirement {
                name: reason.requirement.name.into(),
                actual: reason.requirement.actual,
                required: reason.requirement.required,
                relation: reason.requirement.relation.into(),
                message: String::from_utf8_lossy(reason.text(st)).into_owned(),
            }),
        });
    });
}

#[derive(Debug, Default)]
struct ObservedTrip {
    depth: i32,
    trip_id: u64,
    pending_think: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct PreparedOutcome {
    think: u64,
    action: DepthOutcome,
    depth: i32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum DepthOutcome {
    Descended,
    ReturnedToTown,
    Ascended,
    Stayed,
}

impl ObservedTrip {
    fn observe(&mut self, depth: i32, next_pending: Option<u64>) -> Option<PreparedOutcome> {
        let action = if depth > self.depth {
            DepthOutcome::Descended
        } else if depth == self.depth {
            DepthOutcome::Stayed
        } else if depth == 0 {
            DepthOutcome::ReturnedToTown
        } else {
            DepthOutcome::Ascended
        };
        if depth > 0 && self.depth == 0 {
            self.trip_id += 1;
        }
        self.depth = depth;
        std::mem::replace(&mut self.pending_think, next_pending)
            .map(|think| PreparedOutcome { think, action, depth })
    }
}

thread_local! {
    // Nested engine message handling may invoke another think. A stack keeps
    // its selection separate from the outer think, without holding a lock.
    static FRAMES: RefCell<Vec<Selection>> = const { RefCell::new(Vec::new()) };
}
static PATH: OnceLock<Option<PathBuf>> = OnceLock::new();
static WRITER: OnceLock<Mutex<Recorder>> = OnceLock::new();

struct Recorder {
    file: File,
    think: u64,
    trip: ObservedTrip,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct Goals {
    leaving: bool,
    fleeing: bool,
    fleeing_to_town: bool,
    rising: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct Errand {
    name: String,
    target: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct Line {
    schema: String,
    think: u64,
    turn: i32,
    borg_turn: i32,
    context: String,
    depth: i32,
    hp: i32,
    max_hp: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    healing_potions: Option<i32>,
    sp: i32,
    max_sp: i32,
    danger: i32,
    avoidance: i32,
    check: String,
    check_source: Option<String>,
    goals: Goals,
    errand: Option<Errand>,
    keys: Vec<u32>,
    acted: bool,
    #[serde(default)]
    attacks: Vec<AttackRanking>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    prepared: Vec<PreparedCheck>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    trip_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    previous_prepared_outcome: Option<PreparedOutcome>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    store_exit: Option<StoreExit>,
}

fn failure(error: impl std::fmt::Display) -> ! {
    eprintln!("borg-reborn-rs: core trace: {error}");
    std::process::exit(2)
}

pub(crate) fn begin() -> bool {
    let path = PATH.get_or_init(|| {
        if std::env::var("BORG_CORE_TRACE")
            .is_ok_and(|v| matches!(v.as_str(), "off" | "0" | "false"))
        {
            return None;
        }
        std::env::var_os("BORG_RUN_DIR").map(|dir| PathBuf::from(dir).join("trace-core.jsonl"))
    });
    if path.is_none() {
        return false;
    }
    FRAMES.with(|frames| frames.borrow_mut().push(Selection::default()));
    true
}

thread_local! {
    /// borg-reborn-rs (#1153): the last checks chosen, newest last, kept
    /// whether or not the trace is written (the loop detector's note).
    static RECENT: RefCell<std::collections::VecDeque<&'static str>> = const { RefCell::new(std::collections::VecDeque::new()) };
}

/// #1153: the last (up to 4) checks chosen, oldest first.
pub(crate) fn recent_choices() -> Vec<&'static str> {
    RECENT.with(|r| r.borrow().iter().copied().collect())
}

pub(crate) fn selected(name: &'static str, source: &'static str) {
    RECENT.with(|r| {
        let mut r = r.borrow_mut();
        r.push_back(name);
        while r.len() > 4 {
            r.pop_front();
        }
    });
    FRAMES.with(|frames| {
        if let Some(frame) = frames.borrow_mut().last_mut() {
            frame.name = Some(name);
            frame.source = Some(source);
        }
    });
}

// One marked hook at each successful check's return. It evaluates no policy
// expression again and returns the original `true` unchanged.
macro_rules! choice {
    ($name:literal) => {{
        crate::core_trace::selected($name, concat!(file!(), ":", line!()));
        true
    }};
}
pub(crate) use choice;

pub(crate) fn observe_healing_potions(st: &BorgState) {
    FRAMES.with(|frames| {
        if let Some(frame) = frames.borrow_mut().last_mut() {
            frame.healing_potions = Some(crate::borg_healing_restock::carried_potions(st));
        }
    });
}

pub(crate) fn finish(st: &mut BorgState, active: bool, acted: bool) {
    if !active {
        return;
    }
    let frame = FRAMES.with(|frames| frames.borrow_mut().pop().expect("core trace frame"));
    let depth = st.borg.tr(BI_CDEPTH);
    let check = frame.name.unwrap_or(if acted {
        "unclassified"
    } else {
        "borg_think_no_action"
    });
    let context = if check == "borg_think_store" {
        "store"
    } else if depth == 0 {
        "town"
    } else {
        "dungeon"
    };
    let danger = crate::borg_danger::borg_danger(st, st.borg.c.y, st.borg.c.x, 1, true, false);
    // borg-reborn-rs (#1339): the errands are retired; the field stays in
    // the schema (always null) so that readers of core-trace@2 keep working.
    let errand: Option<Errand> = None;
    let mut line = Line {
        schema: "borg-reborn-rs/core-trace@2".into(),
        think: 0,
        turn: crate::engine::game_turn(),
        borg_turn: st.borg.time.now,
        context: context.into(),
        depth,
        hp: st.borg.tr(BI_CURHP),
        max_hp: st.borg.tr(BI_MAXHP),
        healing_potions: frame.healing_potions,
        sp: st.borg.tr(BI_CURSP),
        max_sp: st.borg.tr(BI_MAXSP),
        danger,
        avoidance: i32::from(st.borg.avoidance),
        check: check.into(),
        check_source: frame.source.map(str::to_string),
        goals: Goals {
            leaving: st.borg.goal.leaving,
            fleeing: st.borg.goal.fleeing,
            fleeing_to_town: st.borg.goal.fleeing_to_town,
            rising: st.rising_to_town(),
        },
        errand,
        keys: st.io.queued_keys(),
        acted,
        attacks: frame.attacks,
        prepared: frame.prepared,
        trip_id: None,
        previous_prepared_outcome: None,
        store_exit: frame.store_exit,
    };
    let writer = WRITER.get_or_init(|| {
        let path = PATH
            .get()
            .and_then(Option::as_ref)
            .expect("enabled core trace path");
        let file =
            File::create(path).unwrap_or_else(|e| failure(format!("{}: {e}", path.display())));
        Mutex::new(Recorder { file, think: 0, trip: ObservedTrip::default() })
    });
    let mut writer = writer.lock().unwrap_or_else(|e| e.into_inner());
    writer.think = writer.think.wrapping_add(1);
    line.think = writer.think;
    let checked = !line.prepared.is_empty();
    line.previous_prepared_outcome = writer.trip.observe(depth, checked.then_some(line.think));
    line.trip_id = checked.then_some(writer.trip.trip_id);
    let mut bytes = serde_json::to_vec(&line).unwrap_or_else(|e| failure(e));
    bytes.push(b'\n');
    // A direct write per line leaves no userspace buffer stranded when the C
    // engine exits on a death. Never hold this lock across a policy call.
    writer.file.write_all(&bytes).unwrap_or_else(|e| failure(e));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trip_outcomes_link_to_the_previous_think_and_keep_one_id_until_town() {
        let mut trip = ObservedTrip::default();
        assert_eq!(trip.observe(0, Some(2)), None);
        assert_eq!(trip.trip_id, 0);
        assert_eq!(trip.observe(1, Some(5)), Some(PreparedOutcome {
            think: 2, action: DepthOutcome::Descended, depth: 1,
        }));
        assert_eq!(trip.trip_id, 1);
        assert_eq!(trip.observe(1, Some(6)).unwrap().action, DepthOutcome::Stayed);
        assert_eq!(trip.observe(2, Some(7)).unwrap().action, DepthOutcome::Descended);
        assert_eq!(trip.observe(1, Some(8)).unwrap().action, DepthOutcome::Ascended);
        assert_eq!(trip.trip_id, 1);
        assert_eq!(trip.observe(0, Some(9)).unwrap().action, DepthOutcome::ReturnedToTown);
        assert_eq!(trip.observe(2, None).unwrap().action, DepthOutcome::Descended);
        assert_eq!(trip.trip_id, 2);
    }

    #[test]
    fn nested_frames_keep_their_own_selected_check() {
        FRAMES.with(|frames| frames.borrow_mut().push(Selection::default()));
        selected("outer", "outer.rs:1");
        FRAMES.with(|frames| frames.borrow_mut().push(Selection::default()));
        selected("inner", "inner.rs:2");
        FRAMES.with(|frames| {
            let mut frames = frames.borrow_mut();
            assert_eq!(frames.pop().unwrap().name, Some("inner"));
            assert_eq!(frames.pop().unwrap().name, Some("outer"));
        });
    }

    #[test]
    fn attack_attempts_are_retained_and_nested_frames_are_isolated() {
        use crate::borg_headers::borg_fight_attack_h::*;
        FRAMES.with(|f| f.borrow_mut().push(Selection::default()));
        let mut outer = attack_ranking().unwrap();
        let result = crate::attack_preference::evaluate(1, BF_THRUST as i32, 25);
        outer.candidate(1, BF_THRUST as i32, 25, &result, true);
        FRAMES.with(|f| f.borrow_mut().push(Selection::default()));
        let mut inner = attack_ranking().unwrap();
        let invalid = crate::attack_preference::evaluate(1, BF_WAND_MAGIC_MISSILE as i32, 0);
        inner.candidate(1, BF_WAND_MAGIC_MISSILE as i32, 0, &invalid, false);
        record_attack_ranking(Some(inner), None);
        let nested = FRAMES.with(|f| f.borrow_mut().pop().unwrap());
        assert_eq!(nested.attacks.len(), 1);
        assert_eq!(nested.attacks[0].winner, None);
        assert_eq!(nested.attacks[0].candidates[0].eligibility, "nonpositive_return");
        record_attack_ranking(Some(outer), Some(BF_THRUST as i32));
        record_attack_ranking(attack_ranking(), None);
        let frame = FRAMES.with(|f| f.borrow_mut().pop().unwrap());
        assert_eq!(frame.attacks.len(), 2);
        let trace = &frame.attacks[0];
        assert_eq!(trace.winner, Some(BF_THRUST as i32));
        assert_eq!(trace.candidates[0].raw_utility, 25);
        assert_eq!(trace.candidates[0].adjusted_utility, 25);
        assert_eq!(trace.candidates[0].parameter_origin, "default");
        assert_eq!(trace.candidates[0].parameter_name.as_deref(), Some("attack_preference_melee_permille"));
        let json = serde_json::to_string(trace).unwrap();
        assert_eq!(serde_json::from_str::<AttackRanking>(&json).unwrap(), *trace);
        assert!(attack_ranking().is_none());
    }

    #[test]
    fn line_round_trips_with_keys_and_errand() {
        let line = Line {
            schema: "borg-reborn-rs/core-trace@2".into(),
            think: 9,
            turn: 1234,
            borg_turn: 120,
            context: "dungeon".into(),
            depth: 3,
            hp: 12,
            max_hp: 40,
            healing_potions: Some(1),
            sp: 2,
            max_sp: 9,
            danger: 15,
            avoidance: 12,
            check: "borg_caution".into(),
            check_source: Some("borg_think_dungeon.rs:1536".into()),
            goals: Goals {
                leaving: true,
                fleeing: true,
                fleeing_to_town: false,
                rising: false,
            },
            errand: Some(Errand {
                name: "trips:1*/1,2\nquoted \"plan\"".into(),
                target: Some(2),
            }),
            keys: vec![27, 82, 49, 48, 48, 13, 0x80],
            acted: true,
            attacks: vec![],
            prepared: vec![],
            trip_id: None,
            previous_prepared_outcome: None,
            store_exit: Some(StoreExit {
                shop: 4, reason: "the planned ware is gone".into(),
                requested: Some(PlannedAction { shop: 4, code: 1, at: [4, 2, 1, 0] }),
                attempts: vec![StoreAttempt {
                    action: PlannedAction { shop: 4, code: 1, at: [4, 2, 1, 0] },
                    outcome: "the purchase made no progress".into(),
                }],
                planned_shop: -1, planned_action: None,
            }),
        };
        let text = serde_json::to_string(&line).unwrap();
        assert!(!text.contains('\n'));
        assert_eq!(serde_json::from_str::<Line>(&text).unwrap(), line);

        let mut old: serde_json::Value = serde_json::from_str(&text).unwrap();
        old.as_object_mut().unwrap().remove("healing_potions");
        assert_eq!(serde_json::from_value::<Line>(old).unwrap().healing_potions, None);
    }
}
