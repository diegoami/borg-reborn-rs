## 2026-11-20: the paladin ships town_phase_reserve = 2 (#784)

Owner, 2026-11-20, right after #778 merged: "for paladins put
town_phase_reserve = 2".

**Done**: `harness/borg_paladin.txt` gains
`borg_param_town_phase_reserve = 2` (with the owner's words and #761's
mechanism in a comment); `scripts/borg-class-files.py fold` wrote
`borg_param_town_phase_reserve__paladin = 2` into `harness/borg.txt`, the
one file a run reads. The paladin block keeps its shape (the line sits
between the phase-curve and oil-curve lines). `docs/CALIBRATION.md`'s #761
section now names the one shipped exception to "every line at today's
value". No other class changes: the common `town_phase_reserve` stays 0.

**What the value does** (#761's PHASE_RESERVE capability): the first two
Scrolls of Phase Door are a requirement of the paladin's town trip, each
worth `town_phase_reserve_percent` (100, the default) of the escape weight,
on top of the Phase Door curve — so Phase Door outbids cheap arrows and
potions for the first two scrolls, and nothing past two.

**Checked**: `scripts/rerecord-smoke-borg-rs.sh` on this commit (builds the
engine, plays the smoke suite on the default borg-rs core): **3/3 runs
identical** — the smoke seeds (mage, warrior, priest) never meet a paladin,
so `smoke-borg-rs` is not re-recorded and stays valid; CI's engine job
compares the PR merged with main and confirms it independently.

**Environment note**: the first rerecord attempt failed before any
comparison — the sibling cemetery checkout `/home/diego/projects/borg-cemetery`
sits on main while `death-tools-pin.txt` pins the `pr-91` native tools, and
that checkout also held another session's uncommitted work (it was not
touched). The rerecord ran with `BORG_CEMETERY_DIR` at a clean separate
clone (`/tmp/opencode/borg-cemetery-run`: data at cemetery main, native
tools at the pin, `verify_native_pin` passing). A checkout whose native
tools match the pin is what any local suite run needs.

**Not checked**: no paladin game beyond smoke — the parameter's effect on
paladins is the zoo's to measure (borg-zoo#513's tiers are the owner's
call; RULES 14: nothing here claims improvement). The Ranger, Rogue and
floor-switch lines of #761 stay 0.

**For the owner**: merge #784's PR when its review is done (RULES 1a).
