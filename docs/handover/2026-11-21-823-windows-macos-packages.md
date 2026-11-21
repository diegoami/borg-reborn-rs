# 2026-11-21: Windows and macOS packages (#823, #824)

The owner, after the #819 dry run: "add the since input. Take on the other
issues such as windows and mac."

## Done

- **The release workflow** (`.github/workflows/release.yml`) has `macos` and
  `windows` jobs beside `linux`, and two new inputs: `since` (where the
  notes' handover list starts) and `release` (off: build and check the three
  packages without a draft). Build-only run
  [44926762215](https://github.com/diegoami/borg-reborn-rs/actions/runs/44926762215)
  passed every job: Linux 36.9 MB, macOS 26.8 MB, Windows 31 MB, source
  62.2 MB.
- **Windows (x86-64)**: MSYS2's UCRT64 toolchain, as upstream's own Windows
  release. Upstream's `main.c` in console mode (`WIN32_CONSOLE_MODE`) with
  the curses and borgh frontends; one static `angband.exe` (OpenSSL and
  ncursesw linked in; only system DLLs, checked). Launchers:
  `play-headless.cmd`/`.ps1`, `watch-borg.cmd`.
- **macOS (Apple silicon)**: `macos-latest` (macOS 14 deployment target),
  Apple's clang, OpenSSL linked in statically, the system's curses (no
  Homebrew library linked, checked). The launchers work with macOS's own
  bash 3.2 and BSD tools.
- **Linux**: the window build has the curses frontend too
  (`watch-borg.sh --terminal`, and without a display).
- The engine side, each a no-op on Linux:
  - `engine/CMakeLists.txt`: `SUPPORT_GCU`, Windows console mode and static
    link, `apply-patches.sh` through bash on Windows, static OpenSSL on
    macOS/Windows, `BORG_RUST_TARGET` for cross builds;
  - `borg-reborn-rs/src/bindings_windows.rs`: Windows layouts (`wchar_t` is
    16 bits there), generated beside `bindings.rs` by `generate.sh`, whose
    `--check` now compares both (CI installs `mingw-w64-x86-64-dev`);
  - `borg-core/` (already declared HOOKED files, marked
    `borg-reborn-rs (#823)`, no line moved: parameter citations unchanged):
    `wchar_t` compares cast to `i32`; `time`/`localtime` through the libc
    crate on Windows;
  - `engine/frontend/main-borgh.c`: checkpoints refused on Windows (no
    `fork`), `setenv` as `_putenv_s`, the logs opened in binary mode
    (Windows wrote CRLF);
  - `harness/build.sh`: `cmake --build`, `angband.exe`, `cc=$CC` in
    `BUILD_ID`; `harness/identity.sh`: Rust recognised in Mach-O and PE by
    `rust_eh_personality`.
- `scripts/release/package.sh PLATFORM` replaces `package-linux.sh`;
  `compare-games.py` compares a package's games with a reference or a
  same-platform harness run, without `xz`.

## How it was checked

- Linux: headless and X11+curses builds play smoke identical to
  `smoke-borg-rs` (locally and in the run); the package's game equals
  smoke-warrior-1; `watch-borg.sh` stays up under Xvfb and in a terminal;
  fmt, clippy, both test suites, `generate.sh --check`, engine and scripts
  tests, parameter citations.
- Windows (CI): the unzipped package, through PowerShell, plays the three
  smoke games **byte-identical to `smoke-borg-rs`**.
- macOS (CI): smoke twice with the same keys; the unpacked package, with
  macOS's own tools, replays the harness's three games byte for byte;
  `watch-borg.sh` keeps its curses window up in a pseudo-terminal.
- Locally: the Windows Rust crate and every C source cross-compile with
  MinGW and link; Wine crashes in this container, so nothing ran here.

**Not checked:** `watch-borg.cmd` (the curses frontend in a Windows
console) has not run: no interactive console on the runner. Nobody has
played a package on a real Windows or Mac.

## Found

- **The compiler decides the game.** The engine draws random numbers inside
  function arguments (`loc(randint0(w), randint0(h))` in `mon-make.c`,
  `generate.c`, ...), and C leaves their order to the compiler: GCC on
  x86-64 evaluates right to left; GCC on arm64 and clang everywhere, left to
  right. So the macOS (arm64) package plays other games from the same seed:
  each deterministic, but not `smoke-borg-rs`'s. A clang build on Linux
  plays exactly the macOS keys; Homebrew GCC on arm64 plays them too. The
  engine is upstream's (RULES 9), so it is not fixed; RELEASING.md and
  PLAYING.txt say so.
- The same applies to any clang build of the harness on Linux: it would not
  match the references or the zoo's games, and nothing stops one today
  (`BUILD_ID` now records the compiler).
- GitHub has retired the `macos-14` runner image.

## Left

- #826 (the source tree's `watch-borg.sh` and its borg.txt) is not done here.
- The `v0.0.0-dryrun.1` draft from the #819 dry run still needs deleting by
  the owner (this session cannot delete releases).

## For the owner

- Is a macOS package that plays different games from the same seed
  acceptable for a release (the notes would say so), or should macOS wait?
- Should `harness/build.sh` refuse (or warn about) a compiler other than
  GCC on x86-64, since such a build plays games the references and the zoo
  never see?
