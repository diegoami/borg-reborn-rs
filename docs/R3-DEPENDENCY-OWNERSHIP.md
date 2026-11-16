# R3 dependency ownership

The exact M13 baseline is borg-rs `1443c2d1b97d110ffbf5ae420d6c6b97ac15f7ea`.
Attack owns 71 production functions and seven public entries; defence owns
56 functions and its single public entry. The named-read projection and
the policy branches, scores, ordering, goals and temporary simulation
writes are independent receiving code. Source audits compare normalized
function bodies, strings, character literals and module declarations.

R3-DEPENDENCIES.json inventories all 112 explicit production dependency
signatures. This extends the R2 boundary's 68 functions with 31 ordinary
attack adapters, four special adapters and nine defence adapters. The
inventory is a review aid; compilation and runtime comparisons also remain
required. The syntactic source gate cannot prove arbitrary Rust semantics.

| Boundary | Ownership and replay rule |
| --- | --- |
| Mutable services | Record complete input memory, result and post-state. Replay checks input before applying post-state. A prior wrong port mutation cannot be hidden. |
| Read-only state queries | Record the result and compare complete input memory before replay. Their engine reads happen only in the driving program. |
| Engine facts and RNG | Record ordered name/arguments/results. Replay consumes values without executing the engine or drawing RNG. Reordered pure requests still disagree. |
| Metadata results | AttackRace and AttackKind contain values only. AttackKind.aware is captured
  at query time and may change in the engine; it is not immutable knowledge. Terminal geometry is a plain-value record. Static names/constants borrow immutable data for the process lifetime. No engine object token is exposed to policy. |
| Items and spells | Items and spell tables remain native owned memory. Deliberate native local values stay at the same expression points. Borrowed named reads are reacquired after mutable services. Item argument diagnostics include the complete owned item and its note-origin descriptor. |
| Motion output parameters | Record initial caller coordinates plus target arguments, then replay both resulting coordinates. The regression control rejects changed initial coordinates. |
| Town journey forecast | Treat forecast as an effect because it mutates trip memory. Return only the required turn count; attack retains the original reserve arithmetic and per-evaluation cache. |
| Attack/escape cross-calls | Defence uses recorded services whose public facades independently compare the owned attack/escape programs. These calls suspend and restore the outer tape; they do not count as ownership of shared service bodies. |

Permanent-buff policy (`borg_fight_perm`), preparation, flow/projection,
spell/item services and engine integration remain explicitly shared. They
are not claimed as independently ported by this milestone. The source
gate forbids attack or defence from delegating its own policy to the
corresponding native-result API. All other shared callees still require
manual inspection when the donor changes, even if a watcher reports no
changed owned function.

The snapshot is owned Rust policy memory, not an engine snapshot. Native
engine access still checks the actual root state's access token. A replay
fork cannot use a copied token to obtain engine objects. Effects execute
once; nested callbacks save/restore the suspended tape and pause depth.
The recorder mutex is released before policy or engine services execute.

Three exact named mutation adapters cover mana, confusion resistance and
Free Action. Grim Purpose retains the native boolean save and 0/1 restore,
including nonbinary inputs. Other temporary buffs remain explicit owned
memory writes in their original order. Equality checks include all owned
memory and the final simulation state; native early-return behavior is
preserved rather than normalized by the port.

Positive entry counts prove invocation, not every branch or successful
effect. The 43 declared defence cases cover every selector entry and
selected restoration/rare simulation paths over loaded engine tables;
they are not natural endgame progression. Full unsealed play, literal
classification/restoration faults with exact re-ports, real nesting and
effect/RNG-once controls provide separate evidence. These checks establish
parity on their covered inputs, not a survival or speed improvement.
