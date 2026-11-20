## 2026-11-20: an unseen monster blocking the stair step (#781)

**Done:** off-by-default bump occupancy, directional oil/detection/Phase Door
response, and emergency cures before stair desperation, including while fleeing.
`caution_unseen_bump_turns`, `caution_unseen_bump_response` and
`caution_unseen_bump_heal_hp_percent` are compiled and shipped 0;
`caution_unseen_bump_throw_tries` defaults to 3. The windows/budgets are
hypotheses for measurement, not calibrated values. See CALIBRATION.md.

Only an unimpaired movement command, unseen melee feedback and an unchanged
observed position establish a grid. Incoming attacks alone do not. All flow
seeds/spreads and old-flow steps refuse remembered occupied grids. Memory
expires, resets on a new level, clears when entered, and clears when a
single-target directional oil throw reports the unseen victim's death.
Normal fuel reserves apply. The new Phase Door site uses bump evidence in
place of the known-race fight test and logs that choice.

**Found:** the historical control at `09a70906` reproduces the Druid death at
37,759 with key hash `8b21c6b94e80e969`. Disabled changes reproduce it too.
CLW never qualifies on any of the 110 final caution decisions: its wound
threshold and danger-versus-healing tests do not overlap. Danger grows
24→170; it is not zero. Recovery's unseen-attacker switches were off, and
recovery is downstream of the stair walk.

Current main (`4e9af99e`, after #768) already survives this seed. Its disabled
patched stream matches the original current-main control. Historical enabled
responses survive to 100,000 (CL7/DL2); a zero-oil-budget check exercises Phase
Door. A separate 100%-HP cure-threshold probe exercises the new cure priority;
ordinary 50%-HP checks did not need that cure. Detect Invisible's new site was
not exercised by these seed runs. A preliminary response wasted throws after
an unseen kill; its retained trial led to the kill-confirmation fix.

**Checked:** workspace fmt/clippy/tests, release borg-core tests, parameter
citations, script/tool tests, vendor check, and default-core smoke (3/3 exact
keys/messages/states). Reborn-core population/full suites were not run; the
change is confined to borg-rs's core and its settings. No population experiment
was dispatched and no improvement claim is made.

Canonical gameplay evidence and reproducible historical patch:
[Cemetery case](https://github.com/diegoami/borg-cemetery/tree/evidence/reborn-781-unseen-bump/validation/borg-reborn-rs/unseen-bump-781).

**Owner:** review the task PR. Any enabled profile/default choice and zoo
measurement remain the owner's decision; no new implementation follow-up was
identified. Other sessions' tasks were left alone. The task merge still needs
the owner's review disposition (RULES 1a).
