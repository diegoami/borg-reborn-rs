# Shared borg.txt examples (proposal)

[PR #308](https://github.com/diegoami/borg-reborn-rs/pull/308) proposes keeping
profiles and errand plans **inside borg.txt**, as a literal YAML block:

```text
borg_reborn_plan = <<BORG_PLAN
version: 1
profiles:
  normal: {parameters: {}}
plans:
  start: [town, shopping]
BORG_PLAN
```

Reviewed examples/fragments can live here as `NAME.borg.txt`. Copy the desired
block/settings into the run's actual `ANGBAND_DIR_USER/borg.txt`; no runtime
profile-directory search, external TOML file or separate plan-file setting is
proposed. A normal watched game uses `.angband-reborn/user/borg.txt`; a guided
game uses its own `SESSION/user/borg.txt` from the manifest. The block reader
must skip all embedded lines when reading flat Borg settings.

See [the proposal](../../../docs/GAME-PLAN-LANGUAGE.md) for profiles, class
ladder plans, precedence and migration. The embedded-block parser, runner and
live switching are **not implemented**. Today core parameters still use flat
`borg_param_<name> = INTEGER` settings read at initialization; see
[CALIBRATION.md](../../../docs/CALIBRATION.md). Existing original-core
`reborn-default.txt`/`BORG_PROFILE` and zoo JSON interfaces keep their meaning.
