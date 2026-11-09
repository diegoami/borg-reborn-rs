#!/usr/bin/env bash
# Named visible Rust launcher. Immutable builds and session-local recordings.
set -euo pipefail
root=$(cd -- "$(dirname -- "$0")" && pwd)
exec python3 "$root/tools/start_recorded_borg.py" "$@"
