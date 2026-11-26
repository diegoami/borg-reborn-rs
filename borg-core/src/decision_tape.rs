//! borg-reborn-rs (#286): execute effects once, replay their values and state.
//! This is an owned Rust policy snapshot, never a cloned or rewound engine.
use crate::state::BorgState;
use std::{any::Any, cell::{Cell, RefCell}, collections::VecDeque, panic::{catch_unwind, AssertUnwindSafe}};

struct Event {
    name: &'static str,
    args: String,
    before: Option<Box<BorgState>>,
    after: Option<Box<BorgState>>,
    result: Box<dyn Any>,
}
enum Tape { Record(Vec<Event>), Replay(VecDeque<Event>) }
thread_local! {
    static TAPE: RefCell<Option<Tape>> = const { RefCell::new(None) };
    static CONTEXT: RefCell<serde_json::Value> = const { RefCell::new(serde_json::Value::Null) };
    static PAUSED: Cell<usize> = const { Cell::new(0) };
}
pub fn in_transaction() -> bool { TAPE.with(|t|t.borrow().is_some()) }
pub fn active() -> bool { in_transaction() && PAUSED.with(|p|p.get()==0) }
struct Pause;
impl Pause { fn new() -> Self { PAUSED.with(|p|p.set(p.get()+1)); Self } }
impl Drop for Pause { fn drop(&mut self) { PAUSED.with(|p|p.set(p.get()-1)); } }
#[derive(Debug)]
pub struct Divergence(pub String);
fn abort(message:String) -> ! {
    #[cfg(test)] { std::panic::panic_any(Divergence(message)) }
    #[cfg(not(test))] {
        // This workspace deliberately builds panic=abort. Persist a request
        // divergence before a controlled stop; never execute an unrecorded
        // dependency or pretend the remainder of the replay is meaningful.
        use std::io::Write;
        let mut row=CONTEXT.with(|c|c.borrow().clone());
        // The provider owns the versioned diagnostic contract. Historical
        // direct tape callers without a version retain their original schema.
        if row.get("schema").is_none() {
            row["schema"]=serde_json::json!("borg-reborn-rs/shadow@1");
        }
        row["replay_error"]=serde_json::json!(message);
        row["replay_stopped"]=serde_json::json!(true);
        let saved=std::env::var_os("BORG_RUN_DIR").ok_or_else(||std::io::Error::other("missing shadow run directory"))
            .and_then(|dir| {
                let mut file=std::fs::OpenOptions::new().append(true).open(std::path::PathBuf::from(dir).join("shadow.jsonl"))?;
                serde_json::to_writer(&mut file,&row)?;file.write_all(b"\n")?;file.flush()
            });
        eprintln!("stateful shadow replay stopped: {message}");
        if let Err(e)=saved {eprintln!("cannot persist stateful mismatch: {e}");}
        std::process::exit(2)
    }
}
fn replaying() -> bool { TAPE.with(|t|matches!(*t.borrow(),Some(Tape::Replay(_)))) }
fn pop(name:&'static str,args:&str,state:Option<&BorgState>) -> Event {
    let event=TAPE.with(|t| match t.borrow_mut().as_mut() {
        Some(Tape::Replay(q))=>q.pop_front(), _=>unreachable!(),
    });
    let Some(event)=event else { abort(format!("extra request: {name} {args}")) };
    if event.name!=name || event.args!=args {
        abort(format!("expected {} {}; requested {name} {args}",event.name,event.args));
    }
    match (event.before.as_deref(),state) {
        (Some(a),Some(b)) if a!=b=>abort(format!("policy state differs before {name} {args}")),
        (Some(_),None)|(None,Some(_))=>abort(format!("state boundary differs at {name}")),
        _=>{},
    }
    event
}
fn value<T:'static>(event:Event) -> T {
    *event.result.downcast::<T>().unwrap_or_else(|_|abort(format!("result type differs at {}",event.name)))
}
fn push(event:Event) { TAPE.with(|t|match t.borrow_mut().as_mut() { Some(Tape::Record(q))=>q.push(event),_=>unreachable!() }); }
/// Stateful dependency. Replay verifies its input state before applying the
/// recorded post-state, so a port's earlier mutations cannot be hidden.
pub fn effect<T:Clone+'static>(st:&mut BorgState,name:&'static str,args:String,f:impl FnOnce(&mut BorgState)->T) -> T {
    if !active() { return f(st); }
    if replaying() {
        let mut event=pop(name,&args,Some(st));
        *st=*event.after.take().unwrap_or_else(||abort(format!("missing effect state: {name}")));
        return value(event);
    }
    let before=Box::new(st.clone());
    let result={ let _pause=Pause::new(); f(st) };
    push(Event {name,args,before:Some(before),after:Some(Box::new(st.clone())),result:Box::new(result.clone())});
    result
}
/// Read-only dependency; its complete input memory is still checked.
pub fn query<T:Clone+'static>(st:&BorgState,name:&'static str,args:String,f:impl FnOnce()->T) -> T {
    if !active() { return f(); }
    if replaying() { return value(pop(name,&args,Some(st))); }
    let result={ let _pause=Pause::new(); f() };
    push(Event{name,args,before:Some(Box::new(st.clone())),after:None,result:Box::new(result.clone())}); result
}
/// Engine query or RNG draw. Only the driving policy calls the engine.
/// Replay validates the request and returns its recorded value.
pub fn engine<T:Clone+'static>(name:&'static str,args:String,f:impl FnOnce()->T) -> T {
    if !active() { return f(); }
    if replaying() { return value(pop(name,&args,None)); }
    let result={ let _pause=Pause::new(); f() };
    push(Event{name,args,before:None,after:None,result:Box::new(result.clone())}); result
}

/// Compare two independent policies. The first drives real effects; the
/// second receives only recorded responses on a private owned state.
/// On request divergence the replay stops; it never falls back to the engine.
pub fn compare<T:Clone+PartialEq>(st:&mut BorgState,context:serde_json::Value,drive:impl FnOnce(&mut BorgState)->T,replay:impl FnOnce(&mut BorgState)->T) -> (T,Option<T>,Result<(),String>,usize,std::collections::BTreeMap<&'static str,u64>) {
    // An opaque dependency may re-enter a ported policy (or the Borg).
    // Save its tape and pause depth; the inner transaction owns its own
    // record/replay, then restores the suspended dependency exactly.
    let _scope=Scope { saved:TAPE.with(|t|t.borrow_mut().take()), paused:PAUSED.with(|p|p.replace(0)), context:CONTEXT.with(|c|c.replace(context)) };
    let mut fork=st.clone();
    TAPE.with(|t|*t.borrow_mut()=Some(Tape::Record(Vec::new())));
    let driven=catch_unwind(AssertUnwindSafe(||drive(st)));
    let recorded=TAPE.with(|t|t.borrow_mut().take());
    let driven=match driven {Ok(v)=>v,Err(e)=>std::panic::resume_unwind(e)};
    let events=match recorded {Some(Tape::Record(v))=>v,_=>unreachable!()}; let count=events.len();
    let mut visits=std::collections::BTreeMap::new();
    for event in &events { if let Some(name)=event.name.strip_prefix("policy::") { *visits.entry(name).or_insert(0)+=1; } }
    TAPE.with(|t|*t.borrow_mut()=Some(Tape::Replay(events.into())));
    let result=catch_unwind(AssertUnwindSafe(||replay(&mut fork)));
    let remaining=TAPE.with(|t|match t.borrow_mut().take() {Some(Tape::Replay(q))=>q.len(),_=>unreachable!()});
    let replayed=result.as_ref().ok().cloned();
    let agreement=match result {
        Err(e)=>match e.downcast::<Divergence>() {Ok(e)=>Err(e.0),Err(e)=>std::panic::resume_unwind(e)},
        Ok(v) if v!=driven=>Err("decision return differs".into()),
        Ok(_) if remaining!=0=>Err(format!("{remaining} recorded requests were not consumed")),
        Ok(_) if fork!=*st=>Err("final policy state differs".into()),
        Ok(_)=>Ok(()),
    };
    (driven,replayed,agreement,count,visits)
}


/// Read-only policy comparison. Both programs borrow the SAME immutable
/// caller memory; neither receives &mut or a copied engine token. Ordered
/// engine/RNG responses still execute once and replay without fallback.
/// Only audited immutable entries may use this boundary; stateful entries
/// retain compare's complete fork and before/after equality checks.
pub fn compare_read<T:Clone+PartialEq>(st:&BorgState,context:serde_json::Value,drive:impl FnOnce(&BorgState)->T,replay:impl FnOnce(&BorgState)->T) -> (T,Option<T>,Result<(),String>,usize,std::collections::BTreeMap<&'static str,u64>) {
    // An opaque dependency may re-enter a ported policy (or the Borg).
    // Save its tape and pause depth; the inner transaction owns its own
    // record/replay, then restores the suspended dependency exactly.
    let _scope=Scope { saved:TAPE.with(|t|t.borrow_mut().take()), paused:PAUSED.with(|p|p.replace(0)), context:CONTEXT.with(|c|c.replace(context)) };
    TAPE.with(|t|*t.borrow_mut()=Some(Tape::Record(Vec::new())));
    let driven=catch_unwind(AssertUnwindSafe(||drive(st)));
    let recorded=TAPE.with(|t|t.borrow_mut().take());
    let driven=match driven {Ok(v)=>v,Err(e)=>std::panic::resume_unwind(e)};
    let events=match recorded {Some(Tape::Record(v))=>v,_=>unreachable!()}; let count=events.len();
    let mut visits=std::collections::BTreeMap::new();
    for event in &events { if let Some(name)=event.name.strip_prefix("policy::") { *visits.entry(name).or_insert(0)+=1; } }
    TAPE.with(|t|*t.borrow_mut()=Some(Tape::Replay(events.into())));
    let result=catch_unwind(AssertUnwindSafe(||replay(st)));
    let remaining=TAPE.with(|t|match t.borrow_mut().take() {Some(Tape::Replay(q))=>q.len(),_=>unreachable!()});
    let replayed=result.as_ref().ok().cloned();
    let agreement=match result {
        Err(e)=>match e.downcast::<Divergence>() {Ok(e)=>Err(e.0),Err(e)=>std::panic::resume_unwind(e)},
        Ok(v) if v!=driven=>Err("read-only decision return differs".into()),
        Ok(_) if remaining!=0=>Err(format!("{remaining} recorded requests were not consumed")),
        Ok(_)=>Ok(()),
    };
    (driven,replayed,agreement,count,visits)
}

struct Scope { saved:Option<Tape>, paused:usize, context:serde_json::Value }
impl Drop for Scope { fn drop(&mut self) {
    TAPE.with(|t|*t.borrow_mut()=self.saved.take());
    PAUSED.with(|p|p.set(self.paused));
    CONTEXT.with(|c|*c.borrow_mut()=self.context.take());
} }

#[cfg(test)]
mod tests {
    use super::*;
    fn context()->serde_json::Value {serde_json::json!({"point":"fixture"})}
    #[test]
    fn readonly_queries_replay_once_and_preserve_the_parent_transaction() {
        let mut st=BorgState::default();
        let draws=Cell::new(0);
        let (_,_,agreement,_,_)=compare(&mut st,context(),|st| {
            let result=effect(st,"opaque","".into(),|st| {
                let (answer,replayed,agreement,_,_)=compare_read(st,context(),|_| {
                    engine("read-query","x".into(),||{draws.set(draws.get()+1);7i32})
                },|_| engine("read-query","x".into(),||panic!("read replay executed a query")));
                assert_eq!(replayed,Some(answer)); assert!(agreement.is_ok());
                st.borg.goal.fleeing=true;
                answer
            });
            engine("parent-tail","".into(),||9i32)+result
        },|st| {
            let result:i32=effect(st,"opaque","".into(),|_|panic!("outer replay executed an effect"));
            engine::<i32>("parent-tail","".into(),||panic!("parent replay executed a query"))+result
        });
        assert!(agreement.is_ok()); assert!(st.borg.goal.fleeing); assert_eq!(draws.get(),1);
    }
    #[test]
    fn readonly_return_and_request_order_faults_reject() {
        let st=BorgState::default();
        let (_,_,agreement,_,_)=compare_read(&st,context(),|_|true,|_|false);
        assert_eq!(agreement,Err("read-only decision return differs".into()));
        let (_,_,agreement,_,_)=compare_read(&st,context(),|_| {
            let _:i32=engine("first","".into(),||1);
            engine("second","".into(),||2i32)
        },|_| engine("second","".into(),||panic!("reordered read escaped replay")));
        assert!(agreement.unwrap_err().contains("expected first"));
    }

    #[test]
    fn effect_and_rng_execute_once_and_complete_state_is_compared() {
        let mut st=BorgState::default(); let effects=Cell::new(0); let draws=Cell::new(0);
        let (value,replayed,match_,count,_)=compare(&mut st,context(),|st| {
            let n=engine("rng","100".into(),||{draws.set(draws.get()+1);37i32});
            effect(st,"keys","escape".into(),|st|{effects.set(effects.get()+1);st.borg.goal.fleeing=true;n})
        },|st| {
            let n:i32=engine("rng","100".into(),||panic!("replay drew RNG"));
            let _:i32=effect(st,"keys","escape".into(),|_|panic!("replay executed an effect"));
            n
        });
        assert_eq!(value,37);assert_eq!(replayed,Some(37));assert!(match_.is_ok());assert_eq!(count,2);
        assert_eq!(effects.get(),1);assert_eq!(draws.get(),1);assert!(st.borg.goal.fleeing);
        assert!(!in_transaction());
    }
    #[test]
    fn input_state_divergence_cannot_be_hidden_by_the_post_state() {
        let mut st=BorgState::default();
        let (_,_,result,_,_)=compare(&mut st,context(),|st|effect(st,"action","".into(),|st|{st.borg.goal.leaving=true;}),|st| {
            st.borg.goal.fleeing=true;
            effect(st,"action","".into(),|_|panic!("must not execute"));
        });
        assert!(result.unwrap_err().contains("state differs before"));
        assert!(st.borg.goal.leaving);assert!(!st.borg.goal.fleeing);
    }
    #[test]
    fn changed_request_return_and_final_state_are_rejected() {
        for variant in 0..4 {
            let mut st=BorgState::default();
            let (_,_,result,_,_)=compare(&mut st,context(),|st|effect(st,"action","1".into(),|_|true),|st| {
                if variant==0 {return effect(st,"action","2".into(),|_|false);}
                if variant==1 {return false;}
                let v=effect(st,"action","1".into(),|_|false);
                if variant==2 {st.borg.goal.fleeing=true;v} else {!v}
            });
            assert!(result.is_err());
        }
    }
    #[test]
    fn opaque_dependency_can_run_a_nested_transaction_without_replaying_effects() {
        let mut st=BorgState::default();let effects=Cell::new(0);
        let (_,_,result,_,_)=compare(&mut st,context(),|st|effect(st,"opaque","".into(),|st| {
            let (v,_,check,_,_)=compare(st,context(),|st|effect(st,"inner","".into(),|st|{effects.set(effects.get()+1);st.borg.goal.fleeing=true;true}),
                |st|effect(st,"inner","".into(),|_|panic!("inner replay effect")));
            assert!(check.is_ok());v
        }),|st|effect(st,"opaque","".into(),|_|panic!("outer replay effect")));
        assert!(result.is_ok());assert_eq!(effects.get(),1);assert!(st.borg.goal.fleeing);assert!(!in_transaction());
    }
}

/// Record/compare an internal ported entry and its explicit arguments.
pub fn mark(name:&'static str,args:String) { engine(name,args,||()); }
