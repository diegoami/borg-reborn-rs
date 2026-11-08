# C Borg documents

What this fork knows about the original Angband C Borg, and what it plans to
do with that knowledge. Every document here reads the C source at
angband/angband `e473720ed88a0f0c7ca66815cef3000eb4a38bc3`
(`origin/c-borg-reference`), not the 4.2.6 tree under `reference/`.

Live state belongs on the issue boards: which bugs are open, what is carried
where, what is decided. These documents describe and plan; they do not track.

## Documents

| Document | What it is |
| --- | --- |
| [The C Borg: purpose and architecture](BORG_SPEC.md) | The specification: what the C Borg is for and how it is built. It covers perception, the key channel, the dungeon decision ladder, flow, danger, caution, depth preparation, modes and configuration, cited as `file:function:lines`. It does not list bugs. Merged with [PR #317](https://github.com/diegoami/neo-angband-tracker/pull/317). |
| [Tunable Borg Plan](TUNABLE-BORG-PLAN.md) | The evaluation and plan for a Borg that can be tuned and improved by experiment, across better-borg-c and borg-rs: baselines, parameters with identity at defaults, the harness, the objective and statistics, and the decisions left to the owner. Merged with [PR #361](https://github.com/diegoami/neo-angband-tracker/pull/361), after the review in [#360](https://github.com/diegoami/neo-angband-tracker/issues/360). Its section *What a remake leaves out* records what a from-scratch Borg would not carry over (munchkin start and lunal mode, by owner decision), and its *Strategy by class* direction, with borg-zoo measurements. |
| [Tunable Borg Plan review prompt](TUNABLE-BORG-PLAN-REVIEW-PROMPT.md) | The prompt for an independent review of the plan, including a re-review checklist. |

## How the claims were checked

| Document | What it is |
| --- | --- |
| [Source-level check programs](../audit/c-reports/spec-checks/README.md) | Standalone C programs that transcribe Borg code and run it on stated, often synthetic, inputs. They are not game runs. |
| [Re-check record](../audit/c-reports/spec-checks/RECHECK.md) | How the specification and the programs were re-verified after review, with the commands to repeat each check, and the observations left to the owner. |
| [Findings' provenance](../audit/c-reports/spec-checks/PROVENANCE.md) | How the findings #310 to #316 were established. |

## On the issue board

- [C findings and candidates](../C-BUGS.md): the landing page, with links to
  filtered issue lists.
- [Open enhancement issues](https://github.com/diegoami/neo-angband-tracker/issues?q=is%3Aissue+is%3Aopen+label%3Aenhancement):
  the quick gains to try before any redesign, and the experiment toolkit for
  early-game survival.
- Session receipts for the C lane: [checkpoint #207](https://github.com/diegoami/neo-angband-tracker/issues/207).
