# D0001 — stegocentipede death, seed11

[Discuss this death](https://github.com/diegoami/better-borg-c/issues/22) · [Feedback process](../FEEDBACK.md)

An ordinary level9 Human Warrior entered melee with a tiger, lost health and
failed to teleport. After killing it, the Borg kept attacking a stegocentipede
at21HP and15HP and died at **engine turn174862, depth12, HP−4**. Raw native
cause: **a stegocentipede**. Six known Speed potions remained in the pack; no
known healing or escape stock was carried, while four Phase Door scrolls were
at Home. Four unidentified Smoky Potions remained; their true effects are not
established by this recording. Zero known-heal counters do not prove that no
healing effect existed among unidentified items.

The death and its short replay windows are verified. The reason those existing
stock/defense decisions were made is **not yet a proved bug**. An attack score61
is not established expected HP damage. A potion might cost an action at an
unsafe moment; this record does not claim that drinking one would rescue it.

## Exact version and build that died

- Consumer source: **337e50eb6c732bf6c908ab0d7c22b12b3a82b7d3**, before the later
  ring carry. Later fixes never rewrite this record.
- Official baseline: **e473720ed88a0f0c7ca66815cef3000eb4a38bc3**.
- Carried corrections at that moment: CBUG-0066/source82, CBUG-0002/source14,
  CBUG-0094/source110 and CBUG-0004/source26. Exact source/applied patch pins are
  in [record.json](record.json), read from that historical commit, not main.
- Actual observed binary SHA256:
  `234841aa91d1ae552d280c122c183c8aaf098af8af5b9e39aed873292aa88a79`.
- Compiler GCC15.2.0, Linux x86_64, CMake Debug, Borg/X11/test frontend, declared
  fork-snapshot observer. Full source-archive/tool/data hashes and build commands:
  [build.json](build.json). Actual options, Borg configuration/input hashes,
  private/game RNG states, commands and environment: [record.json](record.json).
- Ordinary native birth/default point buy, seed11; no cheat death, altered
  damage or human gameplay intervention. The script contains birth and ordinary
  Borg activation acknowledgements. Engine rules are unchanged.

## Recording and verified pre-death windows

[Selected exact logs](events.log) preserve Borg observations from the final200
world turns and the native recent-message buffer at death. Buffer messages have
no independent timestamps; duplicate channels are not extra game events.
[Terminal native map/inventory/status](terminal.map.txt) retains the actual dump.
This is a text recording, not a video or invented every-turn engine simulation.
Native maps include diagnostic/internal state; none is fed into the Borg.

| Point | Actual engine turn | Actual HP | Native snapshot |
| --- | --- | --- | --- |
| T−20 | 174842 | 39 | [snapshot](snapshots/T-minus-20.txt) |
| T−15 | 174847 | 30 | [snapshot](snapshots/T-minus-15.txt) |
| T−10 | 174852 | 31 | [snapshot](snapshots/T-minus-10.txt) |
| T−5 | 174857 | 21 | [snapshot](snapshots/T-minus-5.txt) |
| Death | 174862 | −4 | [terminal](terminal.map.txt) |

Every window reruns from the original start and has the same starting save and
all prior observed decisions/state. Stops occur immediately after the native
world-turn increment, before remaining processing at that tick. HP31 after30 is
an observed native state, not interpolation. World turns, energy, command
boundaries and Borg-think calls are distinct. Live snapshot dumps inherit native
'Killed'/'Death' labels; their actual outcomes are cap/alive in record.json.

## Evaluation and diagnosis

At turn174811 the Borg faced a66HP tiger at full84HP. It logged39HP and
'fleeing (failed to teleport)' at174840. The tiger died by174854. A following
stegocentipede then received attacks while the player was21HP and15HP; native
bite/sting messages immediately precede 'You die'. Monsters are naturally
speed120; the terminal player is normal speed110. The actor's native blows are
physical HURT, not a presumed poison death. Terminal poison status is0.

The carried build and its exact repeat have **byte-identical full traces and
starting saves**, the same death turn/cause, character state and both RNG states.
A separate pristine e473720ed build produced the same complete trace and death.
This locates the observed behavior in original C, not a regression introduced
by these four carries. It still does not prove a defect in that behavior.

The existing speed-defense code compares current/simulated danger and conserves
consumable speed more strictly than repeatable sources. Actual p1/p2 and chosen
eligibility branch were not recorded at the fatal fight. The Home stock and
pack facts also do not explain why resources were left or consumed. Avoid
turning either source-reading hypothesis into a new threshold or strategy.

## Actionable items

[Investigation22](https://github.com/diegoami/better-borg-c/issues/22) was opened
before any change. Its actions are also machine-readable in
[assessment.json](assessment.json):

1. Observe actual existing Speed eligibility/p1/p2 from the verified pre-fight
   state, using declared diagnostics that preserve the original parent run.
2. Trace last town/Home choices and stock constraints. Establish what was known
   and whether a real implementation fault or an intended rule caused this.
3. Pristine comparison is complete: same death. If a reusable original-C defect
   is proved, duplicate-check/coordinate source207, file it in
   neo-angband-tracker, then source fix and a separate consumer carry.

No new tactic is implemented. Ready bugs always pre-empt this investigation.

## Reproduction, saves and sampling limits

From a clone containing the historical commit, prepare the actual source with
its corresponding harness, not a silently newer Borg:

```sh
git worktree add --detach .worktrees/D0001 337e50eb6c732bf6c908ab0d7c22b12b3a82b7d3
(cd .worktrees/D0001 && flock /tmp/angband-validation.lock python3 tools/measure-native.py prepare --ref HEAD --observe --output artifacts/D0001-build)
(cd .worktrees/D0001 && flock /tmp/angband-validation.lock python3 tools/measure-native.py run --build artifacts/D0001-build --output artifacts/D0001-death --seed 11 --cap 400000 --timeout 180 --stall-timeout 30)
```

The legacy runner did not capture runtime calendar/timezone context. Native
monster selection uses local calendar rules for seasonal monsters on December
24–26. Exact observations were repeated in the retained execution environment;
cross-calendar replay is not qualified. Future runs record this context without
changing the native clock. Do not backfill this old death with today's date.

Repeat with a new output directory. For windows, use caps174842/174847/174852/
174857 with otherwise identical conditions. Same native data/toolchain identity
is required for exact reproduction; recorded binary hashes name actual builds,
not a cross-toolchain bit-for-bit promise.

The recent living save is from174751/84HP. It was actually loaded and advanced
to174752 with an ordinary wait as a separate diagnostic, without activating Borg.
A native save omits full Borg memory and compacts monsters. That is **not proof
of exact mid-run continuation**. No living/reloadable post-death save is claimed.

The pilot first survived cap100000 in33.046s (level7/max depth7/72HP). The longer
attempt died in56.351s; unchanged repeat and pristine reference are controls,
not additional independent sampled lives. This is **one starting condition and
one encounter trajectory**, not a survival-rate estimate. Window caps are
intentional replay stops, not new ordinary survivors. Failures remain recorded.

Full traces/result JSON, saves, original native maps and binaries remain outside
Git under persistent artifacts/death-pilot1, with hashes/pinned recovery on
[issue19](https://github.com/diegoami/better-borg-c/issues/19). Committed files
are explicitly curated report evidence requested by the owner. No workstation
/tmp dependency, private profile or strategy-training dataset is introduced.

## Additional five-carry reference from deep-impact-v2

The prospectively declared paired batch33 ran actual five-carry source
3638b690525cd8a381cb106b4c5ee40652babde2 with the later ring correction.
It produces the same byte-identical full trace and starting save as this death,
including turn174862/cause/final game+Borg RNG state. That additional reference
is recorded in record.json and reference-2.build.json; the original death
identity remains337e50eb. Its runtime calendar context was actually captured.
This is a fourth death process attempt in the same starting family, not another
independent sampled life. Six-carry45eac63bd instead survives cap400000 along
a different trajectory; it is a censored comparison, not a replay or a proof
that a particular healing decision rescued this encounter.

## Owner feedback: inventory and survival reserves

[Owner comment on22](https://github.com/diegoami/better-borg-c/issues/22#issuecomment-6685775530)
asks for escape/healing/survival stock to take priority over spare equipment.
The pack held three spare weapons, four armor pieces and an empty Magic Missile
wand, with no active swap equipment. It also had **19occupied/23slots, four free**
(native empty-slot trait4); capacity alone does not explain absence of escapes.
The actual retention/sale/identification purpose, gold and weight need tracing.

Phase Door was carried and used earlier: last recorded BORG read173794, then
`Leaving (restock) restock phase door`173901. At173834 it had already logged no
up stairs found/going down. These observations do not prove a departure with
zero scrolls or justify adding an omniscient escape item at the fatal action.
Investigate restock/retreat continuation and reserves before the final encounter.
[Proposal41](https://github.com/diegoami/better-borg-c/issues/41) tracks the owner
priority and diagnostic comparisons;22 tracks actual decisions. No policy or
claimed scroll rescue is implemented from this terminal dump. Original raw
record/build identity remains unchanged.

## Wider inspection requested by the owner

Default offsets now **T−5/12/25/40/60/82/105 engine turns**, omitting10 per the
owner's correction. Existing10/15/20 windows above remain historical evidence.
All added captures use this death's original337e50eb observed build/start/seed,
not the current corrected Borg, and match every prior observation/start save.

| Point | Actual turn | Actual HP | Depth | Native snapshot |
| --- | --- | --- | --- | --- |
|T−5|174857|21|12|[snapshot](snapshots/T-minus-5.txt)|
|T−12|174850|30|12|[snapshot](snapshots/T-minus-12.txt)|
|T−25|174837|51|12|[snapshot](snapshots/T-minus-25.txt)|
|T−40|174822|78|12|[snapshot](snapshots/T-minus-40.txt)|
|T−60|174802|84|12|[snapshot](snapshots/T-minus-60.txt)|
|T−82|174780|84|12|[snapshot](snapshots/T-minus-82.txt)|
|T−105|174757|84|12|[snapshot](snapshots/T-minus-105.txt)|

Separately labeled earlier event anchors:

- **Before last Phase Door, T−1072**: turn173790,HP39,depth2 — [snapshot](snapshots/T-minus-1072.txt).
- **Before no-upstairs fallback, T−1032**: turn173830,HP40,depth2 — [snapshot](snapshots/T-minus-1032.txt).

The actor was at depth2/max7 before this escape sequence. At173790 it carried
one Phase Door and one Deep Descent with two empty pack slots. Last Phase Door
read173794; repeated no-upstairs/downward fallback notes173774/173834;
**Deep Descent read173862 at34/84HP, depth2**, followed by Danger Level3.4;
restock173901. By174751 it is at depth12/max12 and84HP. Native Deep Descent
uses a delayed destination below maximum depth (here7+5=12), explaining this
transition. It is not evidence of simply walking down one level from11.
[Exact earlier original observations](earlier-events.log) retain the sequence.

The existing off-level escape routine explicitly prefers immediate Teleport
Level before delayed Deep Descent. Investigate its actual eligibility/available
resources and preparedness at the earlier danger, together with stock/unused
Speed and restock/retreat. The delay is known in source comments; this trace
alone is not an implementation-defect or rescue proof. [22](https://github.com/diegoami/better-borg-c/issues/22)
and [41](https://github.com/diegoami/better-borg-c/issues/41) have the finding and
action. No new tactic/threshold or baseline intervention is implemented.

All snapshots are native world-tick boundaries, not interpolated player turns.
A surviving diagnostic cap is not an independent life or a changed performance
cohort. Original raw build/evidence and death attempts remain immutable; full
new traces/saves/maps/results are in the wider-window recovery receipt on22/34.
