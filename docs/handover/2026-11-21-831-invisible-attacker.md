## 2026-11-21: an invisible attacker near: a mode, and Detect Invisible bought with spare gold (#831)

Owner: "when there is an invisible creature around, switch mode and deal with
it ... Scroll of detect invisible are not so worthless after all ... buy them if
you have the spare money (or staff of detect invisible), add some more value
for them especially for blackguards and those who do not have a spell to detect
invisible ... stop resting until you know that the invisible creature is gone
and / or leave the level."

Two Human Blackguards (2078477958, 2037495442) died resting beside a clear icky
thing: 23 and 44 "Something touches you", each followed by a rest, with
impaired HP regeneration. #697's switches existed but shipped off.

**Done** (all in `borg-core/`, shipped in `harness/borg.txt`; the
[CALIBRATION.md](../CALIBRATION.md) section "An invisible attacker near" has the
numbers):

- **The mode** (`borg_recover.rs`): an unseen blow enters it
  (`# Mode: invisible attacker near (y,x)`); 500 game turns without a blow, a
  new level or killing the attacker end it. #697's rest hold and walk away stay.
  New: a blow that has just struck (30 game turns) makes the Borg **detect**
  (`borg_fight_defend::detect_invisible_now`: the same scroll, potion, staff,
  activation and spell list as the defence, which now shares it) at most twice an
  episode, else **strike** the open adjacent grids with `alter` (it attacks an
  unseen monster and otherwise only spends the turn) while at most 3 grids are
  candidates, remembering the grid that struck something and those that spun;
  10 attacks an episode, then, still struck, it **leaves the level**. Hurt, it
  drinks below 50% (#697's heal, now on).
- **The town plan** (`borg_town_capability.rs`): a value curve, no reserve (owner:
  "3 is already too big a reserve ... no reserve, better value and a curve that
  depends on class"). A read of Detect Invisible is worth a diminishing curve like
  Phase Door's and the flasks': the first about 20% of the trip's escape weight, half
  the most it pays at 2 reads, and it competes with the rest of the purse. The percent
  is by class: 20 with nothing to detect, 10 with a Detect Evil spell, 3 with Sense
  Invisible, Detection or Reveal Monsters (`borg_magic::detects_invisible`,
  `detects_evil`). A staff gives the same units at far more gold, so it is bought
  when the scrolls are not stocked. A known Scroll of Detect Invisible is never
  sold or dropped (`borg_town_utility.rs`, `keeps_detect_invisible`; kept, or
  stored at the Home while overweight); the hard cap on copies is the planner's
  own 35 (`borg_town_max_non_missile_copies`). The first version (three reads a requirement
  behind a 250-gold floor) was replaced: see *Found*.
- **Eight new parameters**, four for the town plan (`town_invisible_count` 2,
  `town_invisible_percent` 20, `town_invisible_evil_percent` 10,
  `town_invisible_spell_percent` 3) and four for the mode
  (`recover_unseen_detect_reads`, `_probe_tries`, `_probe_max_grids`,
  `_act_age`); #697's `recover_unseen_attack_turns` 500 and
  `recover_unseen_heal_hp_percent` 50 are now shipped. The compiled defaults
  stay off (a profile without these lines plays as before). Smoke re-recorded: see
  *Checked*.

**Found:**

- **Control.** `3b4924ba` and this tree before the change, with the zoo's
  shipped settings, die exactly as the zoo's games did (the replay's keys are the
  Cemetery's hashes). **#697's two switches alone already save both seeds**
  (CL8/DL3 and CL6/DL2 at the 100,000-turn cap), so the death was a shipped
  setting, not missing code; the mode adds what the owner asked for on top.
- **After** (the current curve, replayed from the zoo bundles' own `user/` files;
  the off settings reproduce the zoo's key hashes `b19d7b6a...` and `8acb6b08...`
  exactly): both seeds reach 100,000 turns (CL6, max DL3 and CL6, max DL2), identical to the old games
  until the first unseen blow. The mode strikes, kills the attacker ("You have killed
  it.") or leaves; seed 1 buys 3 Scrolls of Detect Invisible (45 gold) at turn 50,982.
- **The first design flip-flopped** (the owner asked to fix it). The 250-gold floor
  was read on the purse as it is spent: `smoke-warrior-1` had 401 gold, 242 after
  the flasks and Phase Door, and sold the three scrolls it carried. The reserve is
  gone for a diminishing curve that does not depend on the gold, so no carried
  scroll is sold because the purse shrank, and the first visits (139 and 151 gold)
  still buy exactly what the old plan bought.
- **Zoo bundles** (runs 44894233710 and 44894253654) were downloaded after the
  session's environment allowed their blob hosts: `play_sha256` and `user/borg.txt`
  (`c96f0148...`) match the dossiers. The cemetery's
  `scripts/native/rerun-death.py` omits the step borg-zoo takes to build a cell's
  `borg.txt` (`zoo/borgrs.py:config_text`, #704: harness `borg.txt` plus the
  overrides, `borg_world` line removed), so it cannot rebuild these cells (hash
  53a19330 against c96f0148): a cemetery-side fix, no issue filed.
- **Not shown:** that 2037495442's 1,426 gold would have bought scrolls. That
  gold was found in the dungeon around turn 38,000 and the original never
  returned to town; the harness cannot give a Blackguard a purse. The
  valuation has unit tests and a 678-gold purchase.
- **Order matters:** with a scroll carried the older defence reads it before
  the mode's detect step. That step ran only with the defence turned off
  locally (`# Detecting the invisible attacker.`, "You sense the presence of
  invisible creatures!", the clear icky thing then fought).

**Checked:** `cargo fmt --check -p borg-reborn-rs`, `cargo clippy --workspace
--all-targets -- -D warnings`, `cargo test --workspace --exclude borg-core`,
`cargo test --release -p borg-core` (unit tests for the grid choice, the answers
to a blind attack, the kill message and the reserve), `scripts/check-param-citations.py`;
the default-core smoke suite re-recorded (`smoke-borg-rs`, 2 of 3 games identical; `smoke-warrior-1` diverges at game turn 19,397 in the first town visit: it keeps the identified Detect Invisible scrolls it carries, where the old reference sold them, and buys one more for 15 gold with the gold left). The 12-Blackguard comparison below is of the first design and was not re-run. Local, for
direction only (RULES 20): 12 Human Blackguards, off against on, 100,000 game
turns: 0 deaths in both, sum of CL 75 and 76, sum of DL 34 and 34, HP rests 469
and 404, the mode entered 29 times, no stalls. **No claim of improvement** (RULES
14): those games did not die of an invisible monster off either.
`scripts/borg-core-vendor.py check` was not run (no `vendor/borg-rs` here); every
file touched is already on its HOOKED list.

**Left / for the owner:**

- **A zoo arm** (RULES 20; filed as borg-zoo#580): the shipped settings
  against `borg_param_recover_unseen_attack_turns` 0 and
  `borg_param_town_invisible_count` 0, on the classes suites, measuring deaths to
  invisible monsters, resting stalls, the gold spent on Detect Invisible and the
  survival items it displaced. All eight numbers are hypotheses.
- **Decide:** whether 500 game turns and 20/10/3% with half value at 2 reads are the right
  strength, and whether a Staff of Detect Invisible should be worth more than the
  same reads in scrolls (today it is bought only when scrolls are not stocked).
- An unseen caster ("Something mumbles", #699) is not a blow and does not enter
  the mode.
- The branch is `claude/loving-knuth-y3cjl8` (the cloud session's; the work began
  on `claude/funny-ptolemy-t4a5xz`, merged in), not rule 5's `task/...` name, and was
  worked in the session's own checkout.
- **Cemetery:** `scripts/native/rerun-death.py` omits the step borg-zoo takes to
  build a cell's `borg.txt` (`zoo/borgrs.py:config_text`, #704: harness `borg.txt`
  plus the cell's overrides, `borg_world` line removed), so it cannot rebuild these
  cells (hash 53a19330 against c96f0148). A cemetery-side fix; no issue filed.
