## 2026-11-20: attack categories and winning fights (#760, #759)

Owner-requested combined implementation on `task/T760-fight-fragile`, from
main `08f94e1f`. #757 and #761 belong to another session and are untouched.

- `attack_on_sight` and `caution_winning_fight` are compiled/shipped 0.
- Attack entries extend the existing scary/ignore database with a required
  reason, class/level limits and independent replacement/removal. The
  initial fast/fragile and breeder entries cite engine data; breeder
  entries preserve the existing stoppable/lost-colony policy.
- Synchronized accepted main `ec0f3b28` (#755); preserved its leash changes
  and resolved only the registry/citation overlap.
- A focused attack planner separates damage from the native utility score.
  The general fight race uses remaining HP once, hit/failure chances,
  finite resources, shared mana and speed-scaled incoming damage. Ties do
  not win. Exact observed HP/max HP is retained at the existing perception
  read; health-bar rounding is not used to certify a win. Zero-HP monsters
  still need one damage point. Finishing damage is capped before hit chance,
  fractional arrow averages are retained, and regeneration is charged.
  Own computed fractional blows/shots and displayed speed drive the clock;
  ammo limits count individual uses, not whole turns of fire. The
  fight rung precedes depth-return/light/countdown flight rungs. Unknown
  estimates fall through to ordinary caution.
- The category proposal was presented before building. The six specified
  baseline deaths reproduced on clean `08f94e1f`, with exactly the supplied
  control key streams, before implementation.
- CALIBRATION describes the hypotheses and conservative modelling bounds.
  No population improvement is claimed.

## Migration / resume state (owner request, 2026-11-20)

The owner requested moving this session to another computer, with the same
absolute paths. All three of this session's pending lock requests were
cancelled while still queued; no own validation game was active. Other
sessions' jobs were left untouched. Do not try to resume old tool-session IDs.

**Implementation code pin:** `28670e85b691d72e507f7e516be4ab19793aa0f6`.
[PR #763](https://github.com/diegoami/borg-reborn-rs/pull/763) is a draft;
[CI 44758141143](https://github.com/diegoami/borg-reborn-rs/actions/runs/44758141143)
is green at that code pin, including deterministic/reference borg-rs smoke,
replay, Rust, recorder and engine checks. Later handover commits change
only documentation. This note supersedes earlier statements that default
neutrality or CI still needed checking.

**No enabled local games or 600-start candidate cohorts have run.** Only
the clean baseline six were played: every endpoint and key/state/message
stream matches the supplied controls and canonical Cemetery records.
No enabled survival improvement has been measured. Keep the PR a draft.

The original task remains #760, then #759, then separate #758. All new
behavior stays compiled/shipped 0, every threshold is named/cited, and
`check-param-citations.py` must report 0 wrong. Preserve #757/#761, owned
by another session. The owner has not named a reviewer or authorized merges.
Each source PR ends with a reviewer checklist pinned to its exact head,
instructing the owner-selected reviewer to post a PR comment.

### Concrete remaining work

1. Read the implementation and the related #758 note before continuing.
   Inspect two **identified, not yet fixed** model gaps before the large
   cohort:
   - Native `kill.speed` stores base `m.mspeed`. `mon-move.c:1929-1934`
     adds temporary Haste and subtracts graded Slow. The new fight/Recall
     model currently uses base monster speed: it must account for observed
     Haste to avoid optimistic wins; handle Slow conservatively or correctly.
     Preserve legacy off behavior. These timers already arrive in the
     permitted MonsterView observation. None of the six baseline logs
     contains a visible “starts moving faster” event; that is not a proof
     that every possible encounter has no haste.
   - The focused executor bypasses native `borg_attack`'s
     `revert_fox_to_fight` before melee. A fox's natural attack is not its
     equipped weapon damage. Model availability/revert cost or decline
     unavailable weapon attacks; do not certify wins using the wrong form.
     This matters for Druids in the wider set, not the six Ranger cases.
   These are implementation findings, not owner-approved policy decisions.
2. Build and play the six cases with each switch separately and together;
   check estimates on won fights too. If case 425904011 is not judged
   winning at its finishing opportunity, inspect actual HP/attack timing
   and the player death threshold (also below zero), rather than assuming
   the issue's approximate health-bar narrative is exact. Retain all
   attempts, including failures or unsuccessful intermediate variants.
3. Run the 600 specified starts, paired by seed/race/class, comparing the
   supplied off controls with fights-on and all-three-on. The provided
   controls have 12 deaths and 0 recorded process failures. All common
   configuration settings match our current profile across all 600 files;
   runtime birth/core/errand keys are overridden identically by the adapter.
   Fresh off six + CI establish tested neutrality; if a fresh off case
   diverges, investigate before reusing that control. Report saved and
   newly killed starts per class, all failures, and progress tradeoffs.
4. Publish enabled cases/cohort evidence in Cemetery; update CALIBRATION,
   this note, the #758 note, PR bodies and exact reviewer pins. No merge.

The six starts (seed, race index, class index) are:

```text
425904011 2 7
2095979189 6 7
1976035382 8 7
2049259432 4 7
807683971 0 7
1517990203 4 7
```

### State outside Git

Transfer `/tmp/T760-fight/` intact. It contains the original base executable
and BUILD_ID, full baseline six streams, reconstructed base profile, start
lists, private batch/reproduction/comparison scripts, the supplied off600
manifest and the pinned zoo census. Scripts use versioned immutable runtime
and harness snapshots, refuse to overwrite trials, validate requested
parameters, retain failures and use ten workers. They must be run under
`flock /tmp/angband-validation.lock`. Their expected-head checks must match
the worktree head used for the build; repin after implementation changes.
No runtime snapshot for the enabled versions exists yet.

Original evidence is read-only:
`/tmp/claude-1000/-home-diego-projects-borg-reborn-rs/363b8e4f-0f25-42e8-8383-d41ed8b07227/scratchpad`.
Keep its `r200.txt`, `c400.txt`, `r200.all`, `c400.all`, `zrun.py`,
`flight-deaths/README.md`, and every `r200/off-*` / `c400/off-*` run's
keys/messages/states/summary, user profile, parameter dump and death map.
The original large `trace-core.jsonl` files are optional for transfer;
new runs generate their own traces. Old flight-on arms and unrelated
scratchpad files are not needed to resume this task.

### Canonical evidence work

[borg-cemetery PR #88](https://github.com/diegoami/borg-cemetery/pull/88),
branch `task/T760-flight-evidence`, owns
`studies/reborn-flight-fights-20261120/`. It contains the six full-stream
reproduction proofs, the 600 starts, control/configuration audit and a pinned
zoo census: 8,953 distinct reborn death outcomes at results commit
`a31a57e985517b6b46e1aeea9e8a29c7b5fb6119`. Counts are investigative leads,
not mortality rates. The study explicitly says enabled validation is pending.

The working evidence checkout is `/tmp/T760-cemetery`; its branch is pushed,
so it can be freshly cloned there instead of copying its shared Git object
alternates. Three supplied control deaths were not yet retained after
synchronizing Cemetery main: Priest 199095980 (4/3), Necromancer 122900422
(2/4), Paladin 238844665 (0/5). A private replay input list and frozen clean
08f94e1f runtime are ready; reproduce, compare hashes and export only genuinely
new start/key streams. Existing baseline Ranger records must not be duplicated.
Run Cemetery `scripts/death_ids.py --check` before delivery. Preserve all
published hashes/history. No Cemetery or source PR has been merged.
