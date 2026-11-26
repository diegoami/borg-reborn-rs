# Combined CBorg and PBorg build

Issue #1288. Ctrl-Z opens the selected Borg's command prompt. Each game
process starts with PBorg selected. At that prompt, B switches Borgs and
Ctrl-Z then z starts the selected Borg. The headless default takes the same
path. Its `-cborg` option exercises the switch for native verification.

PBorg reads `user/pborg.yaml`. CBorg reads `user/borg.txt`. The shipped
settings and class overrides still use the legacy source template for the
Zoo's profile machinery and historical hashes. Launchers convert the
resolved settings to YAML before play. Existing PBorg installs can use
`harness/pborg-config.py` to import their old settings without modifying the
original file. The independent plan and scary-list data formats are
selected by keys in the YAML.

The combined-build regression runs the same executable with conflicting
stop depths and verifies that each Borg follows its own file. The new
`pborg_config=yaml` build identity is covered by the companion Zoo adapter
work in diegoami/borg-zoo#810. The existing native parameter receipt uses
its historical origin labels for compatibility.
