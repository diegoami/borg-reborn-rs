# Restore combat reactions after knockout — consumer7

Canonical report: [source14 / CBUG-0002](https://github.com/diegoami/neo-angband-tracker/issues/14).
Discovery credit borg-rs; native proposal/evidence Diego Amicabile. The source
lane/worktree remains independent. Official6808 was read-only checked OPEN,
unmerged, at6f463d01d753089ab695537495ab27f3bc928dfa.

Original source32ab6823c6a993d94ce382b72f8a855a20f7911c and its comment cleanup
6f463d01 were imported with cherry-pick -x as8f3202109 and0009ecddc. The final
source changes one assignment: clear the global borg_dont_react read by the
reaction handler, rather than the unused struct member. The existing reset
boundary after borg_parse(NULL) stays in place. No RNG call or game rule changes.
Final product borg.c SHA256:
`fccea5be3bb31c1e3a8e2aecd4a209fc2877abc2845f64cf98cf343550a1e972`,
identical to reviewed6f463d01. Patch/provenance/removal unit is separate from the
safer swap in patches/series.json.

## Verification

The unchanged source fixture is pinned to
`3de1e3123915587483395dd40c4d2e75704e65f4`,
`docs/audit/c-reports/CBUG-0002/scenario.c`.
Only absolute disposable profile/output strings are relocated. It is linked to
actual product core objects, with the same product test helpers/data before
and after. The before core already carries source82; this comparison isolates
the additional reset. Seeds, assertions and controls are unchanged.

| Native scenario | Before accepted kill reactions | After | Control log |
| --- | --- | --- | --- |
| Knockout, recovery, combat | 0 | 1 | Affected |
| Heavy stun | 1 | 1 | Byte-identical |
| Paralysis | 1 | 1 | Byte-identical |
| No timed effect | 1 | 1 | Byte-identical |

Knockout fails before/pass after; **4/4 after cases pass**, all three controls
match byte-for-byte. Both knockout versions recover at turn511 with20HP and
kill the same kobold at turn519. The suppression flag is1 before/0 after.
`OUTCOME deaths=1` in this fixture counts **one monster killed**, not a player
death. The fixture separately asserts the player is alive. The kill-specific
reaction delta is0→1; additional hit/miss reactions are not extra kill events.

Practical benefit established: the recovered Borg receives combat information
that the original permanently discarded. A worse autonomous move, mortality
improvement or natural knockout frequency is **not demonstrated**. The fixture
enters STUN151 through the real timed API, places a visible legal kobold, then
uses normal engine recovery/combat and the actual Borg screen parser/input
boundary. An external driver chooses attacks and a declared headless renderer
shows native messages. This is not uninstrumented autonomous desktop play.
The actual paralysis message does not match the original parser spelling;
paralysis remains an unchanged negative control, not another proven trigger.

Normal native gate at7da46d00bc238e140a04cc83d2530b9469088acf:
**965/965 unit +5/5 frontend**, exit0. Runtime C is unchanged by later harness
merge/docs. All16 tooling controls pass. Numerical fixture summaries and
kill-reaction markers were checked, since its unit runner exits0 on assertions.

The frozen baseline/cohort-v1 remains unchanged: seeds11/29/47, Human Warrior,
turn20000, wall120s/stall30s. The actual merged source82 run also has identical
baseline traces and3/3 cap survival. The cumulative reset build is measured
after merge and published in the milestone; do not infer a survival benefit
from the controlled information-restoration result.

Copyable reproduction from the branch containing this carry:

```sh
git fetch source-tracker task/cbug-0002-evidence
# The immutable fixture above is used, not FETCH_HEAD.
flock /tmp/angband-validation.lock python3 tools/verify-native.py
```

Retain a native build of the immediate pre-carry consumer (source82 integrated)
in a separate worktree, then run with that build's absolute path as
`--before-build` and this product's build/native as the default after build:

```sh
flock /tmp/angband-validation.lock python3 tools/verify-knockout.py --before-build /home/diego/projects/angband/better-borg-c/.worktrees/safer-swap/build/native --output artifacts/knockout
```

The recorded path above is the actual retained consumer build used for this
receipt; portable before/after source pins and fresh-build commands are in the
milestone review package. Raw logs, object/binary/fixture hashes and result.json
stay under ignored artifacts/. This carry stays open/patch-active until upstream
resolution is incorporated, rebuilt and verified, independently of milestone
external review.
