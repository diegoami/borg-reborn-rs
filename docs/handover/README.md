# Handover notes

One file per task (RULES 5b, #580), so that two open PRs never edit the same
lines. Older entries are in [../HANDOVER.md](../HANDOVER.md), the archive up
to 2026-11-18.

- **Name:** `YYYY-MM-DD-<issue>-<short-name>.md`: the day the PR opens, the
  task issue's number and rule 5's short name, for example
  `2026-11-18-580-handover-dir.md`. File names sort by date, so
  `ls -r docs/handover/` lists the newest first.
- **Content:** a `## YYYY-MM-DD: <title> (#issue)` heading, then what was
  done, what was found, what is left (each item an issue) and anything the
  owner has to decide. Keep the evidence's details in the PR, in
  `docs/CALIBRATION.md` or in borg-cemetery; the note points to them.
- **Changes:** a task changes only its own note, in later commits of the
  same PR. A later task that changes what an older note says writes its own
  note and says so there; it doesn't edit the old one.
