# List Borg switching in Ctrl-Z help

The shared Ctrl-Z prompt now points to help. Pressing `?` lists the
uppercase `B` switch in PBorg and CBorg, including PBorg's fallback reborn
core. The same command works directly from the Ctrl-Z prompt. Switching
changes the session's selection, and `z` starts the selected Borg.

`engine/tests/test_borg_selection.py` uses the headless `-borg-help`
sequence to switch from PBorg to CBorg and back through help with each
Rust core. It checks the displayed commands and the confirmation after
the help screen closes, then verifies that the selected core starts.
The fallback help accepts repeated `?` and reports an invalid command.
The packaged README and engine contract describe the same prompts.
