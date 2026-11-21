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
