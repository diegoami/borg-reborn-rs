## 2026-11-27: A restock trip home is the core's own errand (#1326)

Stacked on #1334 (#1329), whose `probe`/`prepared_probe` it uses.

### What was found

`docs/ERRANDS.md` says a trip home to restock is not fear and the plan
waits. `scared_home` could not see that case: it called a rise the core's
own only while `borg_prepared(depth)` passed, and `borg_prepared` checks
`borg_restock` first, while the core rises for a restock only when its
current depth fails. Every restock trip read as fear: a ladder trip ended,
and `/dive-exploring`, `/explore-level` or `/explore-back` aborted with
"the core wants town". There was a second way to the same result: in the
first 100 game turns on a level the core climbs for a restock without
setting `rising` (`borg_must_return_to_town` waits for the level timer), and
the errand read the climb as "rose from DLn, not fit".

### What was done

- `scared_home`: a restock due at the current depth (`borg_restock`, no
  level timer) or the "Collect from house" reason makes the trip the core's
  own errand (`Home::Errand`, the plan waits), whether `rising` is set yet
  or not. Fleeing to town and any other unprepared depth stay fear.
- `before_dungeon_think`: a climb off a level where a restock is due counts
  as homeward, like `rising`: the level stays in the plan and is not "unfit".
- `docs/ERRANDS.md`: the fear rule names the restock exception.
- Unit test `a_restock_trip_home_is_the_cores_own_errand` on the classifier.

### Checks

CI's four cargo commands. No game was run: errands are not in the smoke
suite, and with no errand every hook returns at once, so `smoke-borg-rs`
is unaffected.

### What is left

The rest of the audit: #1327, #1328, #1330, #1333; owner decisions #1331,
#1332.

### Owner decisions

One level up the restock may no longer be due (the bands are by depth), so
the errand, wanting the deeper level, is refused there ("not prepared for
DLn"): a ladder trip goes home, which is where the restock happens, and a
finite command stops, as "no level is forced" says. Should a finite command
instead go home to restock and resume? That would be a new rule; nothing is
changed for it here.
