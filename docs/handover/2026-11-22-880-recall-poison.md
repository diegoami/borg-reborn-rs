# 2026-11-22 — #880: poison, status blows and breeders no longer veto flight Recall

**Issue:** diegoami/borg-reborn-rs#880. **Branch:** `task/880-recall-poison`
(from `track-p` 7e21967b; owner, 2026-11-22). Track P.

## What was done

New parameter `caution_recall_blow_model` (compiled 0, shipped 1 in
`harness/borg.txt`). At 1, `flight_recall_survivable` (borg_caution.rs):

- prices every blow by its dice (already summed for every blow, at every
  monster turn of the delay); a status blow is no longer an "unmodelled risk",
  because no status stops the Recall countdown (game-world.c:709-714);
- adds POISON's timer: one HP per world tick (game-world.c:513-515) from the
  earliest tick a reachable poisoner can strike to the end of the delay, at
  most the 5 + rlev ticks per action the poisoners can add (the timer stacks);
  the whole delay if already poisoned, nothing with resist poison;
- keeps SHATTER (earthquake) and ranged spells as unmodelled risks;
- waives a breeder's (RF_MULTIPLY) unmodelled risk (future-proofing: no breeder
  at 574b618c has a dangerous spell or SHATTER; POISON is what fixed the worm);
- caps contact HP at the k fastest attackers for the whole delay, k = the
  Borg's neighbours that are not known rubble or wall, and scales RAND_25/RAND_50
  monsters by their attack share (a worm mass: 0.61).

Review (MiniMax-M3, on 8e804eb8) findings 1-6 were addressed in the next
commit; 7 (ship 0 or 1), 8 (attribution: PR footer removed, published commit
trailers stay) and 9 (branch name) are reported to the owner. The re-review
(on 5a8404d0) found rubble counted as open; fixed (rubble is not PASSABLE).

The fight certificate (`winning_fight`, `unknown_fight`/`unknown_damage`)
is unchanged. A note `# Flight Recall forecast: … (#880)` names the cap, the poison HP and
any waived breeders, only the parts that apply.
Param citations shifted by the insertion were refreshed with
`scripts/check-param-citations.py --fix`.

## Owner decisions / open

- "Low damage should not veto" is read as: a blow whose only extra effect is a
  status costs its dice and is not a veto. A numeric low-damage threshold for
  *spell* risks was not added; say if that was meant.
- Breeding during the delay is not priced. Modelled HP can still refuse
  Recall where the open neighbours times the delay exceeds HP. The priest's
  turn-64022 state (#876) was not replayed against the new forecast.
- The issue's optional rule (Recall on a losing breeder fight with no stairs
  reachable) and the low-gold Recall re-buy question are not done.
- A hypothesis until borg-zoo measures it (RULES 14): a zoo request comparing
  `caution_recall_blow_model` 0 against 1 is the next step.
