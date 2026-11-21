# Native verification gate

```sh
flock /tmp/angband-validation.lock python3 tools/verify-native.py
```

This configures Angband with Borg support and the test frontend, builds with
two jobs and runs Angband's `alltests` target (including `allunittests`). It
checks the numerical unit/frontend summaries as well as exit codes: the
original unit runner can return zero after a failed assertion. Logs and
hashed results go to ignored `artifacts/native/`. These baseline checks do
not establish Borg survival; each adoption also needs the source bug's legal
scenario and controls.

The `flock` wraps the shared `/tmp/angband-validation.lock`, so heavy local
builds/scenarios from different tasks don't run concurrently.

## Continuous integration

Native verification runs on GitHub-hosted `ubuntu-latest` for main/PR/manual
events. The explicitly owner-requested donor inventory also runs weekly on
Mondays 09:15 Europe/Berlin and by manual dispatch, reporting to
[ongoing issue24](https://github.com/diegoami/better-borg-c/issues/24). It reads
public C source metadata and queues semantic review; it never executes/imports
fork code or proves better play from activity. Bugs still take priority.

Neither active workflow publishes. Inherited workflows remain archived outside
`.github/workflows/`. No tag/release, lab deployment or private backup is part
of setup. GitHub schedules are best-effort and may be delayed/disabled after
public-repository inactivity; ongoing issue24 records run receipts and follow-up.

See [docs/measurement/RECORD-REPLAY.md](measurement/RECORD-REPLAY.md) for the
native recording/replay qualification results and limitations, and
[docs/deaths/README.md](deaths/README.md) for how natural deaths are recorded.
