#!/usr/bin/env bash
# One headless Borg game from an unpacked release package (#819, #824): CI's
# check that a package plays the reference games. Not shipped (owner,
# v0.1.0-alpha.1); the package folder is BORG_PACKAGE_DIR.
#   BORG_PACKAGE_DIR=DIR scripts/release/play-headless.sh OUTDIR [SEED [RACE [CLASS [TURNS]]]]
# Plays a new character with no window (the borgh frontend) until game turn
# TURNS (default 50000; 0 = no limit) or its death, and writes
# OUTDIR/keys.log, messages.log, state.log and summary.json. RACE and CLASS
# are indices into lib/gamedata/p_race.txt and class.txt, picked with the
# birth menu's letters (defaults: seed 101, Human Warrior). The settings are
# borg/pborg.yaml with the race and class set, as harness/run.sh sets them, so
# seed 101 0 0 20000 plays the same keys as the source tree's smoke-warrior-1
# reference (harness/reference/smoke-borg-rs).
set -euo pipefail
PKG="$(cd "${BORG_PACKAGE_DIR:?set BORG_PACKAGE_DIR to the unpacked package}" && pwd)"
[ $# -ge 1 ] && [ "$1" != -h ] && [ "$1" != --help ] || { sed -n '2,11p' "$0"; [ $# -ge 1 ]; exit; }
SEED="${2:-101}"; RACE="${3:-0}"; CLASS="${4:-0}"; TURNS="${5:-50000}"
for v in "$SEED" "$RACE" "$CLASS" "$TURNS"; do
    [[ "$v" =~ ^[0-9]{1,10}$ ]] || { echo "play-headless.sh: SEED RACE CLASS TURNS are numbers" >&2; exit 2; }
done
[ ! -e "$1" ] || { echo "play-headless.sh: $1 exists; use a new directory" >&2; exit 2; }
mkdir -p "$1"; OUT="$(cd "$1" && pwd)"
mkdir -p "$OUT/user" "$OUT/save" "$OUT/archive" "$OUT/panic" "$OUT/scores"
cp "$PKG"/borg/*.txt "$OUT/user/"
sed -e "s/^borg_respawn_race: .*/borg_respawn_race: $RACE/" \
    -e "s/^borg_respawn_class: .*/borg_respawn_class: $CLASS/" \
    -e "s/^borg_cheat_death: .*/borg_cheat_death: false/" "$PKG/borg/pborg.yaml" > "$OUT/user/pborg.yaml"
# The headless-only executable where there is one (Linux), else the game's.
BIN=./angband; [ ! -x "$PKG/angband-headless" ] || BIN=./angband-headless
# A UTF-8 locale (C.UTF-8 is glibc's; macOS has en_US.UTF-8).
LOC=C.UTF-8; [ "$(uname -s)" != Darwin ] || LOC=en_US.UTF-8
cd "$PKG"   # the game finds ./lib relative to the working directory
set +e
TZ=UTC LANG=$LOC LC_ALL=$LOC "$BIN" -mborgh -n -uBorg \
    -duser="$OUT/user" -dsave="$OUT/save" -darchive="$OUT/archive" \
    -dpanic="$OUT/panic" -dscores="$OUT/scores" -- \
    -seed "$SEED" -race "$RACE" -class "$CLASS" -turns "$TURNS" -menu-letters \
    -keylog "$OUT/keys.log" -msglog "$OUT/messages.log" -statelog "$OUT/state.log" \
    -summary "$OUT/summary.json" -screen 2>"$OUT/stderr.log"
RC=$?
set -e
[ "$RC" -eq 0 ] && [ -s "$OUT/summary.json" ] || { echo "play-headless.sh: the game failed (rc=$RC); see $OUT/stderr.log" >&2; exit 1; }
cat "$OUT/summary.json"; echo
