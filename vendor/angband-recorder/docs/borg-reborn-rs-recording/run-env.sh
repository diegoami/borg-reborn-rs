# Source this before harness/run.sh in the recording build directory.
# The owner's Borg parameters (2026-11-17) plus the recording settings.
# BRR_BUILD: the recording build directory made by setup.sh. When it is not set, the build that
# ~/angband-builds/current points at (the latest setup.sh made), else the first build.
# The link is resolved, so a recording keeps the path of the build that made it.
if [ -z "${BRR_BUILD:-}" ]; then
    if [ -d "$HOME/angband-builds/current" ]; then
        BRR_BUILD="$(realpath "$HOME/angband-builds/current")"
    else
        if [ -L "$HOME/angband-builds/current" ]; then
            echo "run-env.sh: warning: ~/angband-builds/current is a dangling link; using the first build (recorder v0.2.0, no closing frame). Re-run setup.sh or re-point the link." >&2
        fi
        BRR_BUILD="$HOME/angband-builds/borg-reborn-rs-recording"
    fi
fi
export BRR_BUILD
export BORG_CORE=borg-rs
export BORG_ERRANDS_PLAN=harness/plans/errands_plan.txt
export BORG_TOOLKIT_NATIVE=1
# A save every 1000 game turns (about 100 player turns, ~50 KB each), for rewind.sh.
export CHECKPOINT_EVERY=1000
export ANGBAND_BIN="$BRR_BUILD/build/rec/game/angband"
export BORG_PARAM_ATTACK_PREFERENCE_BLACKGUARD_MELEE_PERMILLE=1600
export BORG_PARAM_ATTACK_PREFERENCE_BLACKGUARD_SPELL_PERMILLE=1400
export BORG_PARAM_ATTACK_PREFERENCE_DRUID_SPELL_PERMILLE=1700
export BORG_PARAM_ATTACK_PREFERENCE_MAGE_DEVICE_PERMILLE=1200
export BORG_PARAM_ATTACK_PREFERENCE_MAGE_SPELL_PERMILLE=2000
export BORG_PARAM_ATTACK_PREFERENCE_NECROMANCER_DEVICE_PERMILLE=1200
export BORG_PARAM_ATTACK_PREFERENCE_NECROMANCER_SPELL_PERMILLE=1800
export BORG_PARAM_ATTACK_PREFERENCE_PALADIN_MELEE_PERMILLE=1800
export BORG_PARAM_ATTACK_PREFERENCE_PALADIN_SPELL_PERMILLE=1200
export BORG_PARAM_ATTACK_PREFERENCE_PRIEST_SPELL_PERMILLE=1700
export BORG_PARAM_ATTACK_PREFERENCE_RANGER_PROJECTILE_PERMILLE=2000
export BORG_PARAM_ATTACK_PREFERENCE_ROGUE_PROJECTILE_PERMILLE=1300
export BORG_PARAM_ATTACK_PREFERENCE_ROGUE_THROWN_PERMILLE=1600
export BORG_PARAM_ATTACK_PREFERENCE_WARRIOR_MELEE_PERMILLE=1800
export BORG_PARAM_ATTACK_PREFERENCE_WARRIOR_PROJECTILE_PERMILLE=1200
export BORG_PARAM_ATTACK_PREFERENCE_WARRIOR_THROWN_PERMILLE=1200
export BORG_PARAM_CAUTION_HEAL_WHEN_OUTMATCHED=1
