# Releases

`.github/workflows/release.yml` (#819) bundles upstream Angband with the
Borg and makes a **draft** GitHub release. It is adapted from upstream's
`engine/angband/.github/workflows/release.yaml`, but runs **only on
demand**: never on a push to a branch or on a pull request (the runner slots
are shared with borg-zoo). A release is cut when a milestone is reached.

## Installing from a release

From the release page, download your platform's package and
`SHA256SUMS.txt`:

| platform | package | then |
| --- | --- | --- |
| Linux x86_64 | `borg-reborn-rs-VERSION-linux-x86_64.tar.gz` | `tar -xzf` it, `cd` into it, `./angband` |
| macOS, Apple silicon | `borg-reborn-rs-VERSION-macos-arm64.tar.gz` | the same; the first start may need *Open Anyway* (the binaries are not signed) |
| Windows x86_64 | `borg-reborn-rs-VERSION-windows-x86_64.zip` | *Extract all*, then double-click `angband.exe` in the folder |

Check the download first:

```sh
sha256sum -c --ignore-missing SHA256SUMS.txt
```

(macOS: `shasum -a 256 -c --ignore-missing SHA256SUMS.txt`; Windows:
compare `Get-FileHash` in PowerShell with the file's line.)

`PLAYING.txt` in the package says what each platform needs and how to start
the game; its `README.md` is upstream Angband's, with a section on the Borg
(`scripts/release/README-borg.md`): Ctrl-Z, then z. Nothing is installed:
saves go to `lib/save`, the Borg's settings are `lib/user/borg.txt`.

No `watch-borg` launcher and no `play-headless` runner are shipped (owner,
`v0.1.0-alpha.1`): the game is started directly; CI plays the packages with
`scripts/release/play-headless.sh` and `windows/play-headless.ps1`, given the
package folder as `BORG_PACKAGE_DIR`, and its `lib/` holds empty `save/`, `scores/`, `panic/`
and `user/archive/` folders and the shipped `lib/user/borg.txt`. Without
those, a direct start could not save on Windows (the engine creates them only
on Unix) and played the frozen reborn core. The macOS package is built and
shipped only with the workflow's `macos` input on. It is off by default:
macOS plays other games from the same seed, and nobody has watched it play
yet.

## What a release contains

| file | what |
| --- | --- |
| `borg-reborn-rs-V-linux-x86_64.tar.gz` | `angband` (X11, curses and headless frontends), `angband-headless` (headless only, no X11 library), upstream's `lib/`, the Borg's settings (`lib/user/borg.txt`, read by the game, as `harness/run.sh` prepares a run's, with its scary list; `borg/` holds the same for reference, with the class files and the errands plans), `PLAYING.txt`, `README.md`, the Angband manual as HTML, `LICENSES/`, each binary's `BUILD_ID` and `RELEASE.txt` (commit, upstream commit, borg-rs pin, settings hash), `SHA256SUMS` of every file |
| `borg-reborn-rs-V-macos-arm64.tar.gz` | only with the `macos` input: as Linux's, with one `angband` (curses and headless frontends; OpenSSL linked in, the system's curses) |
| `borg-reborn-rs-V-windows-x86_64.zip` | as Linux's, with one static `angband.exe` (curses and headless frontends, OpenSSL and ncurses linked in) |
| `borg-reborn-rs-V-src.tar.gz` | the committed tree at the release commit plus `cargo-vendor/` (the crates of `Cargo.lock`, each with its licence) and `.cargo/config.toml`, so it builds offline; `SOURCE.txt` says how |
| `SHA256SUMS.txt` | every archive's checksum |

The binaries are not stripped: their SHA-256 is the one their `BUILD_ID`
names, the build identity the zoo reads (docs/ZOO-INTERFACE.md). The other
`vendor/` checkouts of `scripts/setup.sh` (borg-rs, better-borg-c, borg-zoo,
...) are references to read, never build inputs, and are not shipped;
`vendor/angband-recorder` is in the source archive (it is tracked) but not
linked (`SUPPORT_BORG_RECORDING=OFF`).

Licences: Angband's (`engine/angband/docs/copying.rst`: GPL-2.0 or the
Angband licence), GPL-2.0-only for borg-rs's core (`borg-core/`) and ours
(`LICENSE`), and the crates' (MIT, Apache-2.0, Unlicense, Unicode-3.0),
listed by `scripts/release/rust-licenses.py`. The macOS and Windows packages
link OpenSSL 3 statically (Apache-2.0, `LICENSES/openssl.txt`); Windows' also
MSYS2's ncurses (MIT-style, as upstream's own Windows console build).

### What makes them portable (#823, #824)

- `engine/CMakeLists.txt`: `SUPPORT_GCU` (upstream's curses frontend, with
  upstream's curses detection); on Windows upstream's `main.c` in console
  mode (`WIN32_CONSOLE_MODE`), a static executable, and the patch series
  applied through MSYS2's bash; on macOS and Windows a static OpenSSL.
- `borg-reborn-rs/src/bindings_windows.rs`: the engine's layouts on Windows,
  where `wchar_t` is 16 bits, generated beside `bindings.rs` by
  `engine/bindings/generate.sh` (which needs the MinGW-w64 headers;
  `--check` compares both).
- `borg-core/` (declared hooks, `borg-reborn-rs (#823)`): its `wchar_t`
  comparisons cast to `i32` (no-ops where `wchar_t` is 32 bits), and
  `time`/`localtime` through the libc crate on Windows.
- `engine/frontend/main-borgh.c`: checkpoints (`-checkpoint-every`) need
  `fork()` and are refused on Windows; `setenv` is `_putenv_s` there.
- **One seed, one game, per kind of processor.** The engine draws random
  numbers inside function arguments (`loc(randint0(w), randint0(h))` in
  `mon-make.c`, `generate.c`, ...), and C leaves the order of a call's
  arguments to the compiler: GCC evaluates them right to left on x86-64,
  every arm64 compiler (Apple's clang, GCC) left to right, as clang does on
  x86-64 too. So the Linux and Windows packages (x86-64, GCC) play the
  games of `smoke-borg-rs` and of the zoo, while the macOS package (arm64)
  plays other games from the same seed, as deterministically: the `macos`
  job plays smoke twice with the same keys and the package replays them. A
  clang build on Linux plays exactly the macOS keys. The engine is
  upstream's, unchanged (RULES 9), so this is not fixed here; `BUILD_ID`
  records the compiler (`cc=`).
- `harness/build.sh` builds with `cmake --build` (Ninja under MSYS2);
  `harness/identity.sh` recognises the Rust Borg in Mach-O and PE files.

## The jobs

| job | what |
| --- | --- |
| `setup` | the version (`v...`, the tag) and the exact commit; a dispatched version that is already a tag must be that commit; `since` must be a tag or commit |
| `docs` | the Angband manual to HTML (Sphinx, upstream's `docconvert`), versioned with the upstream commit |
| `source` | `scripts/release/source-archive.sh` |
| `linux` | `harness/build.sh` headless, then with `BORG_X11=ON BORG_GCU=ON`; each binary plays the smoke suite on the default core with the same keys, messages and states as `harness/reference/smoke-borg-rs`; `scripts/release/package.sh linux` |
| `linux-clean` | in a bare `ubuntu:24.04` container with only the runtime libraries: the checksums, the licence files, `play-headless.sh` playing smoke-warrior-1's keys, messages and states byte for byte, and `watch-borg.sh` opening its window under Xvfb |
| `macos` | `macos-latest` (Apple silicon; `MACOSX_DEPLOYMENT_TARGET` 14.0), with Homebrew's GNU tools for the harness: the build, smoke twice with the same keys (arm64 plays other games than `smoke-borg-rs`, see above; the job prints how they differ), no Homebrew library linked, `package.sh macos`; then the unpacked package with macOS's own tools only (bash 3.2): the checksums and the three smoke games through `play-headless.sh`, byte for byte the harness's (`scripts/release/compare-games.py`); `watch-borg.sh` in a pseudo-terminal (informative) |
| `windows` | `windows-latest` with MSYS2's UCRT64 toolchain: the build (Ninja), no MSYS2 DLL linked, `package.sh windows`; then the unzipped package with PowerShell: the checksums, the three smoke games through CI's `windows/play-headless.ps1` byte for byte, and `angband.exe` started directly saving to `lib\save` |
| `release` | only when `release` is on (always for a tag push): the notes (`scripts/release/notes.py`), `SHA256SUMS.txt` and `gh release create --draft`; a version with a `-` is marked a prerelease |

## Cutting a release

1. Request a borg-zoo run of the exact commit to release (RULES 20); its
   measurement goes into the notes.
2. Actions → **release** → *Run workflow*: `version` (e.g. `v0.1.0`), `ref`
   (the commit or branch), `milestone` (its PLAN.md row goes into the
   notes, e.g. `P7`), `zoo` (the measurement's URL) and `since` (the tag
   or commit the notes' handover list starts after; empty: the previous
   `v*` tag, or every note for the first release). Or push a tag
   `v...` on the commit (the notes then have no milestone and no zoo link:
   edit them).
3. Read the draft: the notes list the handover notes added or changed since the
   previous `v*` tag, the PLAN.md row and the build identities; edit them,
   then publish. Publishing creates the tag when it does not exist yet.

A version that already has a release, draft or published, fails in the
`release` job (`gh release create` never overwrites one). To re-run a failed
or wrong release, delete its draft first (Releases → the draft → Delete).

The workflow runs from the default branch's copy of `release.yml`
(`workflow_dispatch`) or the tag's (`push`).

## A dry run

To check the packages without making a release, dispatch with `release`
off: every job runs except `release`, and the packages are the run's
artifacts (kept three days). This also works on a branch: *Use workflow
from* the branch, and the same branch as `ref`.

Use a prerelease version that is clearly scratch, e.g. `v0.0.0-dryrun.1`,
dispatched on the commit to try. Check the draft and its packages, then
delete the draft release (Releases → the draft → Delete). A dispatched
draft creates no tag; a dry run by tag push leaves the tag, which is then
deleted by hand (`git push origin :refs/tags/v0.0.0-dryrun.1`) with the
owner's agreement (RULES 5: tags are not moved).
