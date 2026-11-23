## 2026-11-22: The urgent Recall comes before cheaper urgent needs (#956)

**Done.** The urgent pass (`urgent_pass`, borg-core/src/borg_town_budget.rs)
chose the purchase set by most urgent needs met, then least gold. With 140
gold, a 125-gold Recall and a 20-gold ration both urgent, it bought the
ration and left the Recall 5 gold short, against `town_recall_urgent`'s
"bought first, before anything is weighed" (harness/borg.txt). When the
urgent Recall (`Need::Trip`, urgent only through `town_recall_urgent` at the
dive gate) is among the needs, a set that covers it now comes first; the
count of needs and the cost decide after that, as before. Without the
Recall among the needs nothing changes. New test:
`an_urgent_recall_is_not_starved_by_a_cheaper_urgent_need`.

**Found.** Codex's P7 review on #939 (finding P2, "cheaper urgent needs can
starve Recall at the DL5 gate").

**For the owner.** The same review notes that below the gate the
`buy_recall_from_gold = 200` floor, applied after the purchase, keeps a
125-gold scroll out of reach until the purse holds 325 gold. That is the
shipped setting working as written, not changed here; say if it should be
lower.

The review of #961 found the purchase order inside the chosen set still
followed the candidate order; `Budget::build` now lists the urgent Recall
first, so the live selector buys it first.

The same review notes the trade-off this makes: with 140 gold, a 125-gold
urgent Recall and a 20-gold urgent ration or light, the Recall is bought
and the ration or light is left unmet, where the old order bought the
cheaper item. That follows `town_recall_urgent`'s "bought first"; say if a
basic supply should come first instead.

**Left.** Nothing for this finding.
