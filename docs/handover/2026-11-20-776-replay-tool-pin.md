## 2026-11-20: adopt historical replay controller/parameter fixes (#776)

Adopts Cemetery commit `4697893660942ed142744306c634583ec630f88a` through
death-tools-pin.txt, separately from Cemetery PR #91. API 1 and forwarding
arguments remain unchanged. The consumer test now expects an explicit reborn
controller for historical configs without borg_core. No game, policy, source
build input, harness default or reference stream changes.

Cemetery owns the implementation and all gameplay validation evidence:
https://github.com/diegoami/borg-cemetery/pull/91

The pinned tool clears ambient policy, restores environment/class-origin native
parameters and verifies configuration, parameter provenance, executable,
endpoint and all three original gameplay streams. Reborn and native logs have
separate requirements. Keys alone and --any-build never qualify a reproduction.
Custom borg.txt bytes unavailable from the exact source fail verification;
loaded-save replay remains outside the birth-dossier contract.

Both actual controller reproductions, source/build/game-data identities,
original natural deaths and compressed raw streams are retained only in
Cemetery at validation/borg-reborn-rs/historical-replay-settings/. There are
no duplicated gameplay reports here and no population-improvement claim.

Validation: the native-subtree guard and all 96 tools/tests pass with
BORG_CEMETERY_DIR pointing to the exact pinned Cemetery checkout. Cemetery's 130 repository and
79 native-tool tests pass; both exact-binary replay endpoints and all six
original/replayed gameplay-stream pairs match. Consumer CI is recorded in the
PR. The session-start --no-fetch source report made no unrelated pin changes.

Left: owner review-complete/merge decision for this adoption PR (#776), and
landing Cemetery #91. No next milestone or policy experiment is selected.

The owner-run MiniMax-M3 review of #777 approved the pinned adoption with three
nonblocking follow-ups. tools/README.md now states the stricter replay contract;
REFERENCE-RECEIPTS.md explicitly distinguishes its historical inspection pin
from the authoritative current death-tools-pin.txt rather than rewriting old
provenance. Keep the Cemetery task branch until #91 lands: merge that PR before
deleting the branch so the exact consumer pin remains reachable from main.
These follow-ups change documentation only; the native pin and tested behavior
are unchanged.
