# source193: leave Lunal mode when a monster teleports the Borg (patch 0120)

Consumer adoption of [source 193](https://github.com/diegoami/neo-angband-tracker/issues/193),
fix `4ee9a38f3` (branch `task/lunaltele-193`), kit `docs/audit/c-reports/lunaltele-193` from the
evidence branch `task/lunaltele-193-evidence`.

## Defect

Lunal mode makes the Borg stair-scum: it ignores danger and preparedness checks on the assumption
that it stands on, or next to, a staircase. `borg_parse_aux()` tries to switch it off when a monster
spell moves the Borg ("commands you to return." and similar), but that test compares the start of the
line while the game's line starts with the monster's name, and every monster spell line is recognised
earlier and queued as a `SPELL_nnn` reaction before the test is reached. After a teleport-to or
teleport-away the Borg therefore stayed in Lunal mode, away from the stairs, and kept making Lunal
choices such as heading down to a depth it is not prepared for.

## Fix

Switch Lunal mode off where `borg_update()` handles the queued spell reaction for `RSF_TELE_TO` and
`RSF_TELE_AWAY` (`+9` lines). Self-engaged Lunal mode is re-evaluated on the next dungeon think as
before. Source `4ee9a38f3` -> applied `62ffd03a5` (`cherry-pick -x`), patch
`patches/0120-lunal-teleport-off.patch`, scope `src/borg/borg-update.c` only.

## Verification

Before commit `7a4ec7832` (the kit commit, `src/` identical to `origin/main`), tip `62ffd03a5`, kit
unchanged (`tools/verify-lunaltele193.py`). Native gates green at both (965/965 unit, 5/5 frontend);
byte-wise provenance over the source diff; the `src`/`lib`/build scope is the one file.

| Mode | Before | After |
| --- | --- | --- |
| `teleto` (tengu "commands you to return", Human Warrior, seed 193, level 15, depth 10, Lunal configured) | **fails** 0/1: Lunal mode stays on, the Borg heads for the down staircase although unprepared for depth 11 (`lunal_after=1`, `heads_down=1`) | passes 1/1: Lunal mode off, descent refused (`0`, `0`) |
| `teleaway` (mage "commands you to go away") | **fails** 0/1 (`1`, `1`) | passes 1/1 (`0`, `0`) |
| `teleaway-unseen` ("Something commands you to go away") | **fails** 0/1 (`1`, `1`) | passes 1/1 (`0`, `0`) |
| `shriek` (a spell that does not move the Borg), control | passes (`1`, `1`) | passes, **byte-identical** |
| `teleto-nolunal` (Lunal mode already off), control | passes (`0`, `0`) | passes, **byte-identical** |

The committed `before-*.log` / `after-*.log` are the source lane's captures; the product runs of this
tree (`product-result.json`) are byte-identical to all ten of them, so the kit reproduces unchanged.
The source's own rows and hashes are in `receipt.json`, carried unchanged.

## Mode attribution

Per the 2026-11-10 owner rule, each configuration is stated, and "neutral" is claimed only where the
changed branch executes. The new branch is guarded by `borg.lunal_mode`, which is set only by the
configured `borg_lunal_mode` (`borg-init.c`) or by the self-engaged Lunal mode (`borg_self_lunal`,
`borg-think-dungeon.c`). Both default to false.

| Configuration | Branch reachable? | Result |
| --- | --- | --- |
| normal only (every mode pinned off: `normal` zoo profile) | **No**, by construction: `borg.lunal_mode` is never true | no behaviour change; not a measurement |
| munchkin / class rule (`borg_munchkin_by_class`, `borg_munchkin_start`) | **No**, unless borg.txt also sets Lunal: a munchkin start is a different mode from Lunal | no behaviour change; not a measurement |
| Lunal only (`lunal` zoo profile: `borg_lunal_mode` and `borg_self_lunal` on) | Yes | the fixture's Lunal-configured path executes it (above); cohort effect **not measured** |
| unrestricted (all modes on) | Yes, whenever Lunal is on | cohort effect **not measured** |

So the zoo's `normal` and `munchkin-class` arms cannot move on this carry; only a `lunal` arm can show
its effect, and no such arm has run. That batch belongs to borg-zoo issue 54.

Matrix group `lunaltele-193` (5 cases) uses the `after-*.log` rows as its baseline. The full-matrix
result is below.

## Ledger and credit

`patches/series.json` `LUNALTELE-193`, patch-active; consumer issue #484. Source defect, fix and
native fixture by Diego Amicabile (native proposal/evidence); carried unchanged.

Full matrix at this tree: 98 groups / 299 cases, `ok: true`, no failed group, 246/259
byte-identical; the 13 byte differences are in the same six groups as before this carry (`dispel`,
`dispel259`, `fleeheal-408`, `knockout`, `preparation`, `vampire`), so no new difference.
