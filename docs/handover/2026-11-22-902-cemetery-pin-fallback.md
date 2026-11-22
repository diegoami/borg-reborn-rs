## 2026-11-22: Death tools fall back to a checkout at the pin (#902)

**Done.** `tools/cemetery_tools.py` `locate()` no longer stops at the
borg-cemetery beside the primary checkout. Without `BORG_CEMETERY_DIR` it
takes the first checkout whose native tools match `death-tools-pin.txt`:
the sibling, then `vendor/borg-cemetery` (the worktree's, then the primary
checkout's). When it skips one it prints one line, once per process for
each refused checkout, naming the checkout it uses and why. With none
matching, the error lists every refusal and names
`scripts/setup.sh`. An explicit `BORG_CEMETERY_DIR` that does not match is
still an error, never a fallback, and the byte-for-byte pin check is
unchanged.

`scripts/setup.sh` now keeps the primary checkout's `vendor/borg-cemetery`
at `death-tools-pin.txt` (not `pins.json`). A clean checkout follows the
pin without `--checkout`, except under `--strict`, which reports it and
fails as for every other pin. It is a full checkout (1.2 GB here, a
blob-filtered clone), because the exporter registers across all the
Cemetery's trees; it lives once in the primary checkout, shared by every
worktree. CI is unchanged: its action already checks out
`vendor/borg-cemetery` at the pin and sets `BORG_CEMETERY_DIR`.

**Found.** On 2026-11-22 the sibling checkout tracked Cemetery `main`
(`e4b6ad7b`), whose native tools differ from our pin `46978936` since
`0308f48d`. That stopped `scripts/rerecord-smoke-borg-rs.sh` (#874) and a
one-start `harness/run.sh` experiment. With this change, smoke plays
without `BORG_CEMETERY_DIR` on that machine: 3/3 identical to
`smoke-borg-rs`, through `vendor/borg-cemetery`.

**Left.**
- #794 (#793) still moves the pin to `0308f48d`. After it merges, a sibling
  on Cemetery `main` matches again, and `setup.sh` moves the vendor
  checkout on its next run.
- An export that lands in `vendor/borg-cemetery/deaths/` is a working copy:
  a death worth keeping goes to a real Cemetery branch
  (`tools/README.md`).

**For the owner.** Each machine needs one `bash scripts/setup.sh` (about
1.2 GB) to get the fallback.
