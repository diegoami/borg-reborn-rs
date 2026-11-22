# 2026-11-22 — #915: grouped monster messages read as their race

**Issue:** diegoami/borg-reborn-rs#915. **Branch:** `task/T915-grouped-messages` (from `track-p`
`86843a79`, `track-p` merged in up to `72651d25`, after #1026 and #1036). Track P, **P8 "Stabilize P7"**
(owner, 2026-11-22: #915 is a P8 bug; a P8 fix ships on once the evidence supports it). This PR
merges into `track-p` once the owner says its review is done.

Evidence: [borg-cemetery study](https://github.com/diegoami/borg-cemetery/blob/claude/happy-brahmagupta-2jcsa3/studies/fastfrag-cave-spiders-2008390085/README.md) (seed 2008390085, borg-zoo#596).

## What was done

- New parameter `kill_grouped_names`, compiled 0, **shipped 1** in `harness/borg.txt`. With it
  on, `guess_race_name` reads the engine's grouped subject "N <plural>" as the race: first the
  name as it stands (a monster.txt `plural:` is in the normal table), then `plural_aux` undone,
  "-ses" → "-s" and "-s" → "". "N monsters" is an unseen group, like "It". An unknown grouped
  name is noted (`# Grouped monster not known (...)`) and still returns the ghost race, unused,
  as before. `best_normal` factors out the existing scan, unchanged.
- `grouped_subjects_of_every_engine_race`: a unit test over the engine's own `monster.txt`.
- **Review of `32e9b8f0`** (Mavis, MiniMax M3.1-Flash, posted on #943): fixed in `8622a276` and
  after.
  - Finding 2: a grouped death or blink ("3 cave spiders die.", "N <plural> disappear!") is now
    applied to **up to N** known monsters of the race (`grouped_count`; the DIED and BLINK
    handlers of the goal, old-location and late passes in `borg_update.rs`), as #915 asked. With
    the switch at 0, or for any other subject, the count is 1 and the code is the old one.
  - Finding 5: the comment on `grouped_race` now says the "-ses" stem is ambiguous ("masses" from
    "mass", "horses" from "horse") and why the order is safe in 574b618c.
  - Finding 1: no change. "It (offscreen)" goes to the offscreen branch, not the invisible one:
    `strnieq(who, "It", 3)` compares the terminating NUL (`borg_flow_kill.rs:285-297`), so it is
    an exact match. "N monsters (offscreen)" takes the same branch, so the two already agree. A
    test now pins it.
  - Finding 3: `track-p` merged again (`72651d25`); `CALIBRATION.md` and `borg.txt` keep both
    sides; 2,398 parameters, 0 citation errors. #980's sentence "the registry now holds 2394"
    is left as written: like #975's, it states the count of its own PR's base. The #915 section
    states the count with this PR (2,398).
  - Finding 4: the measurement is borg-zoo#652's `grouped` arm, not P8's exit run (borg-zoo#695,
    pinned at `206cf4e7`, without #943).
  - Finding 6: the footer dropped from the PR body. Finding 7: #1042 filed.
- `track-p` merged twice (before and after #1026); the parameter tables kept both sides' rows,
  `check-param-citations.py --fix` re-pointed the citations (2,394 parameters, 0 errors).
- `scripts/rerecord-smoke-borg-rs.sh` with the switch shipped on, at `ba188fc2` and again at
  `8622a276` (with up to N and `track-p` `72651d25`): smoke 3/3 identical to
  `harness/reference/smoke-borg-rs` both times, so nothing was re-recorded (no smoke game has a
  grouped monster message).

## The engine (574b618c)

- `get_subject`, `src/mon-msg.c:320-356`: invisible (not obvious, `message_flags`, `:166-179`):
  "It" or "%d monsters" (`:326-331`); a unique: its name (`:334-335`); one monster:
  "The %s" (`:336-337`); more: "%d %s" with `race->plural` (`:339-340`), else the name and
  `plural_aux` (`:341-344`). `RF_NAME_COMMA` adds "," (`:347-349`; only uniques carry it in
  `monster.txt`). " (offscreen)" is appended last (`:352-353`).
- `plural_aux`, `src/mon-desc.c:27-36`: "es" after a final 's', else "s".
- The message body is in the plural when count > 1 (`show_message`, `mon-msg.c:470-490`), which
  is why the Borg's pain table has both forms (`borg_insert_pain`).

## What was found

- **Every non-unique race reads back.** All 523 non-unique races of `monster.txt` (96 with a
  `plural:`), with a count of 2 and 15, with and without " (offscreen)", resolve to a race of
  their own name; no grouped form is shared by two names; no subject reaches the engine's
  60-byte buffer. Names already ending in 's' ("white worm mass", "cyclops", "creeping copper
  coins") go through `plural_aux`'s "es" or their `plural:`. Singular ("The cave spider") and
  unique names are unchanged. No unique starts with a digit, so a unique is never read as a group.
- **The motivating case, the zoo's own game** (`7f8c6b80` + the #915 commit, local, never
  pushed; the study's `borg-stats-ban.txt`, sha256 `58b6d86c55df`): switch off plays the zoo game
  byte for byte (keys `1ef12466…`, messages `8410b808…`, state `064f346c…`; death by a cave
  spider at 64,483). Switch on: the same keys and state. The only difference is in the notes at
  64,434: "3 cave spiders start moving faster." is no longer "Assuming player ghost" three times;
  "PAIN:3 cave spiders" is used in pass 2 (matched to a known cave spider near the goal).
- **On `track-p`** (this branch, seed 2008390085, 100,000 turns): shipped settings and the
  stats-ban arm, switch off or on, all four live to the cap with the same keys (`d2ea6bb9…`),
  the same state.log and the **same core trace** (`trace-core.jsonl`, danger at every think). The
  grouped message there is "2 giant white mice look even faster!" (a `plural:` name, 37,788),
  matched to a known giant white mouse with the switch on. Again at `8622a276` (`track-p`
  `72651d25`, where `kill_live_zero_hp` ships 1): the four games have the same keys
  (`a5ad4668…`), messages, state and core trace, and this game has no grouped message at all.
  The zoo game with up to N: the same as before (keys `1ef12466…`, the same notes).
- **Up to N is not exercised by any game played here**: none of them has a grouped death or
  blink. Only the unit tests check the count (`group_size`).
- **Why the danger does not move.** A matched pain/status message only marks the message used;
  the kill's speed, haste and sleep come from the engine monster each update
  (`borg_update_kill_new`, `set_timed_flags`), not from the message. With the switch off the
  message went to the ghost race (Morgoth: glyph 'P', colour 'D'), which could have converted a
  known monster of that glyph and colour into Morgoth (`borg_locate_kill`'s "Converting a
  monster"); no non-unique giant is colour 'D', so that never happens in 4.2. What the switch
  changes in play: a grouped death ("3 cave spiders die.") now removes one known monster of the
  race at once, and "N monsters" stamps the need for See Invisible as "It" does.
- **#941 / #947 (`trait_timed_sync`).** Same parser path, different stage, no conflict:
  #947 keeps "You feel ..." out of the pain table in `parse_monster_message`
  (`borg_messages.rs`); #915 acts later, when `borg_locate_kill` names the race of a "PAIN:"
  subject (`borg_flow_kill.rs`). Their messages are disjoint ("You feel ..." has no count). Only
  the parameter tables touch, as adjacent rows. Neither depends on the other.
- **A neighbour of #941 that #947 does not cover** (filed as #1042): "You are no longer confused."
  (`player_timed.txt:227`) matches the pain suffix "[is|are] no longer confused."
  (`list-mon-message.h:39`) and is read as "PAIN:You" ("Assuming player ghost (You)", turn 7,083
  of the `track-p` game). Harmless today (the confusion flag is read from the engine), but the
  same misreading as #941.

## What is left / owner

- Population measurement of the shipped switch: borg-zoo#652, arm `grouped`
  (`borg_param_kill_grouped_names=1` against `ctl`). Its specimen `67a0c69d` has the one-monster
  reading, not up to N, and the arm is not in #652's "if capacity is short" list. **Owner:**
  include `grouped` when capacity is short? Re-specimen #652 to this PR's head? On a tree with
  this PR, `ctl` must say `borg_param_kill_grouped_names=0` to stay "switch off".
- "You are no longer confused." as monster pain: #1042 (owner: fold into #947, or its own task).
