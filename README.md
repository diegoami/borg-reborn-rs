# borg-reborn-rs

The Angband Borg reborn, in Rust: an automatic player for Angband, redesigned
from the ground up in how it **plays**. It stands on its own: the game is
upstream Angband 4.2.6 at `e473720ed88a0f0c7ca66815cef3000eb4a38bc3`, byte for
byte, behind upstream's own Borg seam (`do_cmd_borg`, `inkey_hack`; started
with **Ctrl-Z**, **z**), and the integration around it (a small patch series of
harness hooks, the headless frontend, the bindings, the build, the harness) is
ours ([docs/SPEC.md](docs/SPEC.md)). [borg-zoo](https://diegoami.github.io/borg-zoo/)
measures it and compares it with other Borgs.

**Status (2026-11-10):** M1 is reviewed (tag `m1`); M2 has its Observation,
world model, knowledge tables, decision trace and parameters (T2.1–T2.4);
milestone **E, our own integration** (docs/SPEC.md §10), is merged (tag `e`)
and its review's findings are being fixed. M2 resumes with T2.5. The
Rust Borg plays, but only as a placeholder until M3. Tasks are GitHub issues
([milestones](https://github.com/diegoami/borg-reborn-rs/milestones)).

## Playing

```sh
./watch-borg.sh [NAME]               # build (X11) and open the game; create/load a character, then Ctrl-Z, z
BORG_IMPL=rust harness/build.sh      # the headless build (the zoo's entry point)
harness/run.sh OUT SEED RACE CLASS TURNS menu   # one deterministic headless run
tools/rerun-death.py docs/deaths/DXXXX-... OUT  # rerun a recorded death with the Borg's logs
```

Saves and settings of `watch-borg.sh` are in `.angband-reborn/`. Until M3
the Borg is a placeholder (it walks, rests and descends; see
[docs/HANDOVER.md](docs/HANDOVER.md)). [tools/README.md](tools/README.md)
documents the death records.

Working on this repository: start at [AGENTS.md](AGENTS.md), then
[RULES.md](RULES.md). It is model agnostic: any coding agent, or a person,
follows the same files.

## Related repositories

| Repository | Role here |
| --- | --- |
| [angband/angband](https://github.com/angband/angband) | The game and the original C Borg (`src/borg/`), at the pin above. Read only. |
| [neo-angband-tracker](https://github.com/diegoami/neo-angband-tracker) | The C Borg specification and the Tunable Borg Plan (`docs/c-borg/`); the issue board for C Borg defects. |
| [better-borg-c](https://github.com/diegoami/better-borg-c) | The C Borg with verified bug fixes; native measurement runner; death dossiers. |
| [borg-rs](https://github.com/diegoami/borg-rs) | The faithful Rust port. A reference to copy from, never a dependency (docs/SPEC.md §2); what was copied is ours and listed in docs/PROVENANCE.md. |
| [borg-zoo](https://github.com/diegoami/borg-zoo) | Measures Borg versions on frozen seed suites. Our yardstick. |
| [borg-agent](https://github.com/diegoami/borg-agent) | A language-model player that writes down game knowledge with evidence, meant as an input to this Borg. |
| [borg-runner](https://github.com/diegoami/borg-runner) | Language-model agents play the game from the screen and record every key; what they learn accumulates in one page (`manuals/INSTRUCTIONS.md`). A source of game knowledge (S15). |

Licence: GPLv2 (see `LICENSE`), as borg-rs. Angband itself is dual-licensed
GPLv2 / Angband licence; files copied from other projects keep their
headers and are listed with their provenance.
