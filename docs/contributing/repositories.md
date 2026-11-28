# The related repositories

*Part of the [contributor guide](README.md).*

## Read-only, except where stated

Other repositories are **read only**, at the commits pinned here (RULES 7).
- **Issues.** A session may open issues on borg-zoo, borg-rs, better-borg-c
  and neo-angband-tracker when this repository needs a change there.
- **Upstream.** Nothing is ever posted to angband/angband (RULES 8).
- **Pins.** The pins below were current when this page was written.
  [`pins.json`](../../pins.json), `recorder-pin.json` and
  `death-tools-pin.txt` are the authority.

| Repository | What it is | How we use it | Pin |
| --- | --- | --- | --- |
| [angband/angband](https://github.com/angband/angband) | the game, and the original C Borg in `src/borg/` | `engine/angband/` is its tree, byte for byte, never edited (RULES 9); our hooks are patches applied at build time ([docs/ENGINE.md](../ENGINE.md)). Its C Borg is CBorg. | `574b618c` |
| [borg-rs](https://github.com/diegoami/borg-rs) | the faithful Rust port of the C Borg | its decision core is vendored as `borg-core/`, with our hooks ([docs/architecture/borg-core.md](../architecture/borg-core.md)); `scripts/borg-core-vendor.py check` compares it | `8c5d000a` (`borg-core/PIN`) |
| [better-borg-c](https://github.com/diegoami/better-borg-c) | the C Borg with verified bug fixes | the C ancestor the architecture documents cite; its verified fixes are in [docs/c-borg/fixes/](../c-borg/fixes/README.md) | `46037038` |
| [borg-zoo](https://github.com/diegoami/borg-zoo) ([site](https://diegoami.github.io/borg-zoo/)) | measures Borgs on frozen seed suites | every comparison ([measuring.md](measuring.md)); its pages are copied in `reference/` | `7b373a70`; adapter in `harness/zoo-adapter-pin.txt` |
| [borg-cemetery](https://github.com/diegoami/borg-cemetery) | the registry of every recorded death, its investigations, scenarios and trial reports | runs export deaths there; the death tools in `tools/` are its own, checked against the pin ([tools/README.md](../../tools/README.md)) | `c102319d` (`death-tools-pin.txt`) |
| [player-cemetery](https://github.com/diegoami/player-cemetery) | original player and mixed recordings, working saves | inputs a Cemetery case links to; analysis is not duplicated there | not pinned |
| [angband-recorder](https://github.com/diegoami/angband-recorder) | the optional game recorder | `vendor/angband-recorder`, built only with `-DSUPPORT_BORG_RECORDING=ON` ([docs/ANGBAND-RECORDER.md](../ANGBAND-RECORDER.md)) | `v0.3.1`, `88c9324b` (`recorder-pin.json`) |
| [borg-runner](https://github.com/diegoami/borg-runner) | language-model agents playing from the screen | what they learn is a source of game knowledge, copied in `reference/borg-runner/` | `e55ac6f3` |
| [neo-angband-tracker](https://github.com/diegoami/neo-angband-tracker) | where the C Borg specification, the Strategy Manual and the C Borg defect catalogue were written | they were moved here on 2026-11-21: [docs/strategy/](../strategy/README.md), [docs/c-borg/](../c-borg/README.md) ([docs/DOCUMENTATION-MAP.md](../DOCUMENTATION-MAP.md)) | `c2ddf2ca` |
| [borg-agent](https://github.com/diegoami/borg-agent) | a former agent player | retired on 2026-11-11; its knowledge is in [docs/research/borg-agent/](../research/borg-agent/knowledge/README.md) | |

Each repository has its own README for contributors there.
[docs/SOURCES.md](../SOURCES.md) records what each source taught this
project (S1–S15).

## Checkouts in `vendor/`

`scripts/setup.sh` clones each pinned repository into `vendor/`, which git
ignores. To read a file at its pin, whatever the checkout:

```bash
git -C vendor/borg-rs show 8c5d000a:README.md
```

## Keeping the sources current

Every source moves (RULES 21). To see what moved since each pin, in commits,
new death cases, new tags and changed C Borg functions we cite, run this
read-only report:

```bash
python3 scripts/refresh-sources.py
```

`--update` refreshes the pins, the copies in `reference/` and the death
index, in their own PR. The copies in `reference/` are never edited here;
`scripts/check-reference.py` checks that they are their source's bytes at
the pin ([reference/README.md](../../reference/README.md)).
[docs/research/KNOWLEDGE.md](../research/KNOWLEDGE.md) records what was read
from the moving sources and what it means for the work.
