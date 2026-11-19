## 2026-11-19: town less scary, by levels (#622)

Owner: a graded "town less scary" instead of the on/off
`caution_town_never_scary`.

- **`caution_town_scary_levels_less`** (compiled 0, **shipped 3**, owner: "N=3 sounds
  good"): a town
  monster on the scary list stops being scary N character levels sooner.
- **On 18 classes-18 starts:** N = 3 gives 0 deaths and 102 CL (main 0,
  99), N = 6 gives 1 and 99, N = 11 gives 2 and 98. Thefts rise with N (1,
  13, 16). With #619's kill on sight on top: 0/101, 1/102, 3/90. The deaths
  are early dungeon deaths, none in town.
- **Shipping 3 changes the shipped play,** so `smoke-borg-rs` is re-recorded
  in the same PR (#606).
- **Left:** zoo arms for N (borg-zoo#328 can take them once this and #619
  are merged); whether to keep `caution_town_never_scary` (#605) beside it.
