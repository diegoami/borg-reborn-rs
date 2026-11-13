# Decision fixtures

The scenario files behind [`docs/FIXTURES.md`](../../../docs/FIXTURES.md), the
T4.1 (#118) checks in [`../decision_fixtures.rs`](../decision_fixtures.rs).

Each JSON file is a constructed `Observation`, not a game recording. The
committed dossiers do not contain the original snapshots or traces, so the
fixture takes every number it can from the dossier (HP, depth, monster HP,
pack, the fatal action) and declares the rest in `construction` and `limits`.
No fixture claims a survival benefit or C-Borg parity.

## Fields

| Field | Meaning |
| --- | --- |
| `id`, `title` | Stable id and the case in one line. The id set is fixed in the test. |
| `dossier` | The directory or report the case comes from; the test checks it exists. |
| `citations` | The files and lines the facts come from. |
| `construction` | The invented part: room shape, stat line, constructed monster positions. |
| `limits` | What the fixture cannot claim. Required. |
| `verified` | For an `owner-manual-branch.json`, what the branch actually verified (turn range, end state). Required when cited. |
| `status` | `active` or `gap`. |
| `blocked_by` | For a gap: the open M4 issue that owns the missing action or assessment. |
| `player`, `equipment`, `inventory`, `map`, `monsters`, `objects` | The scenario. `map` is terrain (`#`, `.`, `<`, `>`, `+`, `,`, `?`, optional `@`); monsters and floor objects are placed by coordinate. |
| `forbidden` | The recorded decision the Borg must not make. An action name with an optional direction and mode. |
| `feasible` | The alternative the fixture requires (`active`) or records as missing (`gap`). |
| `current` | For a gap: the action and mode the Borg chooses today. The test pins it. |
| `pins_forbidden` | True when today's pinned action is itself the recorded fatal decision. |

## Active and gap

* **active** — the current action set can express the alternative. The test
  runs the fixture twice (same action, mode and world digest) and requires
  the decision to avoid every `forbidden` and meet one `feasible`.
* **gap** — the alternative needs an action or assessment M4 has not built.
  The test requires the decision to match `current`, and fails as soon as the
  decision changes, so a fixture cannot rot into a skip. When the missing
  capability lands, promote the fixture to `active`.
* `pins_forbidden` separates two kinds of gap: the current decision is
  already a different wrong action (false), or it is still the recorded
  fatal decision (true). A gap never asserts a rescue.

The test also enforces:

* the exact id set in `EXPECTED_IDS`, so a fixture cannot disappear quietly;
* every id appears in `docs/FIXTURES.md`;
* every dossier path exists and every cited owner branch has `verified` text;
* every gap names an owner in `OWNERS` (the open M4 tasks);
* every monster race, class and player race resolves in `knowledge.json`;
* every action name is in `ACTION_NAMES`, so a typo cannot silently become
  "unavailable";
* one decision is one action: the executor's key sequence for a walk or
  attack is a single direction key (the `mmx-war-1` breeder lesson).

`BORG_FIXTURE_PRINT=1 cargo test --test decision_fixtures print_current --
--nocapture` lists what every scenario decides today.

## The knowledge slice

`knowledge.json` is the engine's own static data, trimmed from a
`BORG_KNOWLEDGE_LOG` dump at the engine pin `e473720`. It keeps the monster
races the fixtures name, all nine classes, all player races, the stores, the
energy and stat tables, and the five reserve kinds. Do not edit values by
hand. To regenerate, run a game with `BORG_KNOWLEDGE_LOG=knowledge.json`
(`tools/rerun-death.py` already sets it) and take the same names.

## Adding a case

1. Read the dossier's `report.md` and `overall-assessment.md`; take the exact
   state and the fatal action. Use the owner branch's `events` text for a
   verified alternative.
2. Add a JSON file, a line in `docs/FIXTURES.md`, and the id in
   `EXPECTED_IDS`.
3. Run `BORG_FIXTURE_PRINT=1 … print_current` and set `current`/`status` from
   the real decision. If `current` is the fatal action, set `pins_forbidden`.
4. `cargo test --test decision_fixtures`, then `cargo fmt`, `cargo clippy
   --workspace --all-targets -- -D warnings` and `cargo test --workspace`.
