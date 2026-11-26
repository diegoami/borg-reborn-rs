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
stop depths and verifies that each Borg follows its own file. A Linux headless package also ran with
its shipped YAML. Three 2,000-turn starts matched the pre-change build's
key streams and resolved native settings byte for byte. The companion
Zoo adapter and Cemetery replay changes are diegoami/borg-zoo#812 and
diegoami/borg-cemetery#161. The existing native parameter receipt uses
its historical origin labels for compatibility.

The four default-borg smoke runs kept identical key and state streams.
The reference refresh records the PBorg settings startup message. Native
death tools are pinned to the Cemetery replay support in #161 so the
harness's PBorg configuration receipt can be verified.

In interactive play, P-Borg shows its full introduction once per process,
then credits Diego Amicabile on subsequent starts and selections. C-Borg
credits Ben Harrison and Andrew P. White. The headless frontend retains
its replay message stream. [Harrison's Angband page](https://www.phial.com/angband/).
[Historical Borg README](https://github.com/angband/angband/blob/3.5.1/src/borg/borgread.txt).

Review follow-up synchronized the combined build contract, restored the
watcher's scary-list preflight through the game YAML parser, and added
the upstream CBorg defaults to release packages. PBorg's introduction
identifies the shipped decision core. An absent PBorg settings file
reports the reborn-core fallback.

The re-review reconciled the remaining live build and command-interface
documentation. The settings helper now removes its temporary configuration
files after both successful and failed conversions.
