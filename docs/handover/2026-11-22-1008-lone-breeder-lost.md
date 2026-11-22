## 2026-11-22: The lone-slow attack on a level lost to breeders (#1008)

Branch `task/T1008-lone-breeder-lost`, from `track-p` `6bc4de1d`. Track P,
P8 (owner: "do as you suggest and start working on those issues"). A
follow-up of #878 (PR #891), one of three filed after checking its switches
against the shipped breeder policy (#1008, #1009, #1010; the fourth
suggestion, a `lone-light` arm with `cave_light_engine_radius = 1`, went to
borg-zoo#626 as a comment).

**Done.** `lone_slow_breeder_lost` (flag, compiled and shipped 0; registry
2385 to 2386): with `lone_slow_attack` on, the lone rule leaves a breeder
alone while the level is `breeders_lost`, the condition the scary list's
breeder attack entries already respect. A note once a think says so. Code:
`defers_to_lost_level` in `borg_lone_target.rs`, with a unit test.

**Checked.** CI's commands (see the PR); citations (every `lone_slow_*`
parameter cites its read line exactly; two moved with this change); smoke at
the shipped settings 3/3 identical to `smoke-borg-rs`.

**Not done / left.**
- Not exercised in a game: #878's named-case arms never reached a lost
  level (no "the level is lost" note in the `lone` arm).
- **Measurement:** a borg-zoo arm `lone_slow_attack=1
  lone_slow_breeder_lost=1` against `lone_slow_attack=1` (borg-zoo#626's
  `lone`). Not filed: it is the owner's to start (RULES 20).

**For the owner.** Whether the deferral should cover a lost level only (as
built) or also the stand's one-think pause (`breeder_stand_paused`); the
latter is fear-driven today (#1009).

### After the review (PR #1011, pinned `84249218`)

- Synchronized with `track-p` `72651d25` (one CALIBRATION.md conflict, both
  sections kept; registry 2397 to 2398).
- F1: `attack_asks_the_lost_level_deferral_before_choosing_an_attack` pins the
  wiring: `attack`'s source asks `defers_to_lost_level` with the switch, the
  breeder flag and `breeders_lost` before `focused_ranged_attack`, and
  returns. `attack` needs the engine, so this is a source test (the
  `recall_restricts` pattern); removing the deferral fails it (checked).
- F2: the parameter and the CALIBRATION row name `caution_breeder_stoppable`:
  `breeders_lost` is set only with it on.
- The reviewer's observation (the winning fight still attacks an all-HURT
  breeder, a louse or a fly, on a lost level) is the winning-fight
  certificate question, filed with #1015's follow-up.
