# D0008: grey-mold retreat/re-exposure loop

[Owner feedback](https://github.com/diegoami/better-borg-c/issues/163#issuecomment-6714388294) · [Focused investigation212](https://github.com/diegoami/better-borg-c/issues/212) · [Overall assessment](overall-assessment.md)

Actual C46 source5c23742ced30, Human Priest CL1/depth1, seed1668846659, death10940. See unchanged record.json/build.json for immutable identity.

Original and stderr-only C diagnostic replay both VERIFIED. The priest repeatedly retreats onto the known upstairs then explores back into grey-mold melee, consumes three CLW, never attacks and leaves two Phase scrolls unused. Qualified diagnostics reproduce exception-mode cache entries being reused without danger evaluation in normal exploration, independently matching the related Rust report. Canonical source404 and consumer212 track the mechanism; source156 and shared159 remain separate contributing investigations.

Later consumer carry: source404’s narrow cache correction is merged as patch0049 (PR222). Product controls/regressions pass; a separately labeled corrected diagnostic reaches the turn cap, with divergent RNG. This does not establish a population survival benefit or resolve all escape-intent issues. Original C46 evidence and completed owner review remain unchanged; technical actions stay open.
