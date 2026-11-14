# The option-C pilot: results

*Option C of [REDESIGN.md](REDESIGN.md), chosen by the owner on
2026-11-14. Issues: #207 (C1), #208 (C2), #209 (C3). Measured on
2026-11-14 against borg-rs `4ed3263`.*

## C1: borg-rs's core inside our shell (#207): met

- **The vendored core.** borg-rs's crate is vendored at the pin as
  `borg-core/` (docs/PROVENANCE.md). `BORG_CORE=borg-rs` selects it at run
  time; our own core stays the default.
- **The footprint in borg-rs's code:**
  - its exported `do_cmd_borg` became a plain function that our
    `do_cmd_borg` calls;
  - three test-data paths were repointed;
  - the core reads its own `borg.txt` (`borg-core/borg.txt`).
- **Parity, byte for byte:**
  - Smoke: 3/3 runs identical (keys, messages, state) with borg-rs's own
    build at the pin.
  - Full: 45/45 identical.
  - Our core, the default, is unchanged: smoke and smoke-exact 3/3
    identical with our references.
- **borg-rs's committed references are stale.** They do not match
  borg-rs's own build at the pin; both builds diverge from them at the same
  keys. The oracle is therefore a fresh run of borg-rs's build at the pin
  (`harness/build.sh` in a scratch copy), not its `harness/reference/`.
- **borg-rs's suites have no `menu` flag.** Without it the headless
  frontend creates a different race or class than the suite line asks for
  in 15 of 45 runs (`harness/check-characters.py`). The parity runs keep
  borg-rs's suite as it is, so both sides play the same characters. Every
  measurement after parity uses `menu`.

The same seeds, through `tools/progress.py`:

| full suite, 45 runs | borg-rs (vendored core) | our core (#201) |
|---|---|---|
| alive | 35 | 13 |
| alive and moving | 31 | 9 |
| mean turns frozen | 5,778 | 7,667 |
| mean max depth | 3.53 | 1.67 |
| mean character level | 5.47 | 3.98 |
| reached DL5 | 18 | 0 |

## C2: errands over the core (#208): dive to N, in progress

- **The shell's footprint in borg-rs's code:** one new file,
  `borg-core/src/shell.rs`, plus three hook calls marked
  `borg-reborn-rs (#208)`:
  - `borg_prepared`: depths down to N count as prepared, so the core does
    not climb back up from them;
  - `borg_think`: before the dungeon think, the errand ends at depth N or at
    its budget, through the core's own auto-stop (`borg_oops`);
  - `borg_think_dungeon`: right after `borg_caution`, the errand steers. It
    flows to a known down staircase and takes it, with the core's own
    power-dive steps.
- **How it is called:** `BORG_ERRAND=dive:N`, `BORG_ERRAND_BUDGET`
  (game turns), and the outcome in `BORG_ERRAND_OUT`.
- **With no errand, play is borg-rs's byte for byte** (smoke 3/3).

One character per class (the `-0` line of each class in the full suite,
`menu`), budget 60,000 game turns:

| | done | died | out of budget |
|---|---|---|---|
| `dive:3` | 6 of 9 (4,300–18,200 turns) | 3 | 0 |
| `dive:5` | 6 of 9 (6,800–24,300 turns) | 3 | 0 |

- **The two early deaths, traced.** Without an errand, borg-rs's
  warrior-0 and priest-0 live to the cap; under `dive:3` both die on DL1.
  - **Shopping is not the cause.** Up to the split, both runs make the same
    town choices. Both warriors sell Word of Recall and Berserk Strength
    and buy torches: that is borg-rs's munchkin start, with or without the
    errand.
  - **The split.** Both pairs split at a DL1 level with no known down
    staircase:
    - without the errand, borg-rs says "heading up (bored and unable to
      dive: Clevel < depth)";
    - with it, the readiness override (prepared down to N) switched that
      rule off, and the Borg wandered for 40–70 turns before leaving for
      another reason ("Spent too long on level").
  - **The deaths are borg-rs's own decisions in a game that had become a
    different one.** The warrior fled a kobold archer at 4/20 HP with no
    Phase Door. The priest died confused beside a grey mushroom patch. The
    errand's steering played no part in either.
  - **The fix.** The override now holds only depths at or above the
    current one, so the core does not climb back up from a depth the dive
    reached. The steering does the diving.
- **The sample**, after the fix. All 45 full-suite characters (`menu`,
  60,000-turn cap, budget 50,000):

  | | reached N | died | out of budget | mean max depth |
  |---|---|---|---|---|
  | no errand | n/a | 8 | n/a | 2.89 |
  | `dive:3` | 34 (median 10,574 turns, 1,803–46,697) | 8 | 3 | 2.71 |
  | `dive:5` | 25 (median 11,438 turns, 2,612–45,647) | 13 | 7 | 3.98 |

  **The deaths that happen only under the errand:**
  - 5 under `dive:3`, 8 under `dive:5`;
  - mostly casters (mage, priest, necromancer) at character levels 1–5;
  - on DL2–DL4;
  - killed by wild cats, a scout, a cutpurse, Fang, a white jelly, a fatal
    wound.

  That is the cost of a dive: a weak character arrives where its level
  would not have taken it yet. A dive run stops when it arrives, so per
  turn played it is riskier than borg-rs's own play even where the totals
  match.
- **A first version steered only through readiness**, with the override
  at N, then at N + 5. It left three or four characters on DL1 until the
  budget ran out: borg-rs dives only when bored, or when it is prepared for
  five levels deeper. Steering after caution replaced it.

## Next

1. C2:
   - add *shop*;
   - add the `^Z` keys and `run.sh` flags.
2. C3 (#209): danger and caution into the domain behind a shadow check;
   measure the cost; report.
