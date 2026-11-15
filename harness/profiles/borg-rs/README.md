# Shared Borg configuration examples (proposal)

[PR #308](https://github.com/diegoami/borg-reborn-rs/pull/308) proposes plain
level-condition phases directly in borg.txt:

```text
borg_plan = <<EOF
  level <= 5
    loop-level 0 2
  5 < level <= 10
    loop-level 0 4
  10 < level <= 15
    loop-level 2 6
EOF
```

Not YAML/JSON/TOML. Shared class/progression profiles can live here as
`NAME.borg.txt`: ordinary settings plus this plan. Select/copy the desired
configuration into the run's user/borg.txt; no directory search or external
plan loader. Watched games use `.angband-reborn/user/borg.txt`; guided games
use `SESSION/user/borg.txt` from their manifest.

The parser/runner are not implemented today. See [the proposal](../../../docs/GAME-PLAN-LANGUAGE.md)
for boundary overlap checks, interrupting active errands and ladder migration.
Current parameters still load once at initialization; see
[CALIBRATION.md](../../../docs/CALIBRATION.md). Original-core
`reborn-default.txt`/`BORG_PROFILE` and zoo JSON keep their meaning.
