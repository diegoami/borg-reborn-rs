# D0005 — verified visible reference of the gallant death

[Discuss this death](https://github.com/diegoami/better-borg-c/issues/34) · [Recording/replay work](https://github.com/diegoami/better-borg-c/issues/134) · [Feedback](../FEEDBACK.md)

An ordinary seed29 Human Warrior on actual34-carry source `1b95154f65b2` died naturally to **a gallant at engine turn20445**, CL3/depth2/HP-3. This bounded diagnostic exercised the new visible recorder and local verifier. [record.json](record.json) and [build.json](build.json) retain the actual source/binary/data/config/calendar identity; no old death is relabeled with this build.

## Reproduction and replay qualification

The original native process and verified rerun match every recorded terminal poll/event, Borg action, native clock/file input, logical terminal frame, serialized-world fingerprint and sampled game/Borg RNG state. Starting, recent living and final native saves also match byte-for-byte. [verification.json](verification.json) is the retained certificate; full standalone bundles remain in the primary checkout's ignored artifacts/recording-replay-validation/deaths/D0005-visible-recording/. The original source execution and the replay are separate recorded process attempts, not independent lives.

All Borg notes and all23 noncheckpoint state records match D0002's six-carry reference ([comparison](comparison.json)). The assessment therefore groups this case into D0002's existing encounter family. Names, presentation/controller profile and periodic checkpoint observations differ; no new population survival result or original-C defect is inferred.

## Local inspection

```sh
./replay-borg.py docs/deaths/D0005-visible-recording --from-turn 20340
```

The verifier reruns from the original recorded start, then opens ORIGINAL recorded observations at the requested absolute engine turn. Arrows step, g seeks, Space plays/pauses, i shows inventory and m shows monsters; v returns to the screen. Diagnostic actual/known item and unseen-monster data are labeled and were never fed into Borg inputs. The viewer is logical terminal cells, not pixel video.

The seven requested pre-death offsets5/12/25/40/60/82/105 are retained under snapshots/recorded-T-minus-*.txt/.json. These are losslessly decoded original observations, with their actual phase and turn in record.json. Tick and command phases are distinct; repeated cells are not invented decisions. Full-run verification certifies these recorded observations without loading a mid-run checkpoint.

## Evaluation and actions

Existing issue34's healing/fleeing/early-exposure questions remain open. This validates recording and replay, not an eligible missed heal or a counterfactual rescue. See [assessment.json](assessment.json), [events.log](events.log) and the native [terminal map](terminal.map.txt). No gameplay strategy changed. Loading a saved character remains a NEW cold-Borg session; verified replay re-executes its original start and does not assert savefiles serialize complete Borg memory.

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
