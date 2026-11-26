## 2026-11-18: class values for the core's borg.txt settings (#520, second half)

#534 gave every `borg_param_*` a value per class. This does the same for the
core's own `borg.txt` settings (`borg_*`, `BORG_SETTINGS`), such as the
shopping weights `borg_town_w_*`, which a class file could not override
before: folded into the one `borg.txt`, its line applied to every class.

- **The line:** `borg_<name>@<class> = <value>` (`borg_init.rs`,
  `class_setting_line`). It is kept apart while the file is read, so it never
  changes the common value. A `borg_param_*` class line stays params.rs's.
- **When it applies:** after the file is read (and after the range checks of
  `finish_txt_settings`), each setting with class lines remembers its common
  value. `borg_prepare_race_class_info` (birth and every reincarnation) and
  the end of `borg_init_txt_file` (a `$` reload) then give it the current
  class's value, or the common one. Settings without class lines are
  untouched, so runtime values such as the money-scum amount survive.
- **Refused, with a Borg warning:** an unknown class or setting, a retired
  or `borg_respawn_*` setting, or a value that is not exactly an integer (or
  TRUE/FALSE/YES/NO/T/F/Y/N/1/0 for a true/false setting). Names and classes
  in any case.
- **Checked like the common line** (review of #598): the range checks and
  configuration errors are one function, `check_settings`. Each class line is
  tried on top of the common values, then each class's lines together (two
  class masks can conflict). A value the checks would change (a clamp, such as
  `borg_enchant_limit` 8..15) or reject (`borg_town_plan` 7, a buying-strategy
  mask) refuses the line and stops the Borg (`init_failure`), as an invalid
  common line does. A class value is judged on its own (re-review of #598):
  against the common values with its setting put back to the compiled
  default, so an error the common values have elsewhere is not blamed on it,
  and the same error in that setting's common value does not hide its own.
  If a class's lines fail together, all of that class's lines are dropped.
- **Two kinds of refusal:** a line with a bad *value* (or unknown name or
  class) is dropped with a warning and the common value is used; a line whose
  value the *checks* reject also stops the Borg. `scripts/borg-class-files.py`
  checks values the same strict way (by the setting's kind in
  `BORG_SETTINGS`), so a bad value is caught in the class file.
- **A runtime value is reset by a class line:** a setting with a class line
  gets its class or common value again at a reincarnation. Today the only
  setting written at runtime is `borg_money_scum_amount`; without a class line
  it keeps its runtime value.
- **The class files:** `scripts/borg-class-files.py` now accepts `borg_*`
  lines and folds them as `borg_<name>@<class>`. It checks each name against
  `BORG_SETTINGS`, and refuses the shell's keys (`borg_reborn_*`,
  `borg_plan`, `borg_scary_list`) and `borg_respawn_*`.
- **Shipped files unchanged:** no class file has a `borg_*` line yet, so
  `harness/borg.txt` is byte for byte the same and every game plays as on
  main.
- **docs/ZOO-INTERFACE.md**, *Class values*: the new lines, and that a zoo
  profile cannot write one (`BORG_TXT_SET` names have no `@`).

Left for the owner: which `borg_*` values (e.g. the healing and Phase Door
weights of #520's evaluation) go into which class file, measured as a zoo arm
first. No environment form (`BORG_*__<CLASS>`) for these settings: the
params.rs one (decision B) covers `borg_param_*` only.
