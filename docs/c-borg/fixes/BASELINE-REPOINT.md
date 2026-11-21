# Baseline re-point: drift-group matrix baselines to product logs (2026-11-14)

Closes the follow-up owed on checkpoint 1 and the milestone-13 review receipt: the six groups
whose matrix baselines pointed at logs their carried layer can no longer reproduce byte-for-byte.

## What was re-pointed

For each group the product-run logs from the matrix at `5cefae071` (the state after the knockout
path-mask, `src/` identical to `90658755a`) are committed as `product-after-<mode>.log`
(knockout: `product-<mode>.log`) and the group's `baseline_log` registration points at them. The
source lanes' captured logs stay in each kit directory unchanged, as provenance.

| Group | Drift against the captures | Now |
| --- | --- | --- |
| dangerfull-69 | one `ONEKILL adj_partial` diagnostic line (4 cases) | byte-identical to the product baseline |
| teleotier-176 | one `grid_danger` diagnostic line (4 cases) | byte-identical |
| wandnames-38 | confusion/fear bolt VALUE −6 → −5 (2 of 5 cases) | byte-identical |
| healamount-592 | flee-reason TRACK line from the carried source156 (1 case) | byte-identical |
| deepdescent-590 | same source156 flee-reason TRACK line (2 cases) | byte-identical |
| knockout | `PROFILE %s` printed the absolute per-case profile path (4 cases) | fixture now prints the basename (path masked) and the group is byte-identical |

The knockout path mask (`5cefae071`) is the only fixture change: after the M13-F1 portability
correction the printed path was the runner's per-case directory, which changes with every run and
machine. No decision code, input or assertion changed anywhere.

## Receipt

Full matrix at this tip: **118 groups / 365 cases, `ok: true`, zero failed cases, 313/365
byte-identical** (`artifacts/m13-repoint2/regression-summary.json`). The 12 remaining byte
differences are the long-standing **declared intentional transitions**, not drift: dispel-248
`living` and vampire-257 `near-*`/`near-near` rows (necrolight-296, milestone 9), dispel259
`reserve`/`clear` (losdispel-326, milestone 6), fleeheal-408 (improvement I-002; its baselines
are the before-build runs by design), and preparation (2 documented painmsg shifts). The drift
debt recorded on checkpoint 1 and in the milestone-13 documents is cleared.
