## 2026-11-25: Remove the sandal-specific junking setting (#1265)

### What changed

Removed the Leather Sandals exception and its threshold. The generic comparison still applies to fully identified spares. The parser accepts the retired setting and reports that it was ignored.

### Findings and verification

The review found that the first regression covered the environment override. The native fixture now runs separately with the setting in an old `borg.txt` and with the environment override. Each run checks the diagnostic and shows that the Borg retains the unknown-rune spare. The native town fixture checks that both selling choosers still sell identified inferior spares.

### Remaining work

#1264 tracks the equipment redesign. Dungeon priority disposal requires an identified same-kind pair and caps the whole stack at 15 gold. Town arrival can discard some inferior gear with unknown runes. The full-pack fallback can consume items to make space. Its generic desperate-pass bonus overwrites earlier bonuses for unknown boots, weapons held while digging, and unknown runes.

### Owner decision

The removal needs no further gameplay decision.
