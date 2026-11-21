# Releases

`.github/workflows/release.yml` (#819) bundles upstream Angband with the
Borg and makes a **draft** GitHub release. It is adapted from upstream's
`engine/angband/.github/workflows/release.yaml`, but runs **only on
demand**: never on a push to a branch or on a pull request (the runner slots
are shared with borg-zoo). A release is cut when a milestone is reached.

## Installing from a release

From the release page, download `borg-reborn-rs-VERSION-linux-x86_64.tar.gz`
and `SHA256SUMS.txt`, then:

```sh
sha256sum -c --ignore-missing SHA256SUMS.txt
```

```sh
tar -xzf borg-reborn-rs-VERSION-linux-x86_64.tar.gz
```

```sh
cd borg-reborn-rs-VERSION-linux-x86_64 && ./watch-borg.sh
```

`PLAYING.txt` in the package says what it needs (Linux x86_64, glibc 2.39
or newer, OpenSSL 3's libcrypto; for the window libX11 and the X11 bitmap
fonts), how to watch the Borg (`watch-borg.sh`: create a character, Ctrl-Z
then z) and how to play a game without a window (`play-headless.sh OUTDIR
SEED RACE CLASS TURNS`). Nothing is installed: saves and settings go to
`data/` next to the launchers. Only Linux is built; Windows and macOS are
follow-ups.

## What a release contains

| file | what |
| --- | --- |
| `borg-reborn-rs-V-linux-x86_64.tar.gz` | `angband` (X11 and headless frontends), `angband-headless` (headless only, no X11 library), upstream's `lib/`, the Borg's settings (`borg/borg.txt` as `harness/run.sh` prepares a run's, its scary list, the class files and the errands plans), the launchers, `PLAYING.txt`, `README.md`, the Angband manual as HTML, `LICENSES/`, each binary's `BUILD_ID` and `RELEASE.txt` (commit, upstream commit, borg-rs pin, settings hash), `SHA256SUMS` of every file |
| `borg-reborn-rs-V-src.tar.gz` | the committed tree at the release commit plus `cargo-vendor/` (the crates of `Cargo.lock`, each with its licence) and `.cargo/config.toml`, so it builds offline; `SOURCE.txt` says how |
| `SHA256SUMS.txt` | the two archives' checksums |

The binaries are not stripped: their SHA-256 is the one their `BUILD_ID`
names, the build identity the zoo reads (docs/ZOO-INTERFACE.md). The other
`vendor/` checkouts of `scripts/setup.sh` (borg-rs, better-borg-c, borg-zoo,
...) are references to read, never build inputs, and are not shipped;
`vendor/angband-recorder` is in the source archive (it is tracked) but not
linked (`SUPPORT_BORG_RECORDING=OFF`).

Licences: Angband's (`engine/angband/docs/copying.rst`: GPL-2.0 or the
Angband licence), GPL-2.0-only for borg-rs's core (`borg-core/`) and ours
(`LICENSE`), and the crates' (MIT, Apache-2.0, Unlicense, Unicode-3.0),
listed by `scripts/release/rust-licenses.py`.

## The jobs

| job | what |
| --- | --- |
| `setup` | the version (`v...`, the tag) and the exact commit; a dispatched version that is already a tag must be that commit |
| `docs` | the Angband manual to HTML (Sphinx, upstream's `docconvert`), versioned with the upstream commit |
| `source` | `scripts/release/source-archive.sh` |
| `linux` | `harness/build.sh` headless, then with `BORG_X11=ON`; each binary plays the smoke suite on the default core with the same keys, messages and states as `harness/reference/smoke-borg-rs`; `scripts/release/package-linux.sh` |
| `linux-clean` | in a bare `ubuntu:24.04` container with only the runtime libraries: the checksums, the licence files, `play-headless.sh` playing smoke-warrior-1's keys, messages and states byte for byte, and `watch-borg.sh` opening its window under Xvfb |
| `release` | the notes (`scripts/release/notes.py`), `SHA256SUMS.txt` and `gh release create --draft`; a version with a `-` is marked a prerelease |

## Cutting a release

1. Request a borg-zoo run of the exact commit to release (RULES 20); its
   measurement goes into the notes.
2. Actions → **release** → *Run workflow*: `version` (e.g. `v0.1.0`), `ref`
   (the commit or branch), `milestone` (its PLAN.md row goes into the
   notes, e.g. `P7`) and `zoo` (the measurement's URL). Or push a tag
   `v...` on the commit (the notes then have no milestone and no zoo link:
   edit them).
3. Read the draft: the notes list the handover notes added since the
   previous `v*` tag, the PLAN.md row and the build identities; edit them,
   then publish. Publishing creates the tag when it does not exist yet.

The workflow runs from the default branch's copy of `release.yml`
(`workflow_dispatch`) or the tag's (`push`).

## A dry run

Use a prerelease version that is clearly scratch, e.g. `v0.0.0-dryrun.1`,
dispatched on the commit to try. Check the draft and its packages, then
delete the draft release (Releases → the draft → Delete). A dispatched
draft creates no tag; a dry run by tag push leaves the tag, which is then
deleted by hand (`git push origin :refs/tags/v0.0.0-dryrun.1`) with the
owner's agreement (RULES 5: tags are not moved).
