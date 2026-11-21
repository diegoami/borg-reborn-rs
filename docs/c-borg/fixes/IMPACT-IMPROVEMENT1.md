# Improvement I-001 impact record

Unit: attack a lone adjacent monster one blow would kill
([#324](https://github.com/diegoami/better-borg-c/issues/324), contract and
evidence there; ledger `patches/improvements.json`; merged 8a0c1ba09, PR #394).
Local-only improvement, never proposed upstream (charter, PR #389).

## Contract and acceptance

Contract named candidate, change, budget and acceptance before implementation
(2026-11-09). All four items closed 2026-11-09:

1. Hosted gate: 965/965 + 5/5, tooling 136/136.
2. Targeted evidence: branch run (seed 1907608237, `--random-birth`);
   `start.save` sha256 equals the recorded D0012 dossier start (`f76185e5...`);
   Borg notes identical through turn 5987; first divergence at turn 6017
   (improvement firing - the skip removes the retreat-scan RNG consumption;
   `borg_thrust_damage_one` verified pure); `KILL:the grey mold` at 8576 (x2)
   and 36413; outcome `turn_cap` alive and well at 320,000 turns versus the
   recorded death by poison at 27,770 (final DL2, CL15, HP 196, `dead=0`).
3. Regression matrix: 225/225 cases pass, zero failed groups, zero row changes
   - no declarations needed; the guard never fires in any fixture.
4. Bounded paired cohort (cohort-v1, Human Warrior, seeds 11/29/47, 20k turns,
   fixed birth): old `37386feca` vs new `8a0c1ba09` - 6/6 runs alive at cap, no
   survival regression; seed 11 deeper progress (depth 5 / max 6 versus 4 / 5)
   at identical HP 72 and CL 7; seeds 29 and 47 identical finals.

## Scope and honesty notes

- 3 paired seeds and one targeted life are directional evidence, not
  population proof.
- The targeted life diverges at turn 6017, so the recorded 27516 loop never
  occurs in it - the improvement engaging earlier is the intended behaviour.
- Toolchain: verify-replay spins on legitimately diverging lives (#392, owner
  hand-off); the `.worktrees` symlink accidentally committed by #389 broke
  `measure-native prepare` and was removed (#393). The regression matrix
  requires the plain product build (no `--observe`) with a full build
  including `OurUnitTestLib`.
