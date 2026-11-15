# Player Borg profiles (proposal)

[PR #308](https://github.com/diegoami/borg-reborn-rs/pull/308) proposes shared
player profiles here as `NAME.toml`: one file contains parameters, named
errand plans and optional single-level class overrides. Ladder schedules and
other progression policy belong in these profiles, not new hard-coded engine
commands. See [the format and migration plan](../../../docs/GAME-PLAN-LANGUAGE.md).

Personal profiles go in `ANGBAND_DIR_USER/profiles/NAME.toml`; select them
explicitly as `user:NAME`. Shared names are not silently shadowed. For a normal
watched game the personal directory is `.angband-reborn/user/profiles/`; a
guided game uses its own `SESSION/user/profiles/`.

The TOML loader, named plan runner and live switching are **not implemented**.
No `cautious` preset is shipped or claimed to improve play by this proposal.
Today, the vendored core reads `borg_param_<name> = INTEGER` from its run's
`borg.txt` at initialization; the catalogue/bounds are in
[CALIBRATION.md](../../../docs/CALIBRATION.md). Environment overrides win.
Existing `../reborn-default.txt` and `BORG_PROFILE` configure the original
reborn core and keep their current meaning. Zoo profile JSON remains a
separate measurement/configuration interface.
