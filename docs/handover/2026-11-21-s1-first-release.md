## 2026-11-21: P7 in progress; milestone S1, the first playable release

The owner accepted P7 ("accept it, merge to main", PR #799), and asked: "set up git actions bundling angband together with the born again borg … and create releases … only on demand when we reach a milestone. And define a milestone that we can ship before finishing track P."

**What was done:**
- PLAN.md: P7 marked in progress.
- A new "Releases" section with **S1, the first playable release** (`v0.1.0`): the shipped core on main, bundled with Angband, Linux first.
- Its work: the release workflow (#819, on demand only: `workflow_dispatch` or a milestone tag), the user guide (#817) and the documentation split (#816).

**For the owner:**
- S1's exit asks for borg-zoo's measurement of the exact release commit, given in the notes as a description, not a claim. Request it before tagging.
- Windows and macOS builds only if our integration builds there; otherwise S1 says Linux only.
- The owner publishes the draft release.
