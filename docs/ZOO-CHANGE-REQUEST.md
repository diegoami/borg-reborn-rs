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
  cell, of every adapter: the better-borg-c observer is exact; borg-rs
  without the flag, Neo-original (it turns an after-cap death into
  `turn_cap`) and Mandos (it stops at the first loop boundary at or after the
  cap) are after-key. `tools/endpoint.py` here already reads that field,
  assumes these values for cells without it, refuses any other kind without
  it, and refuses after-key survivors unless told otherwise.
- **Keep death evidence.** The adapter deletes `keys.log` after a run, so a
  natural death in a zoo cell leaves nothing to record. Keep the key log
  (and the summary) of every `death` cell, for example as a workflow
  artifact, so the measured repository can record the death (its RULES 12).
  Until then, borg-reborn-rs's `harness/zoo-check.py` reruns each death
  through `harness/run.sh` and checks its keys against the cell's play hash.
  The smoke death above (rust, seed 29, Grip at turn 666) is recorded this
  way as D0096.
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

## Appendix: prompt for a borg-zoo session

Given to the owner on 2026-11-09 to start the work in borg-zoo. It follows
borg-zoo's own rules: catalogue changes by PR, read-only pins, append-only
results, and the owner starting large experiments.

````text
You are working in diegoami/borg-zoo. Read AGENTS.md, RULES.md and docs/DESIGN.md first, and follow them
(catalogue changes by PR; other repositories read only, at pinned commits; append-only results; no AI
attribution trailers; local heavy runs under `flock /tmp/angband-validation.lock`; large experiments are
started by the owner, not by you).

## Goal

Make borg-zoo able to measure diegoami/borg-reborn-rs, and record runs at the exact turn boundary,
as requested in borg-reborn-rs `docs/ZOO-CHANGE-REQUEST.md`. Read that file at the commit below, together
with `docs/GOALS.md` (section "Observation and scoring rule"), `harness/README.md`, `tools/depth-stream.py`
and `tools/endpoint.py`.

borg-reborn-rs commit to use: the commit that tag `m1` points to (the M1 review merge). `m1` is an
annotated tag, so plain `ls-remote --tags ... m1` prints the tag object, not the commit; peel it:
`git ls-remote https://github.com/diegoami/borg-reborn-rs 'refs/tags/m1^{}'`, or, in a clone,
`git rev-parse 'm1^{commit}'`. Check that the hash is a commit (`git cat-file -t HASH` prints
`commit`). If the tag does not exist yet, stop and ask the owner for the exact commit to use; do not
take the head of a branch, which moves. Write that full commit hash in the catalogue and in the PR.

## What already works (verify, do not assume)

borg-reborn-rs is a borg-rs-style tree: `harness/build.sh` with `BORG_IMPL=<impl>` builds `build/<impl>/`
behind the same borgh frontend, so zoo/borgrs.py builds and runs it unchanged. Three implementations matter:
- `rust`: the reborn Borg (a placeholder strategy until its M3; its results mean nothing yet);
- `c-pristine`: the original C Borg (upstream e473720 src/borg) on borg-reborn-rs's engine tree;
  byte-identical to the zoo's `4.2.6-rs-m0-c` on borg-rs's smoke (3/3) and full (45/45) suites;
- `c-bbc-m7`: better-borg-c tag 4.2.6-impact-v7's src/borg on the same tree.
The controls fetch their pinned C sources on demand (one batched git fetch), so a fresh checkout builds them.
borg-reborn-rs's `harness/zoo-check.py` ran this zoo's adapter (at 3f6b76e) on all three with no
harness failure or config error. Reproduce that with your real catalogue entries before changing code.

## Tasks (one PR each, or one PR with separate commits; say which)

1. **Catalogue specimens** (catalogue/specimens.json), kind "borg-rs", repository "diegoami/borg-reborn-rs",
   ref = the commit above:
   - `4.2.6-reborn-rust` (impl rust, carries 0)
   - `4.2.6-reborn-c-pristine` (impl c-pristine, carries 0; label: original C Borg built by borg-reborn-rs
     under borgh, byte-identical to 4.2.6-rs-m0-c on borg-rs's smoke and full suites)
   - `4.2.6-reborn-c-bbc-m7` (impl c-bbc-m7, carries 76; label: better-borg-c milestone 7's C Borg under borgh)
   Check that the adapter's build accepts `impl` values other than c/rust everywhere (it passes BORG_IMPL
   through and reads build/<impl>/BUILD_ID); fix any place that assumes only c/rust.

2. **Exact boundary, opt-in per specimen.** A specimen field `"exact_turns": true` makes zoo/borgrs.py pass
   `-exact-turns` to borgh (present in borg-rs since e5cb263 and in borg-reborn-rs). The run then stops at
   engine turn = cap. Set it on the three reborn specimens only; existing borg-rs specimens keep their meaning.
   A specimen whose binary lacks the option must fail loudly, not silently fall back.

3. **Record the boundary rule.** Every cell of every adapter gets `"boundary": "exact"` or `"after-key"`
   (better-borg-c observer cells: "exact"; borg-rs cells without the flag, Neo-original and Mandos:
   "after-key"). borg-reborn-rs's tools/endpoint.py already reads this field.

3b. **Keep death evidence.** zoo/borgrs.py deletes keys.log; keep the key log and summary of every
   `death` cell (e.g. a workflow artifact) so the measured repository can record the death.

4. **Depth stream.** Pass `-statelog FILE` and fill `first_turn_at_depth` from it: each line starts
   `<game turn> <x>,<y> d<depth>/<max depth>`; for each depth d, the turn of the first line whose max depth
   is >= d; depths reached only in the final state take the stop turn (same rule as borg-reborn-rs
   tools/depth-stream.py). Do not keep the state log in results (size); keep only the derived field.

5. **Statistics (optional, ask the owner first if unsure).** Today zoo/stats.py computes goal_rate over
   death/turn_cap/completion cells only, per arm. borg-reborn-rs's primary endpoint counts every attempted
   run, per class, at the exact boundary. Either add that as a clearly named extra statistic (do not change
   the meaning of goal_rate), or document that tools/endpoint.py computes it from the cells.

## Checks before you open the PR

- Unit tests for each change (`python3 -m unittest discover -s tests`), including: exact flag passed only
  when the specimen asks; the boundary field on every cell kind; the depth-stream parser on a synthetic
  state log; an unknown impl still builds.
- A local smoke experiment on the three new specimens (3 seeds, under the lock): no harness failure or config
  error; survivors with `exact_turns` stop at exactly the cap; `first_turn_at_depth` is filled; the
  c-pristine cells' play hashes equal those of `4.2.6-rs-m0-c` run with the same flags (pairing check).
- Existing experiments' plans are unchanged (`python3 -m zoo plan` on each experiment file).

## Do not

- Start a classes-1800 or larger run, or any GitHub Actions experiment: propose the experiment file and
  leave the start to the owner.
- Change the borg-rs or better-borg-c specimens' behaviour, or rewrite existing results.
- Open issues or PRs in other repositories; report anything they need in your PR.

In the PR say what changed, what was verified, what was not, and the open questions for the owner.
````
