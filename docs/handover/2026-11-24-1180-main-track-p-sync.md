## 2026-11-24: Sync main into track-p (#1180)

Main `b5d07d64` contained #1126 (ranged-only and immobile caution
resistances), the one main PR absent from track-p `7bf58639`. Its base was
main and no sync had carried it into track-p.

Main moved to `15a064f5` while the sync was in CI. The sync also includes
#1163/#1154, which acknowledges save-load messages so the Borg can start.

The merge applies #852 to the ranged-only list and the immobile caution
modes. The separate #1147 walk-away list still uses its own lookup and the
#931 weighted stair reach check still considers every immobile attacker.
Both #852 settings remain at their shipped values of zero. The calibration
registry preserves both branches' changes and cites the merged source.

RULES 1b assigns the sync to the session that merges into main and makes
the ancestry check part of P merges, promotion review pins and release Zoo
pins. Sync PRs retain main's history and use the usual CI and owner gate.
