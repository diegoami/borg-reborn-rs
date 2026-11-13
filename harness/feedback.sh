#!/usr/bin/env bash
# The end-of-session feedback: how the run went, what the owner thought of
# the buying strategy, and the corrections.
#
# Usage: harness/feedback.sh CHAR_DIR
#
# CHAR_DIR is the character's directory in a session (OUTDIR/char-N). It
# holds objects.log, the numbered log of the objects the run touched.
#
# Two files are written there:
#
#   feedback.md        the whole answer, in the order it was given
#   corrections.jsonl  one record per correction, with the number of the
#                      object log line it is about:
#                      {"line": 12, "text": "bought scroll on accident"}
#
# The corrections are the labels: the owner points at a line of the object
# log and says what was wrong with it. That is the cheap way to get the
# labels the usefulness table needs - at the end of the run, in the owner's
# own words, instead of one label at a time.
#
# With no terminal on stdin (a scripted run) nothing is asked and an empty
# feedback file is written.
set -uo pipefail

DIR="${1:-}"
[ -n "$DIR" ] || {
    echo "feedback.sh: usage: feedback.sh CHAR_DIR" >&2
    exit 2
}
[ -d "$DIR" ] || {
    echo "feedback.sh: no such directory: $DIR" >&2
    exit 2
}

OBJECTS="$DIR/objects.log"
[ -f "$OBJECTS" ] || OBJECTS=""

# The number of objects the run touched.
COUNT=0
if [ -n "$OBJECTS" ]; then
    COUNT=$(grep -c . "$OBJECTS" 2>/dev/null | head -1)
    COUNT=${COUNT:-0}
fi

# read_one <prompt> -> the line on stdout (empty when the answer is empty).
# The prompt goes to stderr: stdout is the answer.
read_one() {
    printf '%s' "$1" >&2
    IFS= read -r line || line=""
    printf '%s' "$line"
}

if [ ! -t 0 ]; then
    {
        echo "# Session feedback"
        echo
        echo "- characters: $DIR"
        echo "- objects logged: $COUNT"
        echo
        echo "## The run"
        echo
        echo "_No feedback collected: the session was not run from a terminal._"
        echo
        echo "## The buying strategy"
        echo
        echo "## Corrections"
    } > "$DIR/feedback.md"
    : > "$DIR/corrections.jsonl"
    echo "feedback.sh: no terminal - wrote an empty $DIR/feedback.md"
    exit 0
fi

echo
echo "feedback.sh: the run is over."
if [ -n "$OBJECTS" ]; then
    echo "feedback.sh: the objects of the run are in $OBJECTS ($COUNT lines)."
    if [ "$COUNT" -gt 0 ] && [ "$COUNT" -le 40 ]; then
        echo
        sed 's/^/    /' "$OBJECTS"
    else
        echo "feedback.sh: (open $OBJECTS to see them; a correction is '<number>: what was wrong')"
    fi
fi
echo

RUN_ANSWER=$(read_one "1/3 - how did the run go? (a line, empty line to skip)")
echo
STRATEGY_ANSWER=$(read_one "2/3 - what did you think of the buying strategy? (a line, empty line to skip)")
echo
echo "3/3 - corrections: one per line, '<number of the object log>: what was wrong'."
echo "      An empty line finishes. The whole run is fine? Leave it empty."
CORRECTIONS=""
while IFS= read -r line; do
    [ -z "$line" ] && break
    CORRECTIONS="${CORRECTIONS}${line}
"
    echo "    noted: $line"
done
echo

{
    echo "# Session feedback"
    echo
    echo "- characters: $DIR"
    echo "- objects logged: $COUNT"
    [ -n "$OBJECTS" ] && echo "- object log: $OBJECTS"
    echo
    echo "## The run"
    echo
    echo "${RUN_ANSWER:-(nothing said)}"
    echo
    echo "## The buying strategy"
    echo
    echo "${STRATEGY_ANSWER:-(nothing said)}"
    echo
    echo "## Corrections"
    echo
    if [ -n "$CORRECTIONS" ]; then
        printf '%s' "$CORRECTIONS"
    else
        echo "_None._"
    fi
} > "$DIR/feedback.md"

# The corrections, one record each, with the object log line they are about.
: > "$DIR/corrections.jsonl"
if [ -n "$CORRECTIONS" ]; then
    while IFS= read -r line; do
        [ -z "$line" ] && continue
        printf '%s\n' "$line" | python3 -c '
import json, sys
raw = sys.stdin.read().strip()
number, sep, text = raw.partition(":")
if not sep:
    print(json.dumps({"line": None, "text": raw}))
else:
    text = text.strip()
    try:
        line = int(number.strip())
    except ValueError:
        line = None
    print(json.dumps({"line": line, "text": text}))
' >> "$DIR/corrections.jsonl"
    done <<< "$CORRECTIONS"
fi

echo "feedback.sh: wrote $DIR/feedback.md and $DIR/corrections.jsonl"
