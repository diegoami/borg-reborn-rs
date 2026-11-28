#!/usr/bin/env bash
# Run each engine test module in its own process, with the job's fixture environment.
# The optional argument names a module exercised in a separate job.
set -euo pipefail
find engine/tests -maxdepth 1 -type f -name 'test_*.py' ! -name "${1:-}" -printf '%f\0' |
    xargs -0 -n 1 -P "$(nproc)" python3 -m unittest discover -s engine/tests -p
