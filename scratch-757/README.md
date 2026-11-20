# PR #757's local 600-start comparison: inputs (scratch only, never merged)

- `manifest-600.txt`: one start per line, `set seed race_idx class_idx` (birth-menu indices, played with `-menu-letters`): 200 Rangers (`r200`) and 50 starts of every other class (`c400`). 100,000 game turns, `-exact-turns`, the borg-rs core.
- `zrun.py`: plays one start as the zoo adapter does. It writes `OUT/user/borg.txt` from `harness/borg.txt` with `key=value` overrides, then runs the game:

      BORG_IMPL=rust harness/build.sh
      python3 scratch-757/zrun.py build/rust/game harness OUT SEED RACE CLASS 100000 [borg_param_NAME=V ...]

  `OUT` gets `summary.json` (`dead`, `death_cause`, `max_clevel`, `max_depth`), `messages.log`, `state.log`, `keys.log` and `archive/Borg.map`.

| arm | build | overrides | deaths |
|---|---|---|---|
| off | `task/T751-flight` with #743 merged (`ad1dff48`; its play at the shipped settings is main `08f94e1f`'s, smoke identical) | none | 12 |
| on, mode 2 | same | `borg_param_caution_stairs_clean_path=2 borg_param_caution_desperate_fast_danger_percent=50 borg_param_caution_recall_cure_min_loss_percent=30 borg_param_caution_stairs_flight_ignore_danger=1 borg_param_caution_stairs_clean_path_flee=1` | 18 |
| on, mode 3 | `6951bc98` | the same, with `borg_param_caution_stairs_clean_path=3` | 16 |

The `on` arm's deaths in the manifest's sets: mode 2: Rangers 10, others 8; mode 3: Rangers 12, others 4 (Mage 0, Necromancer 0, Paladin 1, Priest 2, Rogue 1). Off: Rangers 9, others 3 (Necromancer, Paladin, Priest 1 each).

`caution_desperate_ranged_first_reach` (`50c23a5e`) has no comparison yet; its measurement is a borg-zoo arm (borg-zoo#501). Main now carries #755; a control built from today's main may differ from these runs.
