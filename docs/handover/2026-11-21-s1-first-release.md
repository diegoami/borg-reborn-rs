## 2026-11-21: P7 in progress; milestone S1, the first playable release

The owner accepted P7 ("accept it, merge to main", PR #799), and asked: "set up git actions bundling angband together with the born again borg … and create releases … only on demand when we reach a milestone. And define a milestone that we can ship before finishing track P."

**What was done:**
- PLAN.md: P7 marked in progress.
- A new "Releases" section. First **S0, a preview prerelease** (`v0.1.0-alpha.1`, pre-release on GitHub; owner: "we are still very buggy, S1 is too optimistic"), with a short `PLAYING.md` and the known problems in its notes. Then **S1, the first playable release** (`v0.1.0`), with the full user guide and the zoo's measurement.
- Its work: the release workflow (#819, on demand only: `workflow_dispatch` or a milestone tag), the user guide (#817) and the documentation split (#816).

**For the owner:**
- S1's exit asks for borg-zoo's measurement of the exact release commit, given in the notes as a description, not a claim. Request it before tagging.
- Windows and macOS builds only if our integration builds there; otherwise S1 says Linux only.
- The owner publishes the draft release.
