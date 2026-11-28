#!/usr/bin/env bash
# Run engine tests with the job's fixture environment.
set -euo pipefail
jobs=$(nproc)

if [[ ${1:-} == --fixtures ]]; then
    # Preserve the checks and class selectors that require the test build.
    test -x build/rust-test/game/angband
    xargs -r -d '\n' -n 1 -P "$jobs" bash -c '
        read -r -a args <<< "$1"
        python3 .github/scripts/run-engine-test.py "engine/tests/${args[0]}" "${args[@]:1}"
    ' _ <<'FIXTURES'
test_combat_heal.py
test_town_loadout.py
test_exploration_pack.py
test_town_recall_loadout.py
test_cursed_spares.py
test_junk_consumables.py
test_inferior_spares.py TownSpareSaleTest
test_store_loop.py
test_stair_handoff.py EmergencyStairHandoffTest
test_stair_handoff.py TownRecallReadinessTest
test_stair_handoff.py HealingRestockTest
test_stair_handoff.py LeaveLevelUpstairsTest
test_stair_handoff.py DarkVeinLoopTest
test_stair_handoff.py RecoveryManaLoopTest
test_stair_handoff.py StaleStairMoreTest
test_stair_handoff.py MoreRefusedGatesTest
test_stair_handoff.py NoteStandingFlagsTest
test_stair_handoff.py BreederDoorLoopTest
test_stair_handoff.py DiggerSwapLoopTest
test_stair_handoff.py StaleKoboldDoorTest
test_stair_handoff.py StairScumLootRoutesTest
FIXTURES
    exit
fi

# The named exclusion runs in the other core job.
excluded=${1:-}
if [[ -n $excluded ]]; then
    test -f "engine/tests/$excluded"
fi

shopt -s nullglob
modules=()
for module in engine/tests/test_*.py; do
    case "${module##*/}" in
        test_build.py|"$excluded") ;;
        *) modules+=("$module") ;;
    esac
done
if (( ${#modules[@]} == 0 )); then
    echo 'No engine test modules found' >&2
    exit 1
fi
printf '%s\0' "${modules[@]}" | xargs -0 -r -n 1 -P "$jobs" python3 .github/scripts/run-engine-test.py

# This module changes the recorder patch mtime to exercise CMake reconfiguration.
python3 .github/scripts/run-engine-test.py engine/tests/test_build.py
