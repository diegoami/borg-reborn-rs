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
- **The town plan** (`borg_town_capability.rs`): three reads of Detect Invisible
  are a requirement, worth 30% of the trip's escape weight each (5% when the
  Borg can cast Sense Invisible, Detection or Reveal Monsters), only when the
  purse after the urgent purchases is at least 250 gold. A staff fills the same
  requirement at far more gold, so it is bought when the scrolls are not stocked.
- **Eight new parameters**, four for the town plan (`town_invisible_count`,
  `town_invisible_min_gold`, `town_invisible_percent`,
  `town_invisible_spell_percent`) and four for the mode
  (`recover_unseen_detect_reads`, `_probe_tries`, `_probe_max_grids`,
  `_act_age`); #697's `recover_unseen_attack_turns` 500 and
  `recover_unseen_heal_hp_percent` 50 are now shipped. The compiled defaults
  stay off (a profile without these lines plays as before). Smoke re-recorded.

**Found:**

- **Control.** `3b4924ba` and this tree before the change, with the zoo's
  shipped settings, die exactly as the zoo's games did (the replay's keys are the
  Cemetery's hashes). **#697's two switches alone already save both seeds**
  (CL8/DL3 and CL6/DL2 at the 100,000-turn cap), so the death was a shipped
  setting, not missing code; the mode adds what the owner asked for on top.
- **After.** Both seeds reach 100,000 turns (CL6/DL2 and CL6/DL3), identical to the
  old games until the first unseen blow. In them the mode strikes, kills the
  attacker ("You have killed it.") or leaves, and the scrolls are bought with 213
  and 678 gold.
- **The borg-zoo bundles could not be downloaded** (the session had git access to
  borg-zoo, not the Actions artifacts API), so repro.sh failed at `gh run
  download`. The games were replayed from the Cemetery dossiers instead: the
  zoo's `borg.txt` is the shipped one with the cell's overrides **and with the
  `borg_world` line removed** (`borg-zoo/zoo/borgrs.py:config_text`, #704);
  `borg-cemetery/scripts/native/rerun-death.py` omits that last step, which is
  why it cannot rebuild these cells' `borg.txt` (its hash then differs).
- **The first town visit.** A percent high enough to buy scrolls later also
  bought them at CL1 with 139–151 gold, at the cost of a Cure Light Wounds or
  flasks; the 250-gold floor keeps both first visits identical to the old plan.
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
the default-core smoke suite re-recorded (see the PR for its result). Local, for
direction only (RULES 20): 12 Human Blackguards, off against on, 100,000 game
turns: 0 deaths in both, sum of CL 75 and 76, sum of DL 34 and 34, HP rests 469
and 404, the mode entered 29 times, no stalls. **No claim of improvement** (RULES
14): those games did not die of an invisible monster off either.
`scripts/borg-core-vendor.py check` was not run (no `vendor/borg-rs` here); every
file touched is already on its HOOKED list.

**Left / for the owner:**

- **A zoo arm** (RULES 20; I cannot file it on borg-zoo): the shipped settings
  against `borg_param_recover_unseen_attack_turns` 0 and
  `borg_param_town_invisible_count` 0, on the classes suites, measuring deaths to
  invisible monsters, resting stalls, the gold spent on Detect Invisible and the
  survival items it displaced. All eight numbers are hypotheses.
- **Decide:** whether 500 game turns, a 250-gold floor and 30% are the right
  strength, and whether a Staff of Detect Invisible should be worth more than the
  same reads in scrolls (today it is bought only when scrolls are not stocked).
- An unseen caster ("Something mumbles", #699) is not a blow and does not enter
  the mode.
- The branch is `claude/funny-ptolemy-t4a5xz` (the cloud session's), not rule 5's
  `task/...` name, and was worked in the session's own checkout.
