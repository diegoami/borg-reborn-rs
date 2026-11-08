# D0009: starvation after a full-pack munchkin pickup loop

[Owner feedback](https://github.com/diegoami/better-borg-c/issues/163#issuecomment-6717711058) · [Investigation235](https://github.com/diegoami/better-borg-c/issues/235) · [Full assessment](overall-assessment.md)

Actual C50/source1cd426af4848, Half-Troll Necromancer CL10/D5/maxD9, seed2817460426, starvation218620. Original record.json/build.json/native evidence unchanged; full original-engine rerun VERIFIED.

Last ration191708; Recall storedHome3279; no shopping after3699. At the major loop pack21+55quiver missiles consumes23/23slots, while the Borg treats the nominal last slot as free and repeatedly retries an unpickable Bronze Wand (764visible rejections). Weakness finally exits munchkin, then normal escape cyclesD5/D6 until starvation. Five unknown mushrooms remain; no food/Recall spell is learned.

Canonical source419 and consumer235 track the capacity defect; shared41/180/183/159 cover reserves, identification,Recall and return intent. Native mechanics and all alternatives are qualified in the full brainstorm; no fix, successful rescue or population benefit claimed. Owner review completion has not been inferred.
