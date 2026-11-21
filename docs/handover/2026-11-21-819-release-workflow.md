# 2026-11-21: release workflow, on demand only (#819)

## Done

- `.github/workflows/release.yml`: `workflow_dispatch` (version, ref,
  milestone, zoo URL) and a push of a `v*` tag; nothing else triggers it
  (`scripts/tests/test_release.py` pins that, and that the release is a
  draft). Jobs `setup`, `docs`, `source`, `linux`, `linux-clean`, `release`,
  adapted from upstream's `release.yaml`; docs/RELEASING.md describes each.
- `scripts/release/`: `package-linux.sh` (the package), `source-archive.sh`
  (HEAD plus `cargo vendor`, builds offline), `notes.py` (draft notes: the
  PLAN.md row, the handover notes since the previous `v*` tag, the zoo link
  or a "missing" line, the build identities), `rust-licenses.py`, and the
  package's own `watch-borg.sh`, `play-headless.sh` and `PLAYING.txt`.
- README points to docs/RELEASING.md, which also says how to install from a
  release.

## How it was checked (locally, not by a workflow run)

- Both builds (`harness/build.sh`, then `BORG_X11=ON`) play smoke with the
  keys, messages and states of `harness/reference/smoke-borg-rs`.
- The package, unpacked in a fresh directory: `SHA256SUMS` verifies, and
  `play-headless.sh OUT 101 0 0 20000` writes `keys.log`, `messages.log`
  and `state.log` byte-identical to smoke-warrior-1's reference.
- `watch-borg.sh` under Xvfb keeps its window open until the timeout (once
  the X11 bitmap fonts are installed, and with the UTF-8 locale the launcher
  now sets when there is none).
- The source archive, unpacked, builds with an empty `CARGO_HOME` and
  `CARGO_NET_OFFLINE=true`, with `harness/build.sh` (after a `git init`) and
  with plain CMake (no git).
- The Sphinx conversion of `engine/angband/docs` leaves the tree unchanged.
- `actionlint` reports nothing; `python3 -m unittest discover -s scripts/tests`
  passes.

**Not checked:** the workflow itself has not run. `workflow_dispatch` needs
the file on the default branch; the issue's dry run (a scratch tag making a
draft with a package that runs on a clean machine) is the owner's step after
merge (docs/RELEASING.md, *A dry run*). The `linux-clean` job is that clean
machine (a bare `ubuntu:24.04` container); it could not be run here (no
Docker daemon).

## Found

- The binary links OpenSSL 3's libcrypto and, built on ubuntu-24.04, needs
  glibc 2.39: the package runs on Ubuntu 24.04 / Debian 13 and newer.
  PLAYING.txt says so.
- The binaries are shipped unstripped (29 MB each, 7.5 MB stripped) so that
  their SHA-256 stays the one `BUILD_ID` names; the package is 38 MB.
- The source tree's `watch-borg.sh` copies no `borg.txt` into
  `.angband-reborn/user/`, so a watched game there does not play the shipped
  `harness/borg.txt` settings (and, per the comment in `harness/run.sh`,
  without a borg.txt naming a core the game falls back to the reborn core).
  The package's launcher copies `borg/borg.txt` and its scary list, so a
  release plays the shipped settings. #826 for the source launcher.

## Left

- Windows and macOS packages: not attempted (our CMake integration and the
  Rust staticlib have only been built on Linux): #823 (Windows), #824 (macOS).
- docs/user/GUIDE.md (#817) does not exist yet; when it does,
  `package-linux.sh` ships it as `docs/GUIDE.md` and its install section can
  point to docs/RELEASING.md.
- A release milestone row in PLAN.md (the issue mentions one, in its own PR).

## For the owner

- Merge, then the dry run: Actions → release → Run workflow with
  `v0.0.0-dryrun.1` on `main`; check the draft, then delete it.
- Version scheme: the workflow accepts any `v<digit>...`; a version with a
  `-` is marked a prerelease. Which numbers milestones get is yours.
