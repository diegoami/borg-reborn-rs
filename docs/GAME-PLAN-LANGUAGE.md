# Borg game-plan language proposal

Status: owner-requested design, 2026-11-15. This document specifies proposed
behaviour, not an implemented interpreter or an amendment to SPEC/RULES.
Owner review must accept the language and dependencies before implementation.
The current command surface is [ERRANDS.md](ERRANDS.md).

## Purpose and boundaries

A player should be able to describe a whole game: preparation, repeated trips,
progression gates, changes in caution and reserves, and later a plan for a
particular unique. The plan composes the Borg's errands and typed decisions;
it does not send raw keys or duplicate the engine's rules in another language.

A script is a deterministic, bounded, domain-specific program. It can change
behaviour deliberately during a game and explain why it did so. Arbitrary
Python, shell, filesystem access, networking and engine writes are outside the
language. A run without a plan retains its current decisions. An explicitly
configured plan is never silently ignored or replaced by another core.

The language supersedes no accepted plan today. It develops the timed and
four-layer tactical ideas in issue #212 into a persistent player-facing
contract. Counterfactual tools can later submit the same typed plan AST.
Inventory replacement #293 is a prerequisite for meaningful stock directives;
a plan cannot make the current shopping policy satisfy an unsupported target.

## Dependencies and integration gates

| Capability | Dependency before enabling it |
| --- | --- |
| Errand composition and tracing | Existing shell, outcomes, /loop-levels; versioned adapter and fair state view |
| Runtime parameter edits | Accepted P registry plus an explicit hot/cold classification and cache invalidation audit |
| Behaviour profiles and progression | Accepted R frame/modes (R1/R5) and inventory/preparation interfaces |
| Stock/equipment/spell preparation | Accepted R item capabilities (R4), #293, capability catalogue |
| Unique-fight tactics | Accepted R danger/escape/recovery and combat capabilities (R2/R3), then R4 actions |
| External proposals | Separate adviser design and Q facade/host gates; not required for local scripts |
| Owner's “track B” | Unresolved: main currently names P, R and Q, not B. Do not assign a meaning or label automatically. |

Dependencies mean accepted/promoted capability versions, not merely closed task
issues or unreviewed commits copied from another integration branch. Capability
slices may ship after their dependencies; the entire R track need not finish
before the first strategic interpreter. This is not permission to start another
R milestone or bypass its reviews. Resolve track B and the integration branch
with the owner before scheduling implementation. If B means P, use the existing
P review gate; if it is a new behaviour track, define its charter separately.

## Surface syntax and grammar

Files use UTF-8, a required `borg-plan 1;` header, `#` comments, braces and
semicolons. Keywords are case-sensitive; strings use JSON escapes. Modules are
resolved by content hash from an explicitly supplied plan bundle, never fetched
at runtime. The host exports a versioned catalogue of predicates, errands,
behaviours, parameters, item selectors, spells and actions. Names in examples
are proposed catalogue entries, not claims of existing commands/hot setters.

```ebnf
file       = "borg-plan", integer, ";", { import | constant | tactic }, plan ;
import     = "import", string, "as", identifier, ";" ;
constant   = "const", identifier, ":", type, "=", expr, ";" ;
plan       = "plan", identifier, block ;
block      = "{", { statement }, "}" ;
statement  = "let", identifier, ":", type, "=", expr, ";"
           | "set", parameter, "=", expr, ";"
           | "unset", parameter, ";"
           | "run", call, [ "within", expr ], [ "as", identifier ], ";"
           | "do", call, [ "as", identifier ], ";"
           | "wait", "until", expr, "within", expr, ";"
           | "if", expr, block, [ "else", block ]
           | "repeat", ( integer | "forever" ), block
           | "phase", identifier, [ "until", expr ], block
           | "with", bindings, block
           | "on", event, [ "when", expr ], block
           | "fight", selector, "using", identifier, ";"
           | "resume", ";" | "cancel", ";" | "stop", string, ";" ;
bindings   = "(", parameter, "=", expr, { ",", parameter, "=", expr }, ")" ;
tactic     = "tactic", identifier, "for", selector, "{",
             "requires", expr, ";", "prepare", block,
             "engage", expr, ";", "rules", "{", { rule }, "}",
             "abort", expr, "then", block, "}" ;
rule       = "when", expr, "=>", call, ";" ;
expr       = literal | identifier | field | call | "(", expr, ")"
           | ( "not" | "-" ), expr | expr, operator, expr ;
call       = qualified_name, "(", [ expr, { ",", expr } ], ")" ;
operator   = "or" | "and" | "==" | "!=" | "<" | "<=" | ">" | ">="
           | "+" | "-" | "*" | "/" ;
```

Lexical catalogue references (`parameter`, `selector`, `event`, `field`, `type`)
are typed names, not further embedded text commands. Integers are checked i64;
quantities use catalogue constructors such as `game_turns(5000)`, `count(3)`
and `hp_fraction(0.5)`. Arithmetic overflow/division by zero is an error. Unary
operators bind before multiply/divide, then add/subtract, comparisons, `and`,
`or`; boolean operations short-circuit. Comparison chaining is rejected.
Functions are pure, catalogue-defined queries; no user recursion or unbounded
collections. Unknown knowledge is `Unknown`, not false or zero: comparisons
propagate it, and only a known true condition fires. `known(x)` and explicit
coalescing handle uncertainty. Re-evaluated fields are distinguished from
`let` snapshots; event handlers bind immutable event data explicitly.

`run` awaits a strategic errand; `do` requests one typed action. `fight` installs
a bounded tactical frame. Expressions cannot act or draw engine RNG. Branches
and finite loops do not themselves consume game turns. Every dynamic cycle
must reach a yielding operation; static checks reject obvious empty cycles,
and a host instruction budget catches cycles dependent on runtime conditions.
Instruction exhaustion suspends with an error and never spins inside a think.

## Strategic example

```borg-plan
borg-plan 1;
plan cautious_start {
  on town_required when event.reason != "plan_visit" {
    run town() within game_turns(5000) as home;
    if home.status == "done" {
      run shopping() within game_turns(3000);
      resume;
    } else { stop "Town recovery failed"; }
  }
  phase early until player.level >= 5 {
    with (behaviour = profile("cautious"),
          core.caution_flee_danger_hp_multiplier = 1) {
      repeat forever {
        run loop_levels(0, 2) within game_turns(10000) as trip;
        if trip.status == "scared" { stop "Player handover requested"; }
        if trip.status == "budget" { wait until ready_for(2) within game_turns(1000); }
      }
    }
  }
  phase established { run loop_levels(3, 5); }
}
```

This example relies on the proposed scheduler semantics: a phase's `until`
condition is a level-triggered observer checked at each safe boundary, even
while a yielding child runs. It cancels that child and its infinite repetition,
then advances the phase. A child timeout produces an outcome; it does not reset
the enclosing phase deadline. The sample parameter exists in the current P
registry, but live mutation is proposed and cannot be used today. Profile and
query names likewise require catalogue implementation before execution.

`run` and `do` return typed outcomes: `done`, `aborted`, `scared`, `budget`,
`impossible`, `cancelled` or `error`, plus reason, actual effects and elapsed
turns. An `as` binding exposes the outcome; without one, non-success propagates
to the enclosing frame's failure event, then hands over if unhandled. A wait
that expires is `budget`. `stop` is not catchable by the script. Symbolic inputs
are declared in a typed bundle manifest, validated before startup; an absent
or wrongly typed `input(...)` is an error, never an implicit value. Exact host
resource ceilings are implementation settings to be reviewed before release.

## Execution, events and interruption

There is one active strategic frame, a stack of scoped overrides, and at most
one tactical controller. No parallel player actions. At a completed game-command
boundary the host: finishes the previous action receipt; refreshes fair state;
resolves safety, manual cancellation and events; atomically commits permitted
configuration changes; evaluates phase guards; then chooses at most one action.
Stores, prompts, multi-key actions and pending Recall are not safe points for
mid-command parameter/controller replacement.

Priority is engine legality and accepted safety invariants, manual stop,
core-required emergency/town recovery, tactical abort, phase/event transitions,
tactic rule, strategic errand, then ordinary core policy. This requires typed
R hooks; today's post-caution errand hook cannot implement arbitrary tactical
overrides alone. A preference cannot suppress an accepted escape invariant.
Any proposed ability to relax an invariant requires a separately reviewed
contract and a distinct experimental profile, not an ordinary parameter edit.

`on` declares a lexical handler for a host-defined event. Handlers are selected
by the fixed priority above, innermost lexical scope first, source order for
equal events. One handler runs at a time; events have causal IDs, enter/exit
edges and a bounded queue. The same unresolved town request does not re-enter a
handler every think. Overflow and handler failure are explicit outcomes.
A new emergency can interrupt a handler; lower-priority events wait and are
rechecked against current state before delivery. `resume` is legal only in a
handler and returns to its suspended parent; `cancel` cancels the current
frame and produces a cancellation outcome. A top-level `stop` hands over to
the player and clears descendants.

The scheduler owns an interruption only when the child does not already own it.
In particular `/loop-levels` already suspends for town and restarts its first
endpoint: its adapter exposes that child state rather than scheduling a second
town trip. For other errands, resumption policy (`restart`, `continue`, or
`handover`) is declared in the catalogue and logged, never inferred from a
name. After town, restart/continuation is revalidated for readiness and supplies.
Failed preparation, unavailable Recall and insufficient funds cannot silently
become `done`. Existing finite commands retain their queue/abort semantics
through adapters; the language does not retroactively change the slash parser.

## Behaviour and parameter changes during play

The host publishes a registry with type, units, bounds, semantics, capability
version, origin, mutability (`startup`, `safe_point`, `town_only`), invalidation
requirements and permitted scopes. Current parameters are loaded at startup;
this proposal requires an audit before any are advertised as hot. Cached
danger, paths, preparedness and item valuations must be invalidated/recomputed
as specified by the affected parameter family, including consistency on restore.

Precedence is defaults < run profile < persistent plan < phase/with < tactic
< manual override. Safety validation is independent of that precedence.
`set` updates the current scope, `unset` removes that scope's value, and `with`
restores the *current lower layer* on every exit, timeout and cancellation.
It never restores an old snapshot over a later manual change. No permanent
mutation of borg.txt is implied. Behaviour profiles expand into a versioned
bundle of explicit settings and capabilities, so their names cannot conceal
arbitrary changes. Engine constants, birth options and backend selection are
not runtime plan parameters.

A batch is validated as a transaction before any key is queued. Invalid keys,
unsupported mutability, unsafe combinations or failed cache recomputation
reject the whole batch. Successful commits increment a configuration epoch and
record complete effective values/hash plus origin; a plan cannot see half an
update. Reloading a plan uses the same rule, at a safe point: validate first,
then explicitly restart or migrate named compatible frames. A syntax error
leaves the old plan running, with a visible diagnostic. Startup without a
valid requested plan fails loudly instead of falling back to default play.

## Preparation and unique tactics

A unique is identified by a stable engine-version-qualified race key, with
names as checked aliases. Requirements use fair, known abilities and stock:
healing/escape reserves, known resistances, ammo, equipment, castability,
charges and failure chances. Quantities are declared plan inputs/hypotheses,
not new hard-coded recommendations. Preparation can shop, retrieve from home,
equip, rest or decline the encounter; it cannot create unavailable supplies.
The reserve gate also protects items from sale/discard (#293).

```borg-plan
tactic prepared_unique for unique("catalogue:target") {
  requires reserves_meet("target-kit") and ready_for_encounter(target);
  prepare {
    run stock("target-kit") within game_turns(3000);
    run equip_loadout(loadout("target-kit"));
  }
  engage visible(target) and requirements_hold();
  rules {
    when hp_fraction_now() < input("retreat_hp") => escape(best_legal_escape());
    when not buff_active("speed") and can_use("speed") => use("speed");
    when can_cast("catalogue:attack-spell", target) => cast("catalogue:attack-spell", target);
    when can_fire(target) => fire(target);
    when can_melee(target) => melee(target);
  }
  abort not requirements_hold() or not escape_available() then {
    run town() within game_turns(5000);
    stop "Reassess this encounter";
  }
}
```

A tactic moves through Preparing → Armed → Engaged → Aborting → Finished.
`requires` is the engagement gate checked after preparation; it does not
prevent preparing a missing kit. Preparation has a deadline and can fail
explicitly. The scripted `abort` guard applies once armed/engaged; native
safety still applies during preparation. A live unprepared encounter does
not run a shopping errand under attack: it declines engagement or yields to
native escape. Encounter end releases overrides; re-engagement rechecks the
requirements and obtains a new bounded frame.

Rules select the first known true, currently legal candidate; availability is
checked again immediately before execution. A failed or no-effect action
returns a receipt; per-rule bounded retries/cooldowns come from the capability
catalogue and cannot loop forever on a nonexistent spell. Recheck engagement
and reserves every boundary, not only before the first attack. No rule matching
means normal safe core action for that think, not a forced attack. If an abort
route fails, release to the native emergency policy/player with the failure
recorded; never insist on finishing a scripted combination. Unknown abilities
or enemy health stay unknown, with legal fallbacks. Concrete race/spell/item
IDs, kits, thresholds and exceptions are reviewed and engine-grounded when
actual tactic packs are authored; this illustrative tactic is not a recipe
for a particular unique.

## Persistence, observability and replay

Record plan bytes/hash, import hashes, language/catalogue versions, backend,
base parameters and every effective epoch. Each decision records frame IDs,
source spans, event causes, predicate results (including Unknown), overrides,
errand/action receipt, rejected candidates and interruption/resumption reason.
The player can inspect `plan status`, `plan explain`, `plan pause/resume` and
`plan cancel`; exact command spelling is an implementation deliverable.

A plan checkpoint contains PC/stack, typed locals, event queue/latches, budgets,
active child and override layers. It is tied to character identity and the
engine/Borg checkpoint. An ordinary Angband save is not sufficient to restore
this state. If full Borg restoration is unavailable, replay the lead-in or
start a separately labelled fresh-state branch. Reincarnation drops the old
plan unless an explicit new-character policy creates a new instance. Partial
or incompatible checkpoints fail closed; do not claim exact replay from them.

## Delivery and acceptance

1. Schema/catalogue, parser/type checker and dry-run explanation; no actions.
2. Strategic errands and bounded scheduler, including phase exit during an
   infinite child, nested town handling and existing slash compatibility.
3. Audited runtime parameter/profile transactions and restoration semantics.
4. Stock/equipment requirements and unique-tactic controller on accepted R APIs.
5. Plan checkpoint/replay qualification and optional external-proposal bridge.

Before each slice: name accepted dependencies, task branch/track and review
pin. Tests must include unknown names/types, unreachable targets, unit/bounds
errors, non-yielding loops, instruction/queue limits, stale events, town recovery,
manual changes under a scoped override, atomic rollback and pending prompts.
An inactive or unreachable schedule must preserve keys/messages/states;
active plans must reproduce their recorded transitions and effective epochs.
Use actual engine fixtures for actions and failed effects, then the existing
CI/reference/death gates. Improvement claims remain prespecified zoo work;
writing a script does not establish better survival. The owner chooses the
reviewer and later accepts concrete capability/limit tables, tactic packs,
track B's meaning, and implementation scheduling. No prototype or performance
claim accompanies this design PR.

The companion external-advice proposal is tracked in [#306](https://github.com/diegoami/borg-reborn-rs/issues/306); local plans do not depend on an LLM.
