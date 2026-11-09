# Change request for borg-zoo

*From task T1.8 (#11), 2026-11-09. For the owner to route to
[diegoami/borg-zoo](https://github.com/diegoami/borg-zoo) (RULES 7: nothing
is filed in other repositories from here). Read against borg-zoo `3f6b76e`
(the pin in `pins.json`).*

## What already works, unchanged

`harness/zoo-check.py` loads the pinned zoo and adds three specimens in
memory, then calls the zoo's own `harness.build` and `harness.run_cell`, as a
workflow would. The specimens are kind `borg-rs`, repository
`diegoami/borg-reborn-rs`, ref = this checkout's HEAD. On the zoo's `smoke`
suite all three arms run with **no harness failure and no config error**:

| Specimen (in memory) | Seed 11 | Seed 29 | Seed 47 |
| --- | --- | --- | --- |
| `rust` | turn_cap, max depth 1, turn 20009 | death at turn 666 | turn_cap, max depth 1, turn 20009 |
| `c-pristine` | turn_cap, depth 1, turn 20003 | turn_cap, depth 3, turn 20003 | turn_cap, depth 2, turn 20003 |
| `c-bbc-m7` | turn_cap, depth 2, turn 20004 | turn_cap, depth 2, turn 20004 | turn_cap, depth 2, turn 20000 |

The adapter passes `BORG_IMPL` through and reads `build/<impl>/`, so the
controls need no adapter code. A fresh checkout builds them: the pinned C Borg
sources are cloned on demand, in one batched fetch.

The table also shows what is missing: survivors stop **after** the cap
(20009, 20003, 20004), and `first_turn_at_depth` is empty.

## Requested changes

### 1. Catalogue: three specimens (small, no code)

```json
"4.2.6-reborn-rust":       {"kind": "borg-rs", "impl": "rust",       "repository": "diegoami/borg-reborn-rs", "ref": "<tag m1 commit>", "carries": 0,
                            "label": "borg-reborn-rs M1: the reborn Borg (a placeholder until M3)"},
"4.2.6-reborn-c-pristine": {"kind": "borg-rs", "impl": "c-pristine", "repository": "diegoami/borg-reborn-rs", "ref": "<tag m1 commit>", "carries": 0,
                            "label": "Original C Borg (e473720) built by borg-reborn-rs under borgh; byte-identical to 4.2.6-rs-m0-c on borg-rs's smoke and full suites"},
"4.2.6-reborn-c-bbc-m7":   {"kind": "borg-rs", "impl": "c-bbc-m7",   "repository": "diegoami/borg-reborn-rs", "ref": "<tag m1 commit>", "carries": 76,
                            "label": "better-borg-c milestone 7's C Borg (4.2.6-impact-v7 src/borg) built by borg-reborn-rs under borgh"}
```

`<tag m1 commit>` is the commit of the M1 review merge. These three are the
arms of every comparison in GOALS D8. `4.2.6-reborn-c-pristine` plays the same
keys as `4.2.6-rs-m0-c` on borg-rs's suites, so the two can be paired.

### 2. Adapter (`zoo/borgrs.py`): the exact boundary and the depth stream

Needed before G1 (GOALS, *Observation and scoring rule*):

- **Exact boundary.** Pass `-exact-turns` to borgh. Every borg-rs build at or
  after borg-rs `e5cb263` has it, and so does every borg-reborn-rs build. The
  run then stops at engine turn T itself. Keep it opt-in per specimen
  (e.g. `"exact_turns": true`), so existing borg-rs cells keep their meaning.
- **Record the rule.** Add `"boundary": "exact"` or `"after-key"` to every
  cell. `tools/endpoint.py` here already reads that field, and refuses
  after-key survivors unless told otherwise.
- **Depth stream.** Pass `-statelog FILE` and fill `first_turn_at_depth`
  from it. Each line starts `<game turn> <x>,<y> d<depth>/<max depth>`; the
  first line at which the maximum depth reaches d gives the turn for d. This
  is the rule of `tools/depth-stream.py`, about 20 lines.

### 3. Statistics (optional)

`zoo/stats.py` computes `goal_rate` over death, turn_cap and completion
cells only, and reports it per arm. The GOALS endpoint counts **every
attempted run**, per class. Either add that as an option, or leave it:
`tools/endpoint.py --zoo ARM=cells.jsonl:SPECIMEN` computes it from the zoo's
cells. It was checked on `classes-milestones`: 3,600 cells, harness failures
kept in the denominator.

## Not requested

- No change to how better-borg-c observer specimens run.
- No new suites. The class suites (`classes-1800`, `classes-1800-b`) and the
  deep suite are what G1–G3 use.
