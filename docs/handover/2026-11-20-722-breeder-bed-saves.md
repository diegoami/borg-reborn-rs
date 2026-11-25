## 2026-11-20: the breeder test bed on real characters (#722, PR #741)

The synthetic build capture (`synthetic-build.c`) supports only Human Warriors with mundane items, and the caster version is PR #450. So the bed runs on **living saves** instead.

- **How to make a bed:**
  1. play a start to CL7 (`harness/run.sh OUT SEED RACE CLASS 150000 exact,menu,living-save -- -clev 7`);
  2. keep `OUT/final.save` if the character is alive and on a dungeon level;
  3. run with `save=NAME,start-monsters=NAME:COUNT[:DIST]+...` (`SAVE_ROOT` pointing at the saves).
- **The frontend's `-start-monsters`** works after `-load`. It places the monsters after the replay seed (deterministic), only in the dungeon, and only for an exact monster name (`lookup_monster`'s substring fallback is refused).
  - `summary.json` records `start_monsters`, `start_monsters_requested` and `start_monsters_placed`.
  - `run.sh` warns when fewer were placed than requested, and checks the frontend's bounds (COUNT 1..100, DIST 1..40).
  - The `start_monsters` pair now comes right after `loaded` in `summary.json`, also for synthetic runs (keys unchanged).
- **The first bed**: 15 CL4–7 characters × worms and lice × breeder policies -1, 3, 4, 5. Results on #722:
  - 0 deaths in 120 games;
  - policy 3 kills the most breeders, 4 and 5 add nothing;
  - the lost-level Recall works end to end.
- **Left:**
  - The saves are local (the session's scratch directory). A fixed set of bed saves would be a borg-cemetery scenario package if the owner wants the bed repeatable.
  - "The bed needs a captured build (#382)" in the #722 policy note is superseded by this.
