# Declarative caster build records

`borg-reborn-rs/character-build@2` extends the deliberately bounded @1 adapter
for Mage, Druid, Priest and Necromancer (class indices 1–4), all eleven engine
races, levels 1–15. The caller must select the same race/class at birth.
It is a projected starting character, not a native save, historical encounter
reconstruction or private Borg-memory restoration. The generated-depth contract
remains `synthetic-start@2`; the independent character schema identifies resource
and spell semantics. Existing @1 Human Warrior records remain unchanged.

The fixed-order ASCII record has at most 16 KiB, LF line endings, a final newline,
no duplicate/unknown directives or blank lines, and this layout:

```text
schema:borg-reborn-rs/character-build@2
engine:<exact upstream engine SHA>
identity:<race index>:<class index>:<level>
observed:<source engine turn>:<current HP>:<current SP>
vitals:<experience>:<gold>:<maximum HP>:<maximum SP>
stats:<five undrained natural stats, engine STAT order>
hp-rolls:<all 50 cumulative native HP rolls>
knowledge:<to-hit rune>:<to-damage rune>:<to-armor rune>
spells:<class total spell count>:<remaining study opportunities>
spell:<flags at index 0>:<learning-order entry 0>
... one spell line for every native class spell index ...
item:<tval>:<sval>:<quantity>:<equipped slot or -1>:<to-hit>:<to-damage>:<to-armor>:<pval>:<timeout>:<kind-aware>
... all carried items ...
end
```

The spell count must equal the engine's class table (bounded at 64). Flag bits
are the native `PY_SPELL_LEARNED`, `PY_SPELL_WORKED` and `PY_SPELL_FORGOTTEN`;
first-success XP therefore is not silently re-granted. Learning-order entries
are unique indices followed by sentinel 99. They must correspond exactly to
learned/forgotten spells, with no contradictory flags, gaps or extra entries.
After application, native bonus, HP, mana and spell recalculation must leave
the declared maxima, study count, flags and learning order unchanged. Invalid
states fail; the adapter never learns spells or fills in a book automatically.
Knowing a spell without its book remains legal state; actual casting still
requires the normal engine checks, including book, mana, light and status.

Native authorities at `engine/UPSTREAM`: `player.h:PY_SPELL_*`,
`player-spell.c:spell_learn`, `player-calcs.c:calc_spells`, `calc_mana`,
`calc_hitpoints`, and `obj-gear.c` equipment mechanics. Manual context is
`reference/c-manual/03-magic-and-spells.md` and `06-equipment.md`; their advice
does not override the pinned engine. Armor/weight affects native mana and must
be reflected in the record's maximum SP, not bypassed by an adapter formula.

## Resources and supported projection

Unlike @1's full-resource normalization, @2 preserves **observed integral HP/SP**,
including living HP=0 and exhausted mana. Fractional HP/SP is reset to zero.
Experience, HP rolls, base/current stats, gold, physical mundane kit, equipped
slots, combat-rune knowledge and carried-kind awareness use the existing
projection. Other historical object knowledge is not restored. Nutrition is
normalized to the new birth's state; active non-nutrition effects, shapechange,
stat drain and experience drain are unsupported for capture. Generated cave,
monsters and fresh Borg memory come from the declared scenario, not the source.

Gear still must have native base-kind flags, elements, weight, dice and AC,
with no artifacts, egos, curses, brands/slays or nonzero item stat modifiers.
Quantities are 1–40, at most 48 carried objects within pack capacity, combat
bonuses bounded to ±20, equipped slots legal/unique, equipped stacks size one.
@2 additionally refuses rods/wands/staves and every nonzero pval; per-kind
charge/knowledge support remains #411. Timeout is only allowed for ordinary
fuel-using lights within native torch/lantern fuel limits. An unsupported
carried item rejects the entire record, not just that item.

Use `-capture-build PATH -capture-build-level N` on an ordinary matching
race/class run. Capture is read-only on supported state and preserves existing
files. Failure to reach the level remains an explicit absent capture; reaching
unsupported state fails visibly instead of silently writing a weaker kit.
This capture is at a logged-key boundary, not a restored historical snapshot.
The source run/build/stream identities and selection must be retained in
Cemetery before using it as an observed scenario catalog entry.

Apply with `start-depth=N -- -start-build PATH`. The engine independently hashes
the applied record. Summary adds `start_build_schema`, actual initial HP/SP,
native AC/to-armor and learned/worked/study counts alongside the existing
initial cave/depth/level/maxima checks. Generation finishes before activation.
Synthetic games remain excluded from the natural death registry.

## Validation and remaining integration

Native tests capture all four classes and verify unchanged capture/control
streams, repeated application, exact hash/race/class/resources, malformed spell
arrays, inconsistent derived mana, equipment slot conflicts and native armor AC.
Constructed test mutations are explicitly mechanics fixtures, not observed kits.
@1 regression tests continue to cover the existing Warrior record.

Cemetery's current portable record parser/catalog only accepts @1. It must
explicitly qualify @2 and retain real low/mid-level captures before advertising
caster equipment cases as runnable. Description-only Cemetery dossiers do not
become executable from this schema's existence. Existing suite/evidence pins
remain immutable. Zoo pushid comparison is independently tracked in zoo #144.
