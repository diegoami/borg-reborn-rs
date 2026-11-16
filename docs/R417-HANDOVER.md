# R3 review corrections (#417)

Claude Code reviewed immutable snapshot d5d75cba in #413 and recommends
accepting the code, with evidence acceptance held until cemetery PR #14
merges. This is a reviewer recommendation, not owner acceptance. The original
review snapshot remains fixed; R4 and main/global-reviewed promotion stay gated.

- Cemetery #14 is merged on main at
  99de780bbd80086bdd8b38524ecf230c3e657573. Both source receipts now pin that
  canonical containing commit; their manifest hashes and all original bytes
  remain unchanged. The original cbb23b5c commit is its ancestor. This is not
  final numeric-ID allocation; #344 remains open.
- R3-EXIT-REPORT.md explicitly calls this a mechanical policy migration.
  Source correspondence is deliberate. Shadow tests validate accessors,
  adapters, complete-state and tape integration on covered inputs, not a new
  independently designed strategy. Literal-fault controls show why runtime
  agreement is not guaranteed merely by source correspondence.
- The vendor gate checks exact donor blob IDs and receiving SHA256 hashes for
  every hooked donor file, with the manifest outside build inputs. Check and
  refresh never regenerate it. Explicit record-hooks must be reviewed together
  with the source delta. CI now fetches the exact donor and runs the real gate.
- CI validates expected check/switch arm modes, including every stats row;
  wrong-arm or mid-run mode changes reject. Historical explicit readers stay.
- The gameplay-evidence policy section is copied from main without carrying
  newer Track P runtime changes. The exact pinned zoo profile audit finds
  24/75 profiles with non-inert retired-mode settings; #419 gates later
  operational promotion. Legacy profiles remain intact for older specimens.
- Sparse fixture/positive-utility limits and inherited edge risks are explicit;
  #418 tracks stronger scenarios and low-priority boundary/diagnostic work.
  Current scope does not alter donor policy or claim complete branch coverage.

Local checks pass: 8 vendor tests, 42 source tests and 105 tooling tests. The initial tool test
attempt lacked the pinned cemetery dependency; after adding its unchanged
33c37552 checkout all 105 pass. No measured game ran for that setup error.
The repository source-refresh report identifies newer upstream attack/trip
work; pins remain unchanged and any carry is a separate reviewed task.

No Rust, engine, Cargo or policy input changes belong to this fix. Runtime
input stays 562aedf968831885c32b4f56271a2f7bd0245432aff9159dd629f59601ff4fe1.
Finish exact published-receipt verification and required CI, merge the task
into feature/track-r only, then publish a new immutable review snapshot with
a disposition reply on #413. Never move the old snapshot or treat the
recommendation as owner acceptance. Owner selects/runs any re-review.
