# Borg family after impact v10

Owner direction (2026-11-10): the accepted `4.2.6-impact-v10` snapshot is the
base for a family of Borgs, not for automatic later better-borg-c carries.
[borg-rs#25](https://github.com/diegoami/borg-rs/issues/25) defines this bounded
first lane. New variants may intentionally change the keypress stream;
`current` remains the compatibility control. A reviewed and owner-accepted
variant can receive a `4.2.6-impact-v10-<variant>` tag, but an experiment or a
configuration combination does not automatically earn one.

## Initial strategy registry

| Strategy | Behaviour | Default / evidence status |
| --- | --- | --- |
| `current` (`0`) | Accepted impact-v10 power-valuation buying | Default for every unspecified class and mode; C/Rust parity remains its control |
| `reserve410` (`1`) | Through `BI_MAXCLEVEL <= 7`, hold four Phase Doors and three CLW/CSW/CCW potions. While a deficit remains, buy the cheapest affordable deficit item and suppress other useful purchases. Food, cuts, poison and light at radius 1 or below retain the existing urgent paths. A home-sell guard keeps reserve items in the pack until the deficit closes. | Experimental Rust-only variant; not a survival/progress claim |

`reserve410` is inspired by
[better-borg-c#410](https://github.com/diegoami/better-borg-c/issues/410),
not carried as a successful upstream fix. That proposal's first version caused
a buy/store loop. Its second version prevented the loop by retaining reserve
items, but still failed the declared survival and max-depth-loss margins and
was rejected. The family experiment exists to make such policy differences
selectable and measurable, not to erase that evidence.

The home-sell guard stops the guard-controlled sell/buy pass while the deficit
is open. It does **not** remove the broader store→refill cycle seen in the #410
experiment: the bounded Warrior/munchkin run still sells reserve items to the
home and buys them again after returning. This variant is an experiment, not a
fix for that cycle.

### Independent buy-to-identify policy (#27)

`borg_buy_identify = 1` enables a separate early learning policy, while zero
leaves it off. Through `BI_MAXCLEVEL <= 7`, the Borg may buy one affordable
copy of a still-unaware kind from this deliberately small list: Cure Light
Wounds, Cure Serious Wounds, Remove Hunger, Detect Invisible and Monster
Confusion. The engine's real `kind->aware` flag determines eligibility, so a
kind is not learned repeatedly after its first purchase.

Food, cuts, poison and light at radius 1 or below keep the existing urgent
paths and run first; then any unmet `reserve410` deficit, then learning
purchases, then the old valuation. Missing or unaffordable learning stock falls
through to the old valuation instead of suppressing it. While an unmet reserve
cannot be afforded, the strict reserve policy still blocks learning and the old
valuation together. This is [better-borg-c#412](https://github.com/diegoami/better-borg-c/issues/412)
independently selectable in borg-rs, not a carry or a benefit claim. It can be
combined with either buying strategy without changing the default.

## Configuration and precedence

The C and Rust parsers carry the same seven integer fields. C ignores both the
strategy selection and `borg_buy_identify` and always executes the `current`
policy; Rust resolves the selected family policy when making a buying or
home-selling decision.

| Setting | Default | Meaning |
| --- | --- | --- |
| `borg_buy_strategy_default` | `0` | Global strategy: `0` current, `1` reserve410 |
| `borg_buy_identify` | `0` | Independent early learning policy: `0` off, `1` on; parsed but ignored by C |
| `borg_buy_strategy_munchkin` | `-1` | Munchkin override: `-1` inherit, `0` current, `1` reserve410 |
| `borg_buy_strategy_current_classes` | `0` | Class mask forcing current in either mode |
| `borg_buy_strategy_reserve410_classes` | `0` | Class mask forcing reserve410 in either mode |
| `borg_buy_strategy_current_munchkin_classes` | `0` | Class+munchkin mask forcing current |
| `borg_buy_strategy_reserve410_munchkin_classes` | `0` | Class+munchkin mask forcing reserve410 |

**2026-11-14 (borg-rs#60): the munchkin dimension is gone.** There is one
play mode, so the three munchkin rows above are retired settings (an old
config gets a warning at their inert values, -1/0/0, and a refusal
otherwise), and the precedence below is now: ordinary class masks, then the
global default. To select a strategy for a class, use the class masks; the
munchkin examples below are the record of the earlier selection.

Class masks use `1 << class`: Warrior 1, Mage 2, Druid 4, Priest 8,
Necromancer 16, Paladin 32, Rogue 64, Ranger 128, Blackguard 256 (`0x1ff` is
all nine). Resolution precedence is:

1. class+munchkin masks, only when the effective munchkin-start setting is on;
2. ordinary class masks;
3. the munchkin override;
4. the global default.

The two masks at one precedence must not name the same class; Rust rejects that
configuration during initialization. Values outside the documented strategy or
mask ranges are also rejected. Less specific selections may intentionally be
overridden in either direction. With all defaults zero except the munchkin
inherit marker, every class and mode resolves to `current`.

For example, `borg_buy_strategy_reserve410_classes = 1` selects reserve410 for
Warriors in both modes while leaving every other class at the global default.
To make Warriors current only for munchkin while retaining reserve410 outside
munchkin, add `borg_buy_strategy_current_munchkin_classes = 1`. The existing
harness `NAME=VALUE` flags can set these integers directly.

## Bounded evidence for the first tranche

The implementation's focused checks cover:

- parser defaults, every setting, invalid values and conflicting masks;
- class/mode precedence in both directions, including different Warrior and
  Mage choices for the same mode;
- opt-in, reserve-complete, above-window and urgent reserve gates;
- Phase Door and exactly the three accepted cure tiers;
- the default `current` C/Rust smoke streams and the no-C-Borg link audit;
- explicit reserve410 runtime configuration and purchase behaviour.

The home-sell guard is part of the same reserve gate as the purchase pass, so
`current` never pays its cost or changes its decisions. No large cohort or
population claim belongs to this tranche.

### Executed implementation checks (2026-11-10)

- `cargo test --release --manifest-path borg-rs/Cargo.toml`: **114 passed, one
  existing ignored**. This includes selection precedence, invalid/conflicting
  configuration, urgent/completed gates (including light at radius 1 or below),
  item classification, home-sell retention, the pending-sale goal reset and the
  buy-to-identify gate/list/awareness predicate.
- `cargo clippy --release --all-targets --manifest-path borg-rs/Cargo.toml --
  -D warnings`: clean.
- Fresh C and Rust builds plus `harness/check-no-c-borg.sh`: pass; no C Borg is
  linked into the Rust image.
- `harness/suite.sh smoke` in both roles followed by `harness/diff`: **3/3
  C/Rust identical**. All nine key/message/state streams also match the
  retained impact-v10 C smoke outputs exactly. This checks `current`, not a
  claim that reserve410 has a C oracle.
- `BORG_IMPL=rust harness/suite.sh family-buying harness/out/family-buying`:
  all **8/8** cases complete with terminal saves.
- `python3 harness/regress/family-buying-test.py
  harness/out/family-buying`: **PASS**. All four direct/class-or-mode-selected
  pairs are byte-identical on keys, messages, state and final save; Warrior
  and Mage each differ between current and reserve410 and show at least four
  Phase Door and three Cure Light Wounds purchase anchors.
- `BORG_IMPL=rust harness/suite.sh family-identify harness/out/family-identify`
  followed by `python3 harness/regress/family-identify-test.py
  harness/out/family-identify --baseline harness/out/family-buying`: **PASS**.
  All four cases complete. Every learning decision is noted as
  `# Learning '<desc>'`; each such note names an `{unseen}` kind and none
  follows the engine's completed sale of that kind, so the awareness gate is
  visible in the stream. Each learning kind also has at most one executed
  `{unseen}` purchase, and no `{unseen}` goal follows the completed sale.
  Exactly three cases show independent (non-reserve) learning and the
  documented on/off changes; the reserve410/Warrior case is the one expected
  preemption and stays byte-identical. That case leaves town before the
  learning stage, so it is a valid no-effect case, not evidence of a behaviour
  flip. A mutation control on 2026-11-10 removed the `kind->aware` check from
  the candidate predicate: the validator then fails (the Mage buys 16 Cure
  Light Wounds instead of 5), and it passes again with the check restored.

`harness/run.sh` accepts digits in `borg_*` option names so the reserve410
settings work through the existing generic `NAME=VALUE` flags. The failed
pre-fix attempts to set those two options had no game output and are not
counted as passes.

## Small future zoo plan (not run by this issue)

Use the existing borgh measurement interface from `docs/harness.md` and #7.
(2026-11-14, borg-rs#60: the mode axis of this plan no longer exists; the
matrix below is the record of the plan as written.)

Freeze the seeds and exact source/config identities before any comparison. A
small first matrix is 24 runs:

- strategies: `current`, `reserve410`;
- classes: Warrior (class-based munchkin capable), Mage (not class-based);
- modes: explicit non-munchkin, explicit `munchkin`;
- seeds: 11, 29, 47;
- cap: 20,000 turns, with terminal saves and the existing state/RNG summaries.

Disable class-based munchkin for the explicit non-munchkin Warrior cells;
enable `borg_munchkin_start` for the explicit munchkin cells. Record alive/dead,
max depth, class level, HP, gold and stall/stop reason per cell, with paired
current/reserve traces. This is a later evaluation plan, not an execution
result or a survival/progress claim. Do not enlarge it into a full class ×
strategy × mode sweep without a fresh owner-directed scope.
