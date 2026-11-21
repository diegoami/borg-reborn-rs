#!/usr/bin/env bash
# Make a recording build of borg-reborn-rs:  setup.sh [REV]   (default REV: origin/main, after a fetch)
# Creates a detached git worktree at $BRR_BUILD (default ~/angband-builds/borg-reborn-rs-recording-<REV>,
# with REV as 8 hex digits), applies the return settings to its harness/borg.txt and builds the headless
# game with the recorder on into build/rec, then points ~/angband-builds/current at it (NO_CURRENT=1 skips
# that), which is the build run-env.sh and the other scripts use when BRR_BUILD is not set.
# Refuses an existing $BRR_BUILD: keep old builds, they replay old recordings.
set -euo pipefail
: "${BORG_REBORN_RS:=$HOME/projects/angband/borg-reborn-rs}"
REV="${1:-origin/main}"
# The local main of that checkout is not moved by a fetch and can be far behind (with an old recorder pin).
git -C "$BORG_REBORN_RS" fetch --quiet origin || echo "setup.sh: warning: fetch failed; using what is local" >&2
FULL="$(git -C "$BORG_REBORN_RS" rev-parse --verify --quiet "$REV^{commit}")" \
    || { echo "setup.sh: no such revision: $REV" >&2; exit 2; }
: "${BRR_BUILD:=$HOME/angband-builds/borg-reborn-rs-recording-${FULL:0:8}}"
[ ! -e "$BRR_BUILD" ] || { echo "setup.sh: $BRR_BUILD exists; set BRR_BUILD to a new directory (or unset it)" >&2; exit 2; }
echo "setup.sh: building $REV = ${FULL:0:8} into $BRR_BUILD"
git -C "$BORG_REBORN_RS" worktree add --detach "$BRR_BUILD" "$FULL"
cd "$BRR_BUILD"
sed -i -e 's/^borg_trip_policy = .*/borg_trip_policy = 1/' \
       -e 's/^borg_trip_pack_slack = .*/borg_trip_pack_slack = 1/' \
       -e 's/^borg_trip_unknown_stacks = .*/borg_trip_unknown_stacks = 5/' harness/borg.txt
git diff --stat
flock /tmp/angband-validation.lock bash -c 'cmake -S engine -B build/rec -DCMAKE_BUILD_TYPE=Release \
    -DSUPPORT_BORG_RECORDING=ON -DSUPPORT_X11=OFF >build-rec.cmake.log 2>&1 \
    && make -C build/rec -j"$(nproc)" >build-rec.make.log 2>&1' \
    || { tail -30 build-rec.make.log build-rec.cmake.log 2>/dev/null; exit 1; }
sha256sum build/rec/game/angband
if [ "${NO_CURRENT:-0}" != 1 ]; then
    ln -sfnT "$BRR_BUILD" "$HOME/angband-builds/current"
    echo "setup.sh: ready; ~/angband-builds/current now points at it (the scripts use it when BRR_BUILD is not set)."
else
    echo "setup.sh: ready. export BRR_BUILD=$BRR_BUILD to use it (current was not changed)."
fi
