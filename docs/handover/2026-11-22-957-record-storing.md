## 2026-11-22: Town plan records carry the storing settings (#957)

**Done.** The dispositions (`borg_town_utility::dispositions`) and the
purchases' departure room read `town_departure_free_weight`,
`town_departure_free_slots`, `town_store_keep_min` and
`town_store_only_overweight` from process-wide parameters, and the plan
record (schema 12) stored none of them, so a replay with other settings
stored differently from the same record. Schema 13 adds `storing`
(`[free weight, free slots, keep_min, only_overweight]`,
`Storing::to_array`). A live solve records the settings it used; a replay
of a v13 record uses the record's; older records (v4-v12) keep reading
the process's settings, as before. `town_store_only_overweight` (#711) is
included because it has the same gap. New test:
`the_storing_settings_travel_with_a_v13_record`.

**Found.** Codex's P7 review on #939 (finding P2, "town plan records omit
the inputs needed to replay departure-room decisions").

**Left.** Nothing for this finding. Play is unchanged: only what the
record writes.
