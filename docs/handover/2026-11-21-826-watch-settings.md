# 2026-11-21: watched games play the shipped settings (#826)

## Done

- `harness/prepare-borg-txt.sh DIR`: writes `DIR/borg.txt` and its scary
  list as `harness/run.sh` prepares a run's: the world key removed and the
  scary list's selecting line moved to the end under its SHA-256. It is the
  one place for this; `scripts/release/package.sh` used to have its own
  copy, which now calls the helper and writes the same bytes as before.
- `watch-borg.sh`:
  - the first start writes the shipped settings into
    `.angband-reborn/user/`, so a watched game plays the borg-rs core with
    `harness/borg.txt` (before this, no `borg.txt` was written and the
    frozen reborn core played);
  - later starts keep that `borg.txt`, yours to edit, and say whether it
    differs from the shipped one;
  - `--reset-settings` writes the shipped one again.
- `harness/play.sh`: each character's `user/` gets the shipped settings too,
  so a town session plays the borg-rs core (both cores implement the
  `BORG_TOWN_PLAYER` handover). `config.txt` records `borg_core` and
  `borg_txt_sha256`.
- `README.md`: the manual "copy borg.txt" step is replaced by what the
  launcher now does, and by `--reset-settings`.

## Checked

- Under Xvfb with a fresh data directory:
  - the first start writes the shipped settings;
  - an edited `borg.txt` is kept, with the "differs" note;
  - `--reset-settings` restores the shipped bytes;
  - no temporary directory is left behind (the launcher ends in `exec`).
- `scripts/tests/test_prepare_borg_txt.py`: the helper's output, and that
  all three launchers use it. Full `scripts/tests` pass.

**Not checked:** a long watched game, or a `play.sh` session played by hand.

## For the owner

- `harness/play.sh` now plays the borg-rs core by default. It played the
  frozen reborn core before, without anyone choosing that. `BORG_CORE=reborn`
  still selects the reborn core.

## Added: a game started directly from a package (owner, on v0.1.0-alpha.1)

The owner double-clicked `angband.exe` in the Windows package: "save
failed". Started without `-d` options, the game uses `lib/save`,
`lib/scores`, `lib/panic`, `lib/user` and `lib/user/archive`. The engine
creates them only on Unix (`create_needed_dirs` is inside `#ifdef UNIX`), and
the package shipped none of them. `lib/user` also had no `borg.txt`, so a
game started that way played the frozen reborn core.

- `scripts/release/package.sh`, on every platform:
  - ships those folders, empty;
  - drops anything games in the build tree left there;
  - writes the shipped settings into `lib/user`;
  - writes folder entries into the zip, so empty folders survive.
- The release `windows` job:
  - saves a game with the directories as `watch-borg.cmd` passes them
    (backslash paths);
  - starts the unzipped `angband.exe` with no `-d` options and saves to
    `lib\save`;
  - checks that `lib\user\borg.txt` selects the borg-rs core.
- `PLAYING.txt`: a direct start works, with saves in `lib/save`.
- `v0.1.0-alpha.1`'s draft has the old packages. It needs a new release
  run after this merges, and its draft deleted first.

## Review of #866 (`7fed1031`)

- **Finding 1, fixed.** On later starts, `watch-borg.sh` copied nothing, so
  a kept `borg.txt` (identical or edited) whose scary list was missing made
  the borg-rs core exit at start. Now every missing shipped file is copied
  on every start, and a `borg.txt` naming a list that is not there stops the
  launcher with a clear message. The empty-glob trap is gone, and
  `--build-only --reset-settings` says the reset waits for the next start.
- **Finding 2, fixed.** The Windows steps give MSYS tools POSIX paths
  (`cygpath -u`) and `angband.exe` Windows ones (`-m`/`-w`).
- Checked under Xvfb:
  - an identical or edited `borg.txt` gets its list back;
  - a missing named list stops the launcher with exit code 2.
- The class files and plans are not copied by the source launcher: they are
  folded into `borg.txt` and shipped for reading only (unchanged).
