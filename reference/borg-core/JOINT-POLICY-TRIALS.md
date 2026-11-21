# Coming home and preparing for the next trip

The aim is for the character to come home when there is something worthwhile
to do with its inventory, then leave with suitable supplies. Going deeper
faster is not the goal.

## What has changed?

There are now two connected decisions:

1. **In the dungeon:** should the character keep exploring, go deeper, move
   up a level, or return to town?
2. **In town:** what should it keep, sell, put in the Home, take out of the
   Home, or buy for the next trip?

Both now affect actual play. The shopping changes are implemented: the
character really buys, sells and moves items. They remain experimental and
are not switched on by default.

## The choice closest to your request: come home earlier

We tested an **earlier-return** option that:

- Returns while there are still **four inventory spaces free**.
- Returns with **two unidentified equipment or device stacks**.
- Aims **two dungeon levels shallower** than the existing readiness check
  says the character could handle.

It keeps the ordinary shopping priorities. This lets us inspect earlier
returns without also changing how much the character wants to spend.

A stack counts once: twenty arrows together are one stack, not twenty.
Currently, unknown potions and scrolls do **not** count toward the separate
unidentified-gear trigger. They still take up inventory space. That
exclusion is provisional; we have not established that it is the right
choice for your policy.

## The shopping choices are tradeoffs

**Keep more gold:** spend less readily, leaving money for later opportunities.
That can also mean taking fewer supplies.

**Emphasise learning:** give more value to buying an unfamiliar item to learn
what it is. That competes with food, healing and keeping money.

**Prepare for a longer trip:** tell the shopper to provision for a longer
outing. It buys more supplies. This does not tell the character to stay in
the dungeon longer.

Here is one concrete example, from the same Warrior's first shopping trip:

| Shopping choice | Gold left | Food carried | Healing potions carried |
|---|---:|---:|---:|
| Ordinary priorities | 41 | 2 rations | 2 |
| Keep more gold | 61 | 2 rations | 1 |
| Prepare for a longer trip | 10 | 5 rations | 4 |

So “keep more gold” is not automatically safer: this character saves 20 gold
by taking one fewer healing potion. “Prepare for a longer trip” leaves it
better stocked but with little cash. Which is sensible depends on the trip
we actually want it to make.

## What did the experiments establish?

We tried 24 games with a 100,000-turn limit, using different settings and
starting cases. We
verified actual purchases, sales and Home transfers. We also fixed two
execution bugs: repeatedly offering items a shop would not buy, and trying
to buy an item using an incorrect price.

**We have not established a best set of choices.** Some characters using the
new policies died. These few cases do not show that the new behaviour is
safer than the old behaviour.

Freeing inventory space and learning an item's identity are also different
outcomes. Putting an unknown staff in the Home frees a space, but the staff
is still unknown. The results keep those outcomes separate.

For your original concern, the directly relevant option to inspect is
**earlier returns with ordinary shopping priorities**. The other shopping
choices can then be considered separately. No new default has been chosen.

## Further detail, if needed

- [Exact settings, test results and limitations](validation/2026-11-14-trip-policy/live-integration-technical.md)
- [Recorded numbers for every run](validation/2026-11-14-trip-policy/joint-live-results.json)
