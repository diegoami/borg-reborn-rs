# D0003 — named headless reference of the gallant death

[Discuss this death](https://github.com/diegoami/better-borg-c/issues/34) · [Runner work](https://github.com/diegoami/better-borg-c/issues/111) · [Feedback process](../FEEDBACK.md)

A disposable Human Warrior on actual 29-carry source `127483bc4724` died to **a gallant at engine turn 20445**, level3/depth2/HP-3. The planned named-headless protocol validation used seed29, 30000 additional turns, wall120/stall30. It finished after20444 additional turns because the character died naturally. The final native save remains dead; no resurrection or favorable intervention was used.

This is a **diagnostic reference of D0002**, not another independent starting condition. All native Borg notes and all23 noncheckpoint state records (including sampled game/Borg RNG) match the recorded six-carry D0002 run. [Comparison receipts](comparison.json) pin both actual traces; names, the new PLAY metadata and periodic checkpoint ordering differ, so complete raw traces are not claimed byte-identical. The qualified assessment groups this reference into D0002's encounter family while retaining its actual source/binary/save identity.

## Evaluation and diagnosis

Native behavior and outcome reproduce the existing investigation. This does not demonstrate a new bug, a mortality benefit, or that unused healing would have saved the character. Existing heal/flee/escape/entry questions remain on issue34; see [D0002's assessment](../D0002-gallant/assessment.json). No gameplay strategy is changed.

The new runner successfully exported the native cause, recent messages, decision tail, map/inventory, immutable build/options/seed/input identity and assessment scaffold, then regenerated the registry. [record.json](record.json), [build.json](build.json), [events.log](events.log), [terminal.map.txt](terminal.map.txt) and [assessment.json](assessment.json) retain the actual run. Full logs, starting/recent/final saves and builds remain outside Git in the persistent validation archive described by the [runner verification document](../../measurement/NAMED-HEADLESS.md).

## Replay limits and actions

This build's default5/12/25/40/60/82/105 pre-death windows have not been qualified. Older D0002 captures are historical evidence on their original build, not relabeled as this version. Loading a living save resets Borg memory; faithful mid-run continuation is not claimed. Continue issue34's existing-rule investigation before proposing strategy changes, and route a verified reusable C defect through the canonical source tracker first.

Reproduction uses the committed runner: `./run-borg-headless.py HeadlessDeathProtocol --turns 30000 --seed 29 --timeout 120` from a new disposable save on this exact C source/observer/tool/calendar identity. A later checkout automatically builds its own source version; that is a different reference, not this historical run.

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
