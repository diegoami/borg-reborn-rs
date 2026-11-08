# D0002 — gallant death, seed29

[Discuss this death](https://github.com/diegoami/better-borg-c/issues/34) · [Feedback process](../FEEDBACK.md)

A level3 ordinary Human Warrior used its remaining Phase Door scrolls while
fleeing a dangerous level, then fought scouts and a gallant. It continued melee
at3HP and died to **a gallant at engine turn20445, depth2, HP−3**. Maximum depth3.
Twelve known Cure Light Wounds potions remained in its pack; no CCW was carried.
This records an actual death and questions for investigation, not a proved bug
or evidence that drinking a potion at the final action would save it.

## Exact version/build that died

Original recorded attempt here: **45eac63bd9f7cabe3c4ef1607703324ca8dae0fc**, six
C carries including source77's potion timing correction. Source/patch pins,
actual binary/tool/data/config/RNG/input identity, commands and runtime calendar
are in [record.json](record.json) and [build.json](build.json). Observed binary
SHA256: `aabad5875e34c232044d9989eaaacd9d922967027abd84c874602d61e0083b17` (GCC15.2.0, Linux x86_64, Debug, X11/test frontend).
The fork-snapshot observer is declared; no gameplay actions/RNG draws added.
Native clock unchanged. Ordinary Human Warrior/default birth/seed29, no cheat
death, altered damage, hidden information input or human intervention.

Five-carry **3638b690525cd8a381cb106b4c5ee40652babde2** and pristine
**e473720ed88a0f0c7ca66815cef3000eb4a38bc3** are explicitly separate references,
with their own [reference-1.build.json](reference-1.build.json) and
[reference-2.build.json](reference-2.build.json). No older build is relabelled.

## Recording and replay qualification

[Selected exact observations/messages](events.log) and the actual
[terminal map/inventory](terminal.map.txt) retain the final200 engine turns and
native recent-message buffer. Buffer messages have no independent timestamps;
duplicated channels are not extra attacks. This is a text recording, not video
or a complete every-turn native event stream. Internal diagnostic map contents
are read after the run and never fed into Borg decisions.

The unchanged six-carry repeat has byte-identical full trace/start save, final
character and both RNG states. Five-carry and pristine have byte-identical full
traces to each other; six-carry differs in internal RNG and observations, even
though death/cause/turn/final gameplay and game RNG state agree. Do not describe
all versions as identical encounters or traces. One distinct starting family;
four death process attempts (original, repeat, two version references), not four
independent lives or a survival-rate estimate.

| Point | Actual turn | Actual HP | Native snapshot |
| --- | --- | --- | --- |
| T−20 |20425|31|[snapshot](snapshots/T-minus-20.txt)|
| T−15 |20430|22|[snapshot](snapshots/T-minus-15.txt)|
| T−10 |20435|12|[snapshot](snapshots/T-minus-10.txt)|
| T−5 |20440|3|[snapshot](snapshots/T-minus-5.txt)|
| Death |20445|−3|[terminal](terminal.map.txt)|

All four original-start window replays have identical prior observations/start
save. Stops occur immediately after world-turn increment, before the remaining
processing at that tick; these are actual HP observations, not interpolation
from earlier Borg notes. Window caps are diagnostic stops, not extra survivors.
Native live map headers use legacy death labels; actual alive/cap state is in
record.json. World turns, energy, Borg-think calls and wall seconds are distinct.

## Evaluation and diagnosis

The last Phase Door was read at20323, after several earlier attempts. At20333
an unknown scroll was read (Treasure Detection), at20343 an unknown potion was
quaffed (Boldness). Neither is assumed to be a known healing/escape alternative.
By20422 the Borg logged31/49HP and danger513; by20436 it logged3HP/danger551.
At20443 it attacked the22HP gallant with a dagger; native hits immediately
precede death. Heuristic attack score97 is not an HP-damage prediction, and
heuristic danger507 is not an asserted single enemy blow.

The native CLW healing estimate at3/49HP is15. Existing borg_heal guards require
`danger < hp + clw_heal` and `clw_heal > danger/3`; with the recorded inputs both
are false (507<18;15>169). Thus unused potions alone do not show an eligible
missed heal. The outer caution path also skips heal/defend when fleeing; the
actual runtime branch still needs observation before attributing rejection to
that guard. A potion costs an action and could fail to keep pace with damage.
No threshold, tactic, counterfactual rescue or mortality benefit is implemented.

This behavior already exists in pristine C. It is not established as a new
regression from our carries, nor as a demonstrated original-C defect. The
separate timing carry changes the shared CCW helper, not these CLW thresholds.

## Actions and reproduction

[Investigation34](https://github.com/diegoami/better-borg-c/issues/34) predates
changes: observe actual existing heal/caution eligibility and fleeing state;
trace earlier Phase Door exhaustion and legally available escape/resources.
Related22 concerns another death/unused Speed; a shared bug is not assumed.
Actions/confidence are machine-readable in [assessment.json](assessment.json).
If an original-C defect is proved, coordinate source207 and file canonical
neo-angband-tracker evidence before source fix and separate consumer adoption.
Bugs always pre-empt discretionary exploration.

```sh
flock /tmp/angband-validation.lock python3 tools/measure-native.py prepare --ref 45eac63bd9f7cabe3c4ef1607703324ca8dae0fc --observe --output artifacts/D0002-build
flock /tmp/angband-validation.lock python3 tools/measure-native.py run --build artifacts/D0002-build --output artifacts/D0002-death --seed 29 --cap 400000 --timeout 180 --stall-timeout 30
```

Repeat into a new directory; window caps20425/20430/20435/20440 otherwise keep
conditions unchanged. Match recorded data/toolchain/calendar; native seasonal
calendar rules prevent an unqualified every-date replay claim. Binary hashes
identify actual builds, not guaranteed cross-toolchain identical binaries.
Full runs/traces/maps/saves are persistent ignored artifacts/deep-impact-v2,
with [batch33 recovery](https://github.com/diegoami/better-borg-c/issues/33).
Native saves omit Borg memory and compact monsters; no exact midrun continuation
or reloadable postdeath dungeon is claimed. Recent living save load qualification
is a separate diagnostic, not part of the original trajectory.

The actual recent living save at20073/49HP was loaded and advanced by one
ordinary wait to20074 without activating Borg; [diagnostic receipt](save-load.json).
This validates readability/advancement, not exact Borg-memory continuation.

## Owner feedback: too many enemies too early

[Owner comment on34](https://github.com/diegoami/better-borg-c/issues/34#issuecomment-6685824152)
asks whether this level3 character entered a crowd too early. The investigation
will identify the first known group, entry method and available retreat before
proposing exposure policy. The trace records Phase Door20293 followed by the
critical crowd at20303, then further Phase Door through20323; a deliberate walk
into that crowd is not established. Both earlier preparedness and escape/landing
risk deserve investigation using only information available to the Borg then.
This is a tracked owner observation/action, not yet a code defect, a new crowd
threshold or a demonstrated alternative rescue. Findings will be replied to on34
and reflected in assessment.json/aggregate. Original evidence/version unchanged.

## Wider inspection requested by the owner

Current default profile is **T−5/12/25/40/60/82/105 engine turns**, omitting10
per the owner's correction. The original10/15/20 snapshots above remain legacy
evidence, not part of this new profile. All added original-start replays match
the original six-carry45eac63bd starting save and every prior observation.
No current Borg fix is substituted into this old death.

| Point | Actual turn | Actual HP | Native snapshot |
| --- | --- | --- | --- |
|T−5|20440|3|[snapshot](snapshots/T-minus-5.txt)|
|T−12|20433|18|[snapshot](snapshots/T-minus-12.txt)|
|T−25|20420|31|[snapshot](snapshots/T-minus-25.txt)|
|T−40|20405|48|[snapshot](snapshots/T-minus-40.txt)|
|T−60|20385|48|[snapshot](snapshots/T-minus-60.txt)|
|T−82|20363|48|[snapshot](snapshots/T-minus-82.txt)|
|T−105|20340|47|[snapshot](snapshots/T-minus-105.txt)|

Separately labeled **entry anchor T−160**, turn20285/46HP, [snapshot](snapshots/T-minus-160.txt), precedes Phase Door20293 and the critical crowd observation20303. The105-turn default alone would miss that entry. Earlier knowledge/preparedness still needs investigation; no deliberate walking entry or causal rescue is inferred.

The actor is still48/49HP at T−40 and31HP at T−25, then collapses to3HP by
T−5. These are native tick-boundary observations, not interpolated command
states or eight independent lives. Windows stop immediately after world-turn
increment; healing/damage between nearby offsets may not align with Borg note
boundaries. No post-death reload/Borg-memory continuation assumed. Full raw
runs/saves/maps/metadata are preserved on the expanded-window receipt22/34.

## Owner Phase Door feedback and improvement candidates

[20293/20294/20303/20430 original C34 anchors and analysis](../D0004-named-headless/owner-phase-feedback.md)
record the disputed first jump, room retreat proposal and danger188→451. These
are qualified family-reference observations; this dossier keeps its actual build.
Fight/door rescue and wake order remain unproved. Enhancement154 links source370
separately from bug carries; assessment actions remain open.

## Brainstorming and overall picture

[Shared gallant-family assessment](../D0004-named-headless/overall-assessment.md)
connects stock, threat valuation, entry/escape and recovery, with competing
explanations and focused candidates158/159. Attempted walking escape is examined
both after landing and after scroll exhaustion; safe success and sleeping-enemy
timing remain unproved. This dossier retains its actual source and evidence.
