## 2026-11-20: the ranger ships town_phase_reserve = 2 (#784)

Owner, 2026-11-20, right after #778 merged: "for rangers put
town_phase_reserve = 2" (a correction of the session's first instruction,
which said paladins; the paladin line was withdrawn before anything
merged).

**Done**: `harness/borg_ranger.txt` gains
`borg_param_town_phase_reserve = 2` (with the owner's words and #761's
mechanism in a comment); `scripts/borg-class-files.py fold` wrote
`borg_param_town_phase_reserve__ranger = 2` into `harness/borg.txt`, the
one file a run reads, and removed the folded paladin line the withdrawn
instruction had briefly added. `docs/CALIBRATION.md`'s #761 section now
names the one shipped exception to "every line at today's value". No other
class changes: the common `town_phase_reserve` stays 0, and the paladin is
back exactly as #778 left it.

**What the value does** (#761's PHASE_RESERVE capability): the first two
Scrolls of Phase Door are a requirement of the ranger's town trip, each
worth `town_phase_reserve_percent` (100, the default) of the escape weight,
on top of the Phase Door curve — so Phase Door outbids cheap arrows and
potions for the first two scrolls, and nothing past two.

**Checked**: `scripts/rerecord-smoke-borg-rs.sh` on the paladin commit
(3/3 identical, nothing re-recorded — the smoke seeds are a mage, a
warrior and a priest). The ranger commit changes the same kind of line and
the rerecord ran again on it: **3/3 runs identical** — no smoke seed meets
a ranger either, so `smoke-borg-rs` is not re-recorded and stays valid;
CI's engine job compares the PR merged with main and confirms it
independently.

**Environment note**: the first rerecord attempt failed before any
comparison — the sibling cemetery checkout `/home/diego/projects/borg-cemetery`
sits on main while `death-tools-pin.txt` pins the `pr-91` native tools, and
that checkout also held another session's uncommitted work (it was not
touched). The rerecord ran with `BORG_CEMETERY_DIR` at a clean separate
clone (`/tmp/opencode/borg-cemetery-run`: data at cemetery main, native
tools at the pin, `verify_native_pin` passing). A checkout whose native
tools match the pin is what any local suite run needs.

**Not checked**: no ranger game beyond smoke — the parameter's effect on
rangers is the zoo's to measure (borg-zoo#513's tiers are the owner's call;
RULES 14: nothing here claims improvement). The other #761 lines stay at
their shipped values: the floor switches (`flow_take_phase_short_below`,
`flow_take_phase_first`) 0, `town_unseen_percent` and the food and light
percents 100, and `town_phase_reserve_percent` 100 — the reserve's own
weight, not the count.

**For the owner**: merge the PR (#785) when its review is done (RULES 1a).
