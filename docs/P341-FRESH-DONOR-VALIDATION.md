# Fresh M13 donor audit (#341)

The source/evidence adoption was completed by #331/#342. This task closes
its remaining independent fresh-donor build/run gap; it does not repeat the
source synchronization or change game behavior.

## Exact comparison and scope

The donor tag `4.2.6-rs-town-trip-m13-v1` was verified to peel to
`1443c2d1b97d110ffbf5ae420d6c6b97ac15f7ea`. Both executables were rebuilt
from separate clean worktrees, with actual clean BUILD_ID receipts retained:

| Role | Commit | Runtime source SHA-256 |
| --- | --- | --- |
| Fresh donor | `1443c2d1b97d110ffbf5ae420d6c6b97ac15f7ea` | `d07a8547b2c273793f163afde0a2faff1ed2e61545e3d33e1ce0dcc17c8ecf4c` |
| Accepted combined-P M13 adoption | `2f74e1c5d57ce2b41ded1b5b694174b91b51f265` | `87717140e88805c99b2b8f5d036036cf21eb9b8e953b8cce25d07465a2c2a9a9` |

The receiving full suite uses BORG_CORE=borg-rs,
BORG_CORE_OVERRIDES=off, BORG_ERRAND=none and BORG_STOPSCARED=off.
The donor runs the same explicit menu-letter full suite file: all nine
classes, five starts per class, 100,000 game turns. No cap, seed or check
is reduced. Each implementation's effective borg.txt bytes are compared, alongside
keys.log, messages.log and state.log. Whole-summary comparison remains
strict and its output is retained; existing cross-repository telemetry and
schema differences are explained separately from actual play equality.

## Later safety-policy source is separate

Current track-p also contains owner-accepted #355. Its runtime source is
`0f2e003f988fba56929f259f6098d356ec12c0ec024419750937a13a7dce5d58`.
It intentionally changes native caution, escape and flee routing and removes
stair looting. Relative to the M13 adoption snapshot, the runtime delta is
six borg-core policy files; the two remaining engine/harness paths are its
native incident test and pinned fixture manifest. No other game inputs differ.

This report validates the accepted M13 **adoption epoch** against a freshly
built donor. It does not claim that post-#355 production policy is identical
to the unmodified donor, or attribute a performance/survival effect to it.
The later source's scenario controls retain their own actual receipts in
INCIDENT-350-VALIDATION.json. #349's checked sync preserved that accepted main
runtime byte for byte. This task leaves it unchanged rather than undoing the
safety fix to regain donor parity.

## Evidence retention and limits

P341-FRESH-DONOR-VALIDATION.json records actual source/build/binary identities,
effective configs, per-start summaries/log hashes and trace checks. The dossier
map qualifies fresh native packets through tools/import-deaths.py; original
record/payload bytes and IDs are retained by its receipts. Existing legacy and
qualified evidence is not rewritten. Repeated results and paired controls do
not become additional independent lives. Canonical cemetery contents/pins and
Zoo specimens/runs are unchanged.

Raw builds and attempts remain under /workspace/work/p341-adoption,
/workspace/work/p341-donor and /workspace/work/p341-validation. Archive and CI
results are recorded after the completed task checks. No urgent-purchase branch,
large archived replay corpus, sealed suite or population outcome is claimed.

## Completed results

- Fresh donor and combined-P adoption: 45/45 keys/messages/states and
  effective config bytes match. All receiving calibration parameters have
  default values/origins. The trace has 242,641 consecutive classified rows,
  with zero unclassified rows.
- Whole-summary comparison reports 0/45 complete records identical because
  of effective_settings, borg_decisions, borg_rng, wall_seconds and the
  receiver-only exact_turns/loaded/game_options fields. The checker is
  unchanged and its failed output/hash is retained; this is not called a
  strict whole-record parity pass.
- Three policy-on class pairs match play logs/configs. All 656 schema11
  plans replay, including 500 with a plan; no urgent-purchase coverage.
- The 13 profile/isolation tests, four shopping tests and three loop-errand
  tests pass. Original-core smoke matches all three references. Fresh
  adoption core tests pass328 with four existing ignored.
- All 25 natural-death packages (12 per full implementation plus one
  original-core smoke death) are qualified and mapped. All 2,578 archive
  packages verify. Package counts are not independent population counts.
- Initial errand fixture tests had four/three harness errors because their
  environment isolation hid the cemetery tooling checkout. Adding an ignored
  pointer to the exact pinned checkout fixes discovery; the same assertions
  pass unchanged. A separate retained error probe ends in borg_stopped and
  harness status1, not a death. Initial test logs and their temporary-directory
  cleanup limit are explicitly recorded rather than dropping those failures.

This is an evidence-only task. Current production runtime, donor/recorder/
cemetery pins, plan behavior, policies and configuration defaults are unchanged.
Required CI and focused archive/tool checks accompany the task PR into track-p.
Promotion remains subject to its separate pinned milestone review.
