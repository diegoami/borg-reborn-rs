## 2026-11-22: The urgent town purchase waits for weight room (#955)

**Done.** The live selector (`first_executable_action`,
borg-core/src/borg_town_record.rs) took an urgent purchase (the dive
gate's Word of Recall, #859, and the other urgent needs) before any
disposition, checking only pack slots. It now also needs one copy to fit
the weight still free before the encumbrance starts (`Facts::allowance`,
`BI_CARRY / 2 - BI_WEIGHT`, passed in by borg_town_shadow.rs). When it does
not fit, the room-making dispositions the solver already planned (the
budget's weight is the allowance less the urgent picks) run first, and the
purchase follows on a later solve; with no room to make, the visit leaves
rather than going over. Historical replay (`first_action`) is unchanged.
New test: `urgent_buy_over_the_weight_allowance_frees_weight_first`.

**Found.** Codex's P7 review on #939 (finding P2, "urgent Recall purchase
can exceed the carry allowance").

**Left.** Nothing for this finding. #956 (urgent priority) is separate.
