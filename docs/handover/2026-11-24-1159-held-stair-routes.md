# 2026-11-24 Held down-stair targets

The rising hold now checks the projected danger at a known down-stair
destination when a stair route selects it. The weighted and unweighted
searches and the Recall reachability probe skip a held destination. A
committed stair flow that seeded a newly held destination is cancelled so
the caller can plan again. The existing departure check still evaluates
the hold on arrival. A return to stairs with no intent to leave can still
walk to a down staircase.

The new native fixture covers selection, cancellation, the end of the
hold, and an up-stair walk started by the loop breaker. The #1101 handoff
fixture asserts that caution reverses a dangerous exploration step, the
handoff continues, and the Borg queues descent with the shipped profile.
The historical tests retain their incident profiles. The inventory of
the individual pinned parameters is still open in #1101.

The DL5 and DL9 recordings remain needed for #1106 and #1121. Further
search-accounting policy is tracked in #1141.
