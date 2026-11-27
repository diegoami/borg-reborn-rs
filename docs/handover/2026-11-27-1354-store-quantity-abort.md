## 2026-11-27: store quantity queries no longer abort the game (#1354)

Owner, 2026-11-27, from an audit of the wrapper against borg-rs.

**What was done:** `borg-core/src/engine.rs`: `with_object_quantity` returns
`None` for a quantity the native stack does not hold, instead of an
`assert!` (release builds abort on a panic). Its callers answer "no":
`store_accepts_quantity` false, `purchase_quote_quantity` `i32::MAX`
(unaffordable, so the purchase search tries fewer copies), and both
`sale_quote`s 0 (every caller reads > 0 as a sale worth making). Unit test
`quantity_in_stack_tests`.

**Found:** the quantities come from the Borg's own model (`item.iqty`,
`excess_copies`, remembered store stock) in `borg_town_live.rs`,
`borg_town_shadow.rs` and the loadout modules; all of it is live with the
shipped `borg_town_plan = 1`. borg-rs has no such query.

**Checked:** CI's cargo commands; smoke on the default core identical to
`smoke-borg-rs` (see the PR). Not checked: a game with a stale stack count
(none is known; this removes a crash path, it changes no correct answer).

**Left:** nothing.
