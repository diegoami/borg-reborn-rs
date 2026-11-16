//! Opt-in POC: class-specific level phases from a separate errands_plan.txt.
use crate::borg_headers::borg_h::BORG_NO_DEEPER;
use crate::borg_headers::borg_trait_h::BI_CLEVEL;
use crate::state::BorgState;
use crate::{engine, shell};
use std::collections::BTreeMap;
use std::io::Read;
use std::sync::{Mutex, OnceLock};

/// What a phase asks of the core (owner, 2026-11-16: an additional check,
/// not a forced behaviour).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    /// `max-depth N`: no errand; the core plays as itself and is not
    /// prepared for any level below N (`borg_no_deeper`).
    MaxDepth(i32),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase {
    pub low: i32,
    pub high: i32,
    pub command: Command,
    pub line: usize,
}
/// The deepest level a `max-depth` phase may name (`borg_no_deeper` is a
/// level the core is never prepared for; 127 is the game's maximum).
const MAX_DEPTH: i32 = 127;
#[derive(Clone, Debug)]
pub struct Plan {
    sections: BTreeMap<String, Vec<Phase>>,
}

fn bound(s: &str) -> Result<i32, String> {
    s.parse::<i32>()
        .ok()
        .filter(|n| *n > 0)
        .ok_or_else(|| format!("invalid positive level {s:?}"))
}
fn range(s: &str) -> Result<(i32, i32), String> {
    let t: Vec<_> = s.split_whitespace().collect();
    let result = match t.as_slice() {
        ["level", "<=", n] => (1, bound(n)?),
        ["level", "<", n] => (1, bound(n)?.checked_sub(1).ok_or("level underflow")?),
        ["level", ">=", n] => (bound(n)?, i32::MAX),
        ["level", ">", n] => (bound(n)?.checked_add(1).ok_or("level overflow")?, i32::MAX),
        ["level", "==", n] => (bound(n)?, bound(n)?),
        [a, op, "level", "<=", b] if *op == "<" || *op == "<=" => {
            let a = bound(a)?;
            (
                if *op == "<" {
                    a.checked_add(1).ok_or("level overflow")?
                } else {
                    a
                },
                bound(b)?,
            )
        }
        _ => return Err(format!("unsupported level range {s:?}")),
    };
    if result.0 > result.1 {
        return Err("empty level range".into());
    }
    Ok(result)
}
impl Plan {
    pub fn parse(text: &str) -> Result<Self, String> {
        if text.len() > 65536 {
            return Err("plan exceeds 64 KiB".into());
        }
        let text = text.strip_prefix('\u{feff}').unwrap_or(text);
        let mut sections = BTreeMap::<String, Vec<Phase>>::new();
        let mut section: Option<String> = None;
        let mut pending: Option<(i32, i32, usize)> = None;
        let mut phases = 0;
        for (index, raw) in text.lines().enumerate() {
            let line = index + 1;
            let error = |why: String| format!("line {line}: {why}");
            let s = raw.trim();
            if s.is_empty() || s.starts_with('#') {
                continue;
            }
            if raw.contains('\t') {
                return Err(error("tabs are not supported".into()));
            }
            let indent = raw.len() - raw.trim_start_matches(' ').len();
            if indent == 0 && s.starts_with('[') && s.ends_with(']') {
                if pending.is_some() {
                    return Err(error("previous phase has no errand".into()));
                }
                let name = &s[1..s.len() - 1];
                if name != "default" && !shell::CLASSES.contains(&name) {
                    return Err(error(format!("unknown class {name:?}")));
                }
                if sections.insert(name.into(), Vec::new()).is_some() {
                    return Err(error(format!("duplicate section {name}")));
                }
                section = Some(name.into());
            } else if indent == 2 {
                if pending.is_some() {
                    return Err(error("previous phase has no errand".into()));
                }
                if section.is_none() {
                    return Err(error("start with [default] or a class section".into()));
                }
                let (low, high) = range(s).map_err(error)?;
                pending = Some((low, high, line));
            } else if indent == 4 {
                let (low, high, header) = pending
                    .take()
                    .ok_or_else(|| error("one command is required per phase".into()))?;
                let command = match s.split_whitespace().collect::<Vec<_>>().as_slice() {
                    ["max-depth", n] => {
                        let n = n
                            .parse::<i32>()
                            .ok()
                            .filter(|n| (1..MAX_DEPTH).contains(n))
                            .ok_or_else(|| error(format!("max-depth needs a level 1..{}", MAX_DEPTH - 1)))?;
                        Command::MaxDepth(n)
                    }
                    ["max-depth", ..] => return Err(error("max-depth takes one level".into())),
                    ["loop-levels", ..] => {
                        return Err(error(
                            "loop-levels was removed (owner, 2026-11-16); use max-depth N".into(),
                        ))
                    }
                    _ => {
                        return Err(error(
                            "a phase is max-depth N; no aliases".into(),
                        ))
                    }
                };
                let entries = sections.get_mut(section.as_ref().unwrap()).unwrap();
                if entries.iter().any(|p| low <= p.high && high >= p.low) {
                    return Err(error(
                        "overlapping phases (shared boundaries need <)".into(),
                    ));
                }
                entries.push(Phase {
                    low,
                    high,
                    command,
                    line: header,
                });
                phases += 1;
                if phases > 256 {
                    return Err(error("more than 256 phases".into()));
                }
            } else {
                return Err(error(
                    "expected [class], two-space range or four-space command".into(),
                ));
            }
        }
        if pending.is_some() {
            return Err("last phase has no errand".into());
        }
        if sections.is_empty() || sections.values().any(Vec::is_empty) {
            return Err("plan/sections must not be empty".into());
        }
        Ok(Self { sections })
    }
    /// A class section replaces the default completely: its gaps never fall
    /// through to another class's/default phase.
    fn validate_class(&self, class: &str) -> Result<(), String> {
        if self.sections.contains_key(class) || self.sections.contains_key("default") {
            Ok(())
        } else {
            Err(format!("no [{class}] or [default] plan"))
        }
    }
    pub fn phase(&self, class: &str, level: i32) -> Result<Option<&Phase>, String> {
        let entries = self
            .sections
            .get(class)
            .or_else(|| self.sections.get("default"))
            .ok_or_else(|| format!("no [{class}] or [default] plan"))?;
        Ok(entries.iter().find(|p| level >= p.low && level <= p.high))
    }
}

struct Runtime {
    plan: Plan,
    path: String,
    owner: Option<(u32, u64, i64)>,
    phase: Option<usize>,
    /// The `borg_no_deeper` the configuration set, before any `max-depth`
    /// phase lowered it.
    no_deeper: Option<i32>,
    cancelled: bool,
}
#[derive(Debug, PartialEq, Eq)]
enum Transition {
    Keep,
    Install(Phase),
    Stop,
}
impl Runtime {
    fn select(
        &mut self,
        me: (u32, u64, i64),
        class: &str,
        level: i32,
    ) -> Result<Transition, String> {
        if self
            .owner
            .is_some_and(|(c, b, t)| (c, b) != (me.0, me.1) || me.2 < t)
        {
            self.phase = None;
            self.cancelled = false;
        }
        self.owner = Some(me);
        if self.cancelled {
            return Ok(Transition::Keep);
        }
        match self.plan.phase(class, level)?.cloned() {
            Some(p) if self.phase != Some(p.line) => {
                self.phase = Some(p.line);
                Ok(Transition::Install(p))
            }
            Some(_) => Ok(Transition::Keep),
            None => {
                self.cancelled = true;
                Ok(Transition::Stop)
            }
        }
    }
}
static RUNTIME: OnceLock<Mutex<Option<Runtime>>> = OnceLock::new();
fn fatal(error: impl std::fmt::Display) -> ! {
    eprintln!("errands_plan: {error}");
    std::process::exit(2)
}
fn checked_setting(name: &str) -> Option<String> {
    shell::borg_txt_setting(name)
}
fn validate_budget(value: Option<&str>) -> Result<(), String> {
    match value {
        None | Some("none") => Ok(()),
        Some(value) if value.parse::<i64>().ok() == Some(0) => Ok(()),
        _ => Err(
            "POC plans require BORG_ERRAND_BUDGET unset, zero or none (without whitespace)".into(),
        ),
    }
}

fn decode_plan(bytes: &[u8]) -> Result<&str, String> {
    if bytes.len() > 65536 {
        return Err("plan exceeds 64 KiB".into());
    }
    std::str::from_utf8(bytes).map_err(|e| format!("plan is not UTF-8: {e}"))
}
/// Called before core initialization; absent configuration is inert.
pub(crate) fn initialize() {
    RUNTIME.get_or_init(|| {
        let file = std::env::var_os("BORG_ERRANDS_PLAN")
            .map(|p| {
                p.into_string()
                    .unwrap_or_else(|_| fatal("non-UTF-8 plan path"))
            })
            .or_else(|| checked_setting("borg_plan"));
        let Some(file) = file else {
            return Mutex::new(None);
        };
        if file.trim().is_empty() {
            fatal("empty plan path");
        }
        let budget = std::env::var_os("BORG_ERRAND_BUDGET");
        let budget = budget.as_ref().map(|v| {
            v.to_str()
                .unwrap_or_else(|| fatal("non-UTF-8 BORG_ERRAND_BUDGET"))
        });
        validate_budget(budget).unwrap_or_else(|e| fatal(e));
        let errand = std::env::var("BORG_ERRAND")
            .ok()
            .or_else(|| checked_setting("borg_reborn_errand"));
        if errand
            .as_deref()
            .is_some_and(|s| !matches!(s.trim(), "" | "none"))
        {
            fatal("a plan conflicts with an automatic BORG_ERRAND/borg_reborn_errand");
        }
        let path = if std::path::Path::new(&file).is_absolute() {
            file
        } else {
            String::from_utf8(engine::path_build_user(4096, file.as_bytes()))
                .unwrap_or_else(|e| fatal(e))
        };
        let mut bytes = Vec::new();
        std::fs::File::open(&path)
            .and_then(|f| f.take(65537).read_to_end(&mut bytes))
            .unwrap_or_else(|e| fatal(format!("{path}: {e}")));
        let text = decode_plan(&bytes).unwrap_or_else(|e| fatal(e));
        let plan = Plan::parse(text).unwrap_or_else(|e| fatal(format!("{path}: {e}")));
        let class_id = engine::class_cidx();
        let class = shell::CLASSES
            .get(class_id as usize)
            .unwrap_or_else(|| fatal("unknown player class"));
        plan.validate_class(class).unwrap_or_else(|e| fatal(e));
        if let Some(dir) = std::env::var_os("BORG_RUN_DIR") {
            let target = std::path::Path::new(&dir).join("errands_plan.txt");
            if target.canonicalize().ok() == std::fs::canonicalize(&path).ok() {
                fatal("plan input must be separate from BORG_RUN_DIR/errands_plan.txt");
            }
            std::fs::write(target, &bytes)
                .unwrap_or_else(|e| fatal(format!("cannot preserve plan: {e}")));
        }
        Mutex::new(Some(Runtime {
            plan,
            path,
            owner: None,
            phase: None,
            no_deeper: None,
            cancelled: false,
        }))
    });
    // Reinitialization/reincarnation must validate the new class as well.
    let class_id = engine::class_cidx();
    let class = shell::CLASSES
        .get(class_id as usize)
        .unwrap_or_else(|| fatal("unknown player class"));
    if let Some(r) = RUNTIME.get().unwrap().lock().unwrap().as_ref() {
        r.plan.validate_class(class).unwrap_or_else(|e| fatal(e));
    }
}
pub(crate) fn cancel() {
    if let Some(runtime) = RUNTIME.get() {
        if let Some(r) = runtime.lock().unwrap().as_mut() {
            r.cancelled = true;
        }
    }
}
pub(crate) fn tick(st: &mut BorgState) -> bool {
    let Some(runtime) = RUNTIME.get() else {
        return false;
    };
    // Called only outside stores, after queued keys have been consumed. Do
    // not replace a plan while the core is waiting for Recall.
    if st.borg.goal.recalling != 0 {
        return false;
    }
    let class_id = engine::class_cidx();
    let class = shell::CLASSES
        .get(class_id as usize)
        .unwrap_or_else(|| fatal("unknown player class"));
    let turn = i64::from(engine::game_turn());
    let level = st.borg.tr(BI_CLEVEL);
    let event = {
        let mut guard = runtime.lock().unwrap();
        let Some(r) = guard.as_mut() else {
            return false;
        };
        let me = (class_id, engine::player_birth_key(), turn);
        let change = r.select(me, class, level).unwrap_or_else(|e| fatal(e));
        if matches!(change, Transition::Install(_)) {
            r.no_deeper.get_or_insert(st.cfg(BORG_NO_DEEPER));
        }
        (change, r.path.clone(), r.no_deeper)
    };
    match event {
        (Transition::Install(p), path, configured) => {
            let configured = configured.unwrap_or_else(|| st.cfg(BORG_NO_DEEPER));
            let what = match p.command {
                Command::MaxDepth(n) => {
                    // No errand: the core plays as itself, never prepared
                    // for a level below N (borg_prepared, "No deeper").
                    st.core.cfg[BORG_NO_DEEPER as usize] = configured.min(n + 1);
                    shell::install_plan_stop();
                    format!("max-depth {n}")
                }
            };
            crate::borg_io::borg_note(
                st,
                &format!("# Plan: {path} class={class} level={level} line={} {what}", p.line),
            );
            st.borg.goal.type_0 = 0;
            false
        }
        (Transition::Stop, _, _) => {
            shell::install_plan_stop();
            crate::borg_think::borg_oops(
                st,
                &format!("plan has no phase for {class} level {level}"),
            );
            true
        }
        (Transition::Keep, _, _) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const EXAMPLE: &str = "[default]\n  level <= 5\n    max-depth 2\n  5 < level <= 10\n    max-depth 4\n[mage]\n  level <= 5\n    max-depth 1\n  5 < level <= 10\n    max-depth 3\n";
    #[test]
    fn shipped_file_is_parsed_and_covers_all_classes_until_50() {
        let text = include_str!("../../harness/plans/errands_plan.txt");
        let plan = Plan::parse(text).unwrap();
        for class in shell::CLASSES.iter().copied().chain(["default"]) {
            let phases = plan.sections.get(class).unwrap();
            for level in 1..50 {
                assert_eq!(
                    phases
                        .iter()
                        .filter(|p| level >= p.low && level <= p.high)
                        .count(),
                    1,
                    "{class} {level}"
                );
                assert!(plan.phase(class, level).unwrap().is_some());
            }
            assert!(plan.phase(class, 50).unwrap().is_none());
        }
        assert!(Plan::parse(&text.replace('\n', "\r\n")).is_ok());
        assert!(Plan::parse(&format!("\u{feff}{text}")).is_ok());
    }
    #[test]
    fn canonical_commands_and_precise_input_errors() {
        for command in ["/max-depth 2", "MAX-DEPTH 2", "max-depth:2", "loop-levels 0 2"] {
            assert!(Plan::parse(&format!("[default]\n  level <= 5\n    {command}")).is_err());
        }
        assert!(Plan::parse("[default]\n  level <= 5\n    loop-levels 5 3")
            .unwrap_err()
            .contains("loop-levels was removed"));
        let mut large = vec![b'a'; 65536];
        large.push(0xff);
        assert_eq!(decode_plan(&large).unwrap_err(), "plan exceeds 64 KiB");
        assert!(decode_plan(&[0xff]).unwrap_err().contains("UTF-8"));
        for v in [
            None,
            Some("0"),
            Some("none"),
            Some("+0"),
            Some("-0"),
            Some("000"),
        ] {
            assert!(validate_budget(v).is_ok());
        }
        for v in ["", " 0 ", "-1", "1", "NONE"] {
            assert!(validate_budget(Some(v)).is_err());
        }
    }
    #[test]
    fn strict_settings_distinguish_absence_from_read_and_decode_errors() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "pr308-settings-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        let path = root.join("borg.txt");
        assert_eq!(
            shell::borg_txt_value_at(&path, "borg_plan").unwrap(),
            None
        );
        std::fs::write(&path, "borg_plan = errands_plan.txt\n").unwrap();
        assert_eq!(
            shell::borg_txt_value_at(&path, "borg_plan").unwrap(),
            Some("errands_plan.txt".into())
        );
        std::fs::write(&path, [0xff]).unwrap();
        assert!(shell::borg_txt_value_at(&path, "borg_plan").is_err());
        assert!(shell::borg_txt_value_at(&root, "borg_plan").is_err());
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(root).unwrap();
    }
    #[test]
    fn controller_switches_once_and_can_be_cancelled() {
        let mut r = Runtime {
            plan: Plan::parse(EXAMPLE).unwrap(),
            path: "test".into(),
            owner: None,
            phase: None,
            no_deeper: None,
            cancelled: false,
        };
        assert!(matches!(
            r.select((1, 1, 1), "mage", 5).unwrap(),
            Transition::Install(Phase { command: Command::MaxDepth(1), .. })
        ));
        assert_eq!(r.select((1, 1, 2), "mage", 5).unwrap(), Transition::Keep);
        assert!(matches!(
            r.select((1, 1, 3), "mage", 6).unwrap(),
            Transition::Install(Phase { command: Command::MaxDepth(3), .. })
        ));
        assert!(matches!(
            r.select((1, 1, 4), "mage", 5).unwrap(),
            Transition::Install(Phase { command: Command::MaxDepth(1), .. })
        ));
        assert_eq!(r.select((1, 1, 5), "mage", 11).unwrap(), Transition::Stop);
        assert_eq!(r.select((1, 1, 6), "mage", 5).unwrap(), Transition::Keep);
        assert!(matches!(
            r.select((0, 2, 1), "warrior", 5).unwrap(),
            Transition::Install(Phase { command: Command::MaxDepth(2), .. })
        ));
    }
    #[test]
    fn max_depth_phases_parse_and_mix_with_loops() {
        let p = Plan::parse("[default]\n  level <= 5\n    max-depth 2\n  level > 5\n    max-depth 3\n").unwrap();
        assert_eq!(p.phase("mage", 3).unwrap().unwrap().command, Command::MaxDepth(2));
        assert_eq!(p.phase("mage", 6).unwrap().unwrap().command, Command::MaxDepth(3));
        for bad in ["max-depth", "max-depth 0", "max-depth -1", "max-depth 127", "max-depth 2 3", "max-depth x", "MAX-DEPTH 2", "/max-depth 2"] {
            assert!(Plan::parse(&format!("[default]\n  level <= 5\n    {bad}\n")).is_err(), "{bad}");
        }
        assert!(Plan::parse("[default]\n  level <= 5\n    max-depth 126\n").is_ok());
    }
    #[test]
    fn selects_class_and_boundaries_including_drain() {
        let p = Plan::parse(EXAMPLE).unwrap();
        for (level, last) in [(1, 1), (5, 1), (6, 3), (10, 3), (5, 1)] {
            assert_eq!(p.phase("mage", level).unwrap().unwrap().command, Command::MaxDepth(last));
        }
        assert_eq!(p.phase("warrior", 5).unwrap().unwrap().command, Command::MaxDepth(2));
        assert!(p.phase("mage", 11).unwrap().is_none());
    }
    #[test]
    fn rejects_overlaps_aliases_and_malformed_files() {
        for text in [
            "",
            "[wizzard]\n  level <= 5\n    max-depth 1",
            "[mage]\n",
            "[default]\n  level <= 5",
            "[default]\n  level <= 5\n    ladder",
            "[default]\n  level <= 5\n    loop-level 0 1",
            "[default]\n  level <= 5\n    max-depth 0",
            "[default]\n  level <= 5\n    max-depth 1\n  5 <= level <= 10\n    max-depth 2",
            "[default]\n\tlevel <= 5\n    max-depth 1",
        ] {
            assert!(Plan::parse(text).is_err(), "{text}");
        }
    }
    #[test]
    fn class_gaps_do_not_fall_back_and_missing_class_is_an_error() {
        let p = Plan::parse("[default]\n  level > 1\n    max-depth 2\n[mage]\n  level == 1\n    max-depth 1").unwrap();
        assert!(p.phase("mage", 2).unwrap().is_none());
        let p = Plan::parse("[mage]\n  level == 1\n    max-depth 1").unwrap();
        assert!(p.phase("warrior", 1).is_err());
    }
}
