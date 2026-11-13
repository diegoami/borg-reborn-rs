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

## Game knowledge

What the Borg knows about the game comes, in this order, from the engine,
from two manuals, and from recorded play. The engine
([`engine/angband/`](engine/angband/), upstream `e473720`) is the only
authority: every formula, table and prompt is checked in `src/` or
`lib/gamedata/` and cited there (RULES 16). The manuals are copied in
[`reference/c-manual/`](reference/c-manual/README.md) (S14): the **Angband
Strategy Manual**, a source-grounded guide to how the game works underneath
and how to survive it (chapters 0–16, appendices A1–A10, and *notes for the
borg* for automatic players), and the **official user manual** for this exact
version (`user-manual/`) for commands, screens and options. The third input
is evidence from play: the death dossiers, borg-runner's accumulated
knowledge and our own decision traces. Nothing is a claim until
[borg-zoo](https://diegoami.github.io/borg-zoo/) measures it.

## Related repositories

| Repository | Role here |
| --- | --- |
| [angband/angband](https://github.com/angband/angband) | The game and the original C Borg (`src/borg/`), at the pin above. Read only. |
| [neo-angband-tracker](https://github.com/diegoami/neo-angband-tracker) | The C Borg specification and the Tunable Borg Plan (`docs/c-borg/`); the Angband Strategy Manual and the official user manual of our game version (`docs/c-manual/`, copied in `reference/c-manual/`); the issue board for C Borg defects. |
| [better-borg-c](https://github.com/diegoami/better-borg-c) | The C Borg with verified bug fixes; native measurement runner; death dossiers. |
| [borg-rs](https://github.com/diegoami/borg-rs) | The faithful Rust port. A reference to copy from, never a dependency (docs/SPEC.md §2); what was copied is ours and listed in docs/PROVENANCE.md. |
| [borg-zoo](https://github.com/diegoami/borg-zoo) | Measures Borg versions on frozen seed suites. Our yardstick. |
| [borg-agent](https://github.com/diegoami/borg-agent) | *Retired 2026-11-11* (a failed experiment): no longer an input. Its roles, model-played games and claims with evidence, moved to secondary goal Q (docs/GOALS.md D12). Its pin goes in the next source-refresh PR. |
| [borg-runner](https://github.com/diegoami/borg-runner) | Language-model agents play the game from the screen and record every key; what they learn accumulates in one page (`manuals/INSTRUCTIONS.md`). A source of game knowledge (S15). |

Licence: GPLv2 (see `LICENSE`), as borg-rs. Angband itself is dual-licensed
GPLv2 / Angband licence; files copied from other projects keep their
headers and are listed with their provenance.
