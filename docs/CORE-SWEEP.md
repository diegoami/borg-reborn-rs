# Track P: owner-run calibration sweep prompt

This is a handoff, not a launched experiment or an improvement claim.
The review-fix pin must pass validation and owner milestone acceptance
before dispatch; see P269-VALIDATION.md.
The owner starts population experiments and agrees confirmation suites
(borg-zoo RULES 8 and this repository RULES 20). P1–P3 review PRs remain
unmerged for the owner's chosen reviewer. No sealed suite is opened here.

## Exact prompt for the zoo session

Work in **diegoami/borg-zoo**, following its AGENTS.md, RULES.md,
README.md, docs/DESIGN.md and docs/FINDINGS.md. Add a catalogue PR for the
experiment below; wait for the owner to start it. Do not edit another
repository or launch an Actions population run on this prompt alone.

Read **diegoami/borg-reborn-rs at
`2ec1f2507ec81f375a7967b2876597865458aa76`** (strict core/errand fixes; validation in P269-VALIDATION.md), especially
CALIBRATION.md, CORE-TRACE.md, ZOO-INTERFACE.md, P3.1-VALIDATION.md and the
example profiles. Its build source hash is
`1f5eb966caeb02bd8584f439961747e5fe5df89aed654bed46a7ea8b1c6ca0e9`.
Require its task CI to be green and its code to be present on main before
any owner dispatch. The compatibility contract was tested against zoo pin
`c5f1b0429ea898c8d5484aa4f1ac0d681972aac7`. A newer zoo adapter needs the
same contract check before using it. Freeze the actual zoo commit too.

### Question and arms

Explore whether earlier mana recovery improves early **Mage** completion
of 100,000 engine turns. This is a hypothesis. The existing diagnostic
threshold 10 lost one game; it is not a recommended value. The new sweep
studies default 6 versus 7 and 8, one parameter only:
`borg_param_recover_rest_sp_tenths`. All three are within the declared 0..10
range. The high-depth threshold and every other core parameter stay default.

Copy the three entries from `docs/examples/core-calibration-profiles.json`:
`core-calibration-default`, `core-recover-rest-7`, `core-recover-rest-8`.
They select `borg_reborn_core:1`, explicitly disable munchkin-by-class,
munchkin start, Lunal and self-Lunal, and select static valuation. The
pinned harness base sets `borg_reborn_errand=none`. Preserve these settings
and hash the actual file. Use no policy environment overrides: clear
BORG_CORE, BORG_ERRAND, BORG_PARAM_* and other inherited Borg controls.
The file must select the core and values itself. No C Borg control or
engine change is required for this within-specimen parameter experiment.

Register specimen `reborn-p-core-2ec1f250` at the full commit above,
`kind:borg-rs`, `impl:rust`, `exact_turns:true`, `depth_stream:true`.
The catalogue fragment and exploratory experiment are supplied beside
this prompt. Run the zoo's tests, `python3 -m zoo plan --experiment ...`
and its preflight before proposing a dispatch. Copy no old results into
this experiment. Record commit, compiler, binary, profile/config and
resolved-parameter hashes with each arm.

### Suites and selection, frozen before outcomes

Use two **new unsealed** SHA256-v1 rule suites, preserving the catalogue's
collision checks against every existing suite. Freeze both expanded
assignments and hashes before exploratory play; do not inspect any sealed
outcome or reuse/edit a frozen suite. Assigned birth is Mage only, cycling
all 11 pinned races in their catalogue order, with default game birth options.
Both suites use exact 100,000 engine turns, wall_seconds900, stall_seconds90,
no saves. The example fragment names:

- `core-mage-rest-dev-128-v1`: 128 shared seeds per arm, 384 exploratory games.
- `core-mage-rest-confirm-900-v1`: 900 fresh shared seeds for the eventual
  two-arm confirmation,1800 games, disjoint from exploration and all
  existing suites. Role `confirmation`, not `sealed`.

Primary outcome is **successful survival to the cap**: 1 only for an alive
run that reaches the exact turn cap; 0 for every other planned cell. Keep
actual death/stall/timeout/crash/configuration-error classifications
separate; a process failure is not relabelled a natural death. Every cell
counts. Publish counts of all outcomes, not just completed games.

Exploration has no confirmatory significance claim. Rank 7 and 8 by paired
mean outcome difference from control on all 128 seeds; ties choose 7 (the
smaller departure). If the winner's difference is not positive, report
no candidate and stop. Otherwise freeze that one candidate and a new
confirmation experiment manifest **before reading any confirmation result**.
It contains only control and the selected profile; retain the control_profile
field and the same specimen/cap/birth/modes. Do not retune from confirmation.

The confirmation is one prespecified comparison: exact two-sided McNemar,
alpha 0.05, on **all 900 planned pairs** of the binary primary outcome above.
Report candidate-minus-control effect and paired 95% interval, discordant
rescued/lost counts, and every process-outcome count. A positive effect
with p<0.05 supports this endpoint only. Depth, level, game turns, rest
counts, per-race splits, causes and trace changes are descriptive; no
post hoc subgroup or second endpoint becomes confirmatory.

Sizing targets a 10-percentage-point absolute difference. The worst-variance
normal approximation gives 785 pairs; 900 provides rounding margin. With
all 900 pairs discordant, a net gain 0.10 means a 0.55 candidate-win chance;
the exact test rejects at >=480 or <=420 wins and has 85.05% power for that
boundary model. `docs/examples/core-sweep-power.py` reproduces it. These
are design assumptions, not observed mortality or a guarantee for every
population. Do not resize or stop early based on interim outcomes.

A configuration/pipeline defect stops interpretation: retain every attempt,
report the defect and create a corrected experiment identity after a fix.
Do not drop failed cells, change seeds/caps/timeouts/assertions, or present
an incomplete/adapted confirmation as the prespecified result. The zoo's
existing complete-pair death table is descriptive here; the primary
analysis explicitly includes all planned pairs and process failures.

### Evidence and death handoff

Preserve the actual user/borg.txt, profile definition/hash, BUILD_ID,
binary/compiler/source identities, summary, stderr, params-core.json and
trace-core.jsonl (or a hashed archive plus a final200-row tail). The native
frontend writes the last two beside the summary without BORG_RUN_DIR.
Verify the selected core, changed value/origin and every unchanged default
from the parsed snapshot before accepting an arm's configuration. The
current adapter's config_checked field verifies only the written file;
it is not a substitute for that snapshot. Record its SHA256 separately.

The pinned adapter retains natural-death keys but deletes survivor keys
and state logs after hashing them. Preserve extra key/state evidence before
cleanup when diagnosing survivors; retain the native core trace for every
arm. Keep results append-only, never overwrite cells. Keep both arms in
one workflow run when possible; otherwise disclose workflow/compiler/date
and harness differences explicitly.

Every natural death needs its immutable dossier in borg-reborn-rs, with the
actual source/build/seed/profile/config identity, native key hash and the
last 200 named thinks plus resolved parameters. Preserve the source build's
BUILD_ID and native input files so the owning repository can import those
cases faithfully with tools/export-death.py and reconcile them. Route that
handoff to the owner; the zoo session does not write the Borg repository.
Use first divergent think/queued keys, HP/SP/danger and parameter values to
explain a sample of rescued/lost pairs, without inferring a nested tactical
reason from a composite root-check name.

Finish with the full result ledger, paired primary analysis, assumption
checks, causes/trace examples, artifact/death-dossier links, failures and
limitations. Update the zoo findings and link back to track P #218. A
negative/null result is a completed experiment; any tuning change belongs
in a separate Borg PR after the confirmation, preserving default controls.
