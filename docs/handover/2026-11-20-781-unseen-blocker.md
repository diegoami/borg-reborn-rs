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
Normal fuel reserves apply. All pinned engine brand, slay and shape melee
verbs are covered by a source-data test; nonliving oil victims clear on
"It is destroyed." too. The new Phase Door site uses bump evidence in
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
[Cemetery case](https://github.com/diegoami/borg-cemetery/tree/adf585125e1d1b1b525d1e5424797c4e938e990f/validation/borg-reborn-rs/unseen-bump-781).

**Owner:** review the task PR. Any enabled profile/default choice and zoo
measurement remain the owner's decision; no new implementation follow-up was
identified. Other sessions' tasks were left alone. The task merge still needs
the owner's review disposition (RULES 1a).


## Review follow-up

Synchronized current main fcaecf2b without rewriting either branch. The
response now follows ordinary healing/defence/Recall/food and precedes stair
walking. Detection shares existing danger/darkness gates. The independent
cure intentionally ignores danger; its risk is explicit. Bump phases retain
emergency landing checks and increment ordinary level escape accounting
(the escape limit starts leaving; it is not a hard cap). Twitch steps now
supply the same bounded bump evidence.

The F1 report's delayed-think ordering is not the ordinary queued-input
ordering: messages are parsed and queued keys/directions answered before a
new think. The cited second throw follows a miss, then kills; retained logs
show no third throw after that kill. Nevertheless pending-shot completion
is now explicit, preserving an unresolved shot across intermediate updates
and clearing it on an oil-break resolution, kill, cancel, expiry or entry.
New tests cover delayed death feedback, a miss followed by an unrelated
death, and all cleanup boundaries. Queued attempts reserve the budget.

The original cure calculation is checked directly: of110 final caution
rows,70 meet the30% wound gate. The first is turn37069, HP29/44, danger76;
all70 fail BOTH `danger < HP +20` and `20 > danger/3`. Thus the historical
explanation remains valid; it does not attribute the loss to zero danger.

The registry count is2269 (main2265 plus four); historical snapshot counts
remain unchanged. Cemetery links are pinned and the Phase Door table includes
the new site. Refreshed validation/evidence and the exact new review pin are
recorded on PR #782. Both PRs still await the owner's review disposition.

Latest integration: synchronized main d7e58cdf after #787/#785 landed during
validation. Earlier review runs remain pinned to fcaecf2b; latest shipped
smoke uses the new main reference. No shipped values changed in this task.
