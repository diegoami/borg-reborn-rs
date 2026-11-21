#!/usr/bin/env bash
# Make a recording build of borg-reborn-rs:  setup.sh [REV]   (default REV: main)
# Creates a detached git worktree at $BRR_BUILD (default ~/angband-builds/borg-reborn-rs-recording),
# applies the return settings to its harness/borg.txt and builds the headless game with the
# recorder on into build/rec. Refuses an existing $BRR_BUILD: keep old builds, they replay old recordings.
set -euo pipefail
: "${BORG_REBORN_RS:=$HOME/projects/angband/borg-reborn-rs}"
: "${BRR_BUILD:=$HOME/angband-builds/borg-reborn-rs-recording}"
REV="${1:-main}"
[ ! -e "$BRR_BUILD" ] || { echo "setup.sh: $BRR_BUILD exists; set BRR_BUILD to a new directory" >&2; exit 2; }
git -C "$BORG_REBORN_RS" worktree add --detach "$BRR_BUILD" "$REV"
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
echo "setup.sh: ready. export BRR_BUILD=$BRR_BUILD if it is not the default."
