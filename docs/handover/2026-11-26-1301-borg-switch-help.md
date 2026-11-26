# List Borg switching in Ctrl-Z help

The shared Ctrl-Z prompt now points to help. Pressing `?` lists the
uppercase `B` switch in PBorg and CBorg, including PBorg's fallback reborn
core. The same command works directly from the Ctrl-Z prompt. Switching
changes the session's selection, and `z` starts the selected Borg.

A native headless build exercised both help screens with scripted keys.
With both PBorg cores, the script switched PBorg to CBorg and back from
help before starting PBorg. The existing Borg selection and settings
tests passed.
