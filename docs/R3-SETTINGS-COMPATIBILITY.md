# Pinned M13 configuration compatibility audit

R3 review #413 finding 4; audit of borg-zoo catalogue/profiles.json at
f5043c96a1a427d7d5cb624c91a795ef3277ae9f. No zoo files or historical profiles were changed.

Of 75 catalogue profiles, 24 explicitly set at least one retired
M13 mode setting to a non-inert value. Those combinations must not be used
with this M13-based receiving core. They can remain valid for older/C Borg
specimens; this audit is not a global prohibition on those historical profiles.

| Profile | Non-inert retired settings |
| --- | --- |
| munchkin-class | borg_munchkin_by_class=true |
| munchkin-on | borg_munchkin_start=true |
| lunal | borg_self_lunal=true, borg_lunal_mode=true |
| munchkin | borg_munchkin_start=true |
| munchkin-d6 | borg_munchkin_start=true |
| munchkin-d10 | borg_munchkin_start=true |
| risky-munchkin | borg_munchkin_start=true |
| rs-buy-current-munchkin | borg_munchkin_start=true |
| rs-buy-reserve410-munchkin | borg_munchkin_start=true |
| rs-buy-current-munchkin-id | borg_munchkin_start=true |
| rs-buy-reserve410-munchkin-id | borg_munchkin_start=true |
| sw-no-self-scum | borg_munchkin_by_class=true |
| sw-no-swaps | borg_munchkin_by_class=true |
| sw-risky | borg_munchkin_by_class=true |
| sw-dynamic-calcs | borg_munchkin_by_class=true |
| sw-enchant-20 | borg_munchkin_by_class=true |
| sw-enchant-5 | borg_munchkin_by_class=true |
| sw-chest-2 | borg_munchkin_by_class=true |
| sw-kills-uniques | borg_munchkin_by_class=true |
| rs-buy-munchkin-override-current | borg_buy_strategy_munchkin=0, borg_munchkin_start=true |
| rs-buy-classmunchkin-reserve-warrior | borg_buy_strategy_reserve410_munchkin_classes=1, borg_munchkin_start=true |
| rs-buy-classmunchkin-current-warrior | borg_buy_strategy_current_munchkin_classes=1, borg_munchkin_start=true |
| normal-munchkin | borg_munchkin_start=true |
| town-negative-munchkin | borg_munchkin_start=true |

This is a static settings audit, not a native scenario/cohort execution. It
checks the explicitly named retired values, not every unrelated option,
profile inheritance rule or arbitrary user-supplied profile. Unlisted profiles
are not automatically certified compatible. Ordinary normal/off mode values
are inert and accepted with warnings where the donor permits them.

The receiver validates BORG_ERRAND with its existing command parser,
BORG_STOPSCARED as none/off/flee or the documented bounded threshold/aliases,
and BORG_ERRAND_BUDGET as none or a nonnegative i64 game-turn budget. Invalid
UTF-8, syntax and values exit 2. Budget 0 is unbounded, not a negative sentinel.
The R3 corpus uses the vendored core, pristine core settings, no errands and
stopscared off. This audit does not loosen that measured contract.

Before promotion or an owner-started M13 cohort, enforce specimen/profile
compatibility in the owning zoo admission path and validate actual supplied
environment/configuration through those parsers. Track the operational gate
in #419. No zoo write, population experiment or ready-redesign promotion is
authorized by this audit.
