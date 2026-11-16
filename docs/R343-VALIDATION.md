# M13 carry into Track R (#343)

Local acceptance passes. PR #357 integrates only into feature/track-r after
final CI. Main and global reviewed receive no redesign promotion. R3 still
needs accessor #363, attack #364, defence #365 and combined review #366.

The core is exact M13 tag `4.2.6-rs-town-trip-m13-v1`, commit
`1443c2d1b97d110ffbf5ae420d6c6b97ac15f7ea`, from accepted main
`2f74e1c5d57ce2b41ded1b5b694174b91b51f265`. The receiver runtime input hash is
`a698f1f6810032239b721af1fc51b48e818e53efacd345450ba29dbff2af1511`.
Its code was committed at b3eff0b0 before measurement: inputs_dirty=no;
git_dirty=yes reflects reports outside the runtime inputs. The fresh clean
donor input hash is d07a8547b2c273793f163afde0a2faff1ed2e61545e3d33e1ce0dcc17c8ecf4c.
Actual build/binary identities are retained in R343-COMPARISON.json and
validation/r343/*-BUILD_ID. Historical v12/R1/R2 measurements are unchanged.

## Policy and integration

The independently owned danger, caution, escape and recovery bodies carry
the M13 changes. Named view fields add BI_PSTUN, BI_VACID, BI_VELEC,
BI_VFIRE and BI_VCOLD without boolean coercion. The native physical-cold
temporary-resistance asymmetry is preserved. Every trait group has a
native-value mapping check. Healing uses the accepted shared parameter
registry's 0 percent plus 20/40/60 HP defaults, with M13 source provenance;
five retired parameter names remain rejected. No threshold is invented.
The eight changed already-ported donor functions are listed in
R343-SOURCE-CHANGES.json. The port's retreat danger temporary is scoped to
its original calculation point to avoid an unused initializer; branch and
effect order are unchanged.

New town/trip state gains Clone/PartialEq only for owned values. Tests
exercise deep copies, final-state divergence and nested effect-once replay.
No engine snapshot/token authority is introduced. The 68 shared dependency
interfaces, strict source gate, invalid-grid diagnostics and existing native
ABI remain intact: 46 options, store_buy_direct=false, guarded town-query
patches. Accepted P naming, strict importer, recorder/cemetery pins and
errands-plan work are preserved. Town/trip stay off by default. No moving
reference pin or Zoo population pin changes.

Attack, defence, item/spell actions, flow, preparation and town/trip policy
remain shared native services. Equality here validates the existing R2
ownership boundary; it is not independent validation of those future ports.

## Exact games and coverage

- Fresh donor full: 45 games. Receiving off, check and switch each match all
  45 donor keys.log/messages.log/state.log streams byte for byte, with equal
  effective borg.txt hashes and common gameplay fields.
- Check and switch each have zero mismatches across 261,953,840 comparisons,
  including 445,762 stateful transactions. Every one of the 11 public
  stateful entries has positive coverage in each mode. Rogue 1063 still
  provides the single rare caution_teleport entry; the strict flag, seed
  and 100,000-turn cap are unchanged.
- Per mode, legacy shadow-stats@1 records 23,440,173 tape events. Those split
  into 949,047 policy markers and 22,491,126 actual dependency requests.
  The raw @1 field is not relabelled; derived counts are explicit in the
  report. Entry counts prove entry, not every branch or successful effect.
- Native smoke off/check/switch is 3/3 identical, with positive subsystem
  coverage and zero mismatches. Original-core smoke also matches its
  accepted reference 3/3. Full engine checks cover inactive/active errands
  and plan parsing; all 13 pinned Zoo profile-contract checks pass.
- The strict cross-repository harness/diff still reports 0/45 complete
  records identical. Donor/receiver telemetry and summary schemas differ;
  that failed strict result is retained. No checker is weakened. The
  independent report checks all three actual streams, effective inputs,
  identities and common gameplay fields separately. Same-receiver
  off/check and off/switch full diffs pass 45/45.

## Changed branches and controls

A declared test-m13-danger build uses constructed owned memories and loaded
engine race/blow tables. All 140 actual native/owned estimator pairs match
across elemental vulnerability/temporary-resistance and sound/stun-protection
combinations. Clearing the five projection inputs on a private port state
produces 41 detected faults, covering each new field. These are synthetic
branch checks, not observed encounters or population evidence.

A separate build restores the literal v12 danger body. Both modes reject
its acid-breath fixture: M13 native 440 versus stale port 340, with durable
stderr/process receipts and harness exit 1 after assertion abort. Literal
re-porting restores all 140 matches, all 41 input-fault detections and the
original fixture's keys/messages/states. No successful game or natural death
is fabricated for the failed control. Patches and identities are in
R343-CONTROLS.json, R343-STALE-DANGER-FAULT.patch and R343-DANGER-RE-PORT.patch.

A real engine callback-nesting build has four nested entries in each of
off/check/switch, identical play streams and zero shadow mismatches. It is
kept separate from normal games. Existing six invalid-grid subprocess tests
and opaque nested/effect-once/freshness unit checks pass.

## Opt-in town/trip and evidence

Warrior, Mage and Rogue, race 0 / seed 1001 / 20,000 turns, each match the
exact donor in all three receiver modes: nine receiving contexts, three
donor contexts. All 984 receiving schema11 plan records replay, 750 with a
plan and none taking an urgent purchase first. Repeated modes are not
independent trials; no urgent-purchase coverage is claimed. The donor harness
supplies identical declared NAME=VALUE opt-in configs to both binaries;
normal full/smoke/errand checks use the receiving harness. The full historical
external replay corpus is not rerun. See R343-TOWN-TRIP.json.

Every measured natural death is exported with its actual source/build/config.
Full donor has 12 deaths; the three receiving full modes have 36 run deaths
representing 12 repeated encounter/result identities. Original smoke/death
regressions contribute two further identities. All are reconciled. The
26 qualified source packages are mapped in R343-DOSSIER-MAP.json; repeated
build receipts/package counts do not establish independent trials.
All 2,669 retained packages verify, including the unchanged 2,455-case frozen
legacy inventory. Final numeric allocation remains central at borg-cemetery
acceptance (#344); provisional IDs never overwrite legacy slots.

Local checks: 175 workspace tests; 335 core tests, four upstream ignored;
92 tool tests; 33 source-tool tests; Clippy (including the test feature),
formatting, vendor and archive/ID checks. Engine discovery passes 68 checks
with one optional skip; the separate required pinned-profile run passes 13.
Initial operational clone/build attempts and failed strict/schema/control
results are retained under /workspace/work/r343-*; raw campaigns and plan
corpus are under /workspace/work/r343-evidence with hashes in the reports.

No survival, performance, full redesign, sealed validation or #293 shopping
fix claim is made.
