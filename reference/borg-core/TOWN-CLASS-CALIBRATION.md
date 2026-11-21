# Class profiles and the calibration sequence

**Active combined study:** the owner has already dispatched the milestone-12
trip + live inventory experiments. See [exact pins and evidence boundaries](ACTIVE-JOINT-EXPERIMENT.md).
The standalone local results and unrun Knowledge/Home grid below remain separate.

Zoo integration request: [borg-zoo#88](https://github.com/diegoami/borg-zoo/issues/88).

The town utility evaluator is shared by every class. Maximum HP, level,
spells, supplies and intended journey already change its decisions. Optional
class-specific **parameters** are resolved before birth by the runner and
written as ordinary Borg settings. There are no per-class stock quotas or
new class branches inside the evaluator.

Status: **configuration mechanism verified, class tuning pending live tests**.
The shared `reference` profile remains identical for all nine classes;
`class_defaults` is empty. The validation probe's values demonstrate routing,
not accepted class recommendations. The class-calibration runner remains shadow-only; the separately requested
[live executor](TOWN-LIVE.md) is now available for the local outcome comparison.

## Resolution and the zoo format

The portable [profile file](../tools/town-plan/zoo-profiles.json), schema 2,
contains shared `base`, optional `class_defaults`, and named experiment `arms`.
Each arm has `settings` and optional `class_overrides`.

The override order is explicit:

1. Shared `base`.
2. `class_defaults[actual_class]` supplies a class baseline.
3. The arm's `settings` applies the experiment's changes.
4. The arm's `class_overrides[actual_class]` is most specific.

Each run retains the resolved values, their source layer, selected profile
name, settings hash and the complete source profile/experiment manifest.
Unknown class names, misspelled settings, duplicate JSON keys, wrong types and
out-of-range rates fail instead of silently selecting defaults. Class overrides
can change active town utility parameters; mode switches and live execution
cannot be enabled through this shadow-calibration path.

Normal progression profiles require automatic destination and duration:
`target_depth = 0`, `trip_turns = 0`. Fixed-journey fixtures such as the depth-2
test need explicit `calibration_only: true`. Increasing `depth_gain` still
advances the destination with progress; it does not pin it at a dungeon level.

The zoo already implements class selection at
[`catalogue.resolve_profile`](https://github.com/diegoami/borg-zoo/blob/7ae123b8240e92428470e9c2b6145eebc9ef614b/zoo/catalogue.py).
The [generated native catalogue](../tools/town-plan/zoo-class-profiles.json)
uses that format: complete ordinary `settings` profiles plus a selector:

```json
{
  "by_class": {
    "Mage": "rs-town-class-selection-probe-mage-normal",
    "Ranger": "rs-town-class-selection-probe-ranger-normal",
    "Blackguard": "rs-town-class-selection-probe-blackguard-normal"
  },
  "otherwise": "rs-town-class-selection-probe-normal"
}
```

That selector is a **validation probe**: Mage cash priority 50, Ranger future
Home discount 750, Blackguard risk 1500; other classes use the shared vector.
The production reference selector has an empty `by_class` and a shared fallback.
Fixed/assigned birth determines the class before the zoo resolves the profile;
the zoo rejects class selectors with random birth. All nine class indices are
checked against game data; `menu` birth selects Ranger 7 and Blackguard 8 correctly.

## Use and reproduce

```sh
# Print the nine matched default/probe commands; no games start:
python3 tools/town-plan/calibrate.py --arm reference --arm class-selection-probe

# Export profiles in the existing zoo catalogue format:
python3 tools/town-plan/calibrate.py --arm reference --arm class-selection-probe \
  --export-zoo /tmp/town-class-profiles.json

# Explicit classes and paired seeds; --run is required to execute:
flock /tmp/angband-validation.lock python3 tools/town-plan/calibrate.py \
  --arm reference --arm class-selection-probe --classes Mage Ranger Blackguard \
  --seeds 2901 --turns 1000 --out harness/out/rust/MY-UNIQUE-CLASS-CHECK --run

# Report every class/attempt, including failures or missing cells:
python3 tools/town-plan/report.py --root harness/out/rust/MY-UNIQUE-CLASS-CHECK
python3 -m unittest discover -s tools/town-plan -p 'test_*.py'
```

The default matrix is one seed per class, 501–509, 30000 game turns. With
`--seeds`, the same explicit seed list is crossed with every selected class
and arm. `--wall-seconds` defaults to 120 per cell. A full manifest is written
before the first game. Existing output roots and zoo export files are preserved;
use a fresh path. Every attempted failure remains visible. Schema-1 portable
profiles remain readable; historical fixed-journey examples are marked as such.
The historical seven-class report remains available for its frozen data; new
manifest-based reports include Ranger/Blackguard and group results per class.

## What has passed

[Validation receipt](validation/2026-11-14-town-class-profiles/README.md):
14 Python tests; 36 native zoo cells through the pinned, unmodified adapter;
18 further runner/report cells; all nine classes and shared fallbacks checked;
1952 new v5 records replayed. Both game-parsed settings and recorded vectors
match the selected profiles. Matched shadow arms retain identical gameplay.

This validates profile selection and observation, not class-specific fitness.
No vector was chosen because it saved more gold, scored more shadow utility,
or happened to survive one of these short runs.

## Calibration and acceptance after live execution is ready

1. **Shared baseline first.** Compare current, no-shops, utility reference and
   utility with cash priority 50 on the same starts. All utility arms keep
   automatic ambition/duration. Report each class separately. Covering all
   nine classes with five seeds per class gives 180 executions for four arms;
   this is an exploratory pilot, not a sized improvement claim.
2. **Explain class failures.** Examine provisioning, purchases, storage,
   descent/town loops and death evidence. Change a small number of priorities
   supported by those observations: cash, risk, Home or a need weight. Use
   one-factor comparisons before interactions. The existing reserve410 Rogue
   hypothesis ([zoo#79](https://github.com/diegoami/borg-zoo/issues/79)) does not
   establish parameters for this different utility model.
3. **Freeze candidate overrides.** Write the complete shared vector and class
   overrides, source/specimen hashes, modes, seeds and limits before the next
   batch. Record each selected native profile and actual parsed settings.
4. **Fresh-seed confirmation.** Use disjoint starts with the same per-class
   endpoints. Proposed primary endpoint: alive at the common cap after
   reaching DL5. Secondary: deaths, time to depth 5/10/20, maximum depth/level,
   town time and operational failures. Gold and snapshot utility are diagnostics,
   not acceptance criteria. Agree sample size/confirmation rules with the owner
   before making improvement claims; five pairs per class are only a pilot.

The prerequisites are the separately reviewed live executor and owner-started
outcome experiments. Shadow configurations deliberately leave gameplay unchanged
and cannot select survival winners. This change neither enables the executor
nor dispatches a zoo workflow. A zoo request can now cover catalogue import,
per-cell shadow-record capture and parsed-parameter verification, with the live
comparison explicitly gated on borg-rs#36 / PR #42's later execution work.

### Combined capability branch (v7)

Current shared defaults include the six capability parameters documented in
[TOWN-CAPABILITIES.md](TOWN-CAPABILITIES.md). Historical mode keys are absent
because this runtime retired those modes. Live exports explicitly enable both
`borg_town_plan` and `borg_trip_policy`; shadow exports keep both off. Class
selection still uses shared defaults with optional explicit overrides, not
separate hardcoded purchase lists. These exports are candidate configurations,
not changes to the already frozen zoo studies.
