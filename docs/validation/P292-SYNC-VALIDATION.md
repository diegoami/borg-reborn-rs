# Main synchronization for owner-authorized promotion #292 (#298)

The owner instructed “you can merge 292 and continue” after the independent
report recommended acceptance. Subsequent #295/#296/#299 work is excluded.

Preserve main352cefeb (recorder consumer CMake dependency handling and guided
debugging) and its concurrent successor2bb9b72f (loop-levels). No P policy
change to either feature, recorder/engine pin update, or external write.
The reviewed errand fix remains unchanged. The original review is archived
unchanged in REVIEW-292-STRICT-ERRAND.md. Its base6acbf170 has the same tree
as main0ed0e5de. Qualification: old apply_line already parsed the complete
line before applying commands; the actual defect was warning/continuing
on invalid configuration instead of exiting. The accepted fix still holds.

The new CMake/shell bytes change build identity. Pre-loop combined source
c4a38683a164a71682184e28adfceff9bbecda1d4c5b24c0edec010c3d7415ac
passed full45/45 v12 play parity,295,464 classified trace rows, original
smoke3/3 and47 engine tests (1 environment skip). D2423–D2432 retain its
ten natural deaths. Disk exhaustion prevented the first fixture launch;
a fresh retry passed after clearing only regenerable Cargo caches. Raw
runs, linked game binaries, sources and dossiers were preserved. No failed
launch was called a death or silently included as a successful run.

Final combined source:
15289917658b91656e6bb91993ad8efa29eeff1874fce2ffe9cbe05ce3b2a47f.
Binary:791ce71d9869dfb56188b8e4bc2f73298b868f2886127a2e1819013cd59bdc7d.
Actual BUILD_ID git revision:a1515c47c20dea74f011796d2c303eb684ce5884.
Clean index/documentation commit4bf5a0d2 has identical build inputs; the
records retain the actual build identity. Upstream engine remains42cd156ac.

Final required Rust checks pass: formatting, Clippy -D warnings,
154+4+5 original tests and143 core tests (1 ignored). Script19/tool78 tests
pass. All50 engine tests pass with one local binding-regeneration environment
skip. Full45/45 default keys/messages/states match pinned v12 with295,464
contiguous classified trace rows; original smoke3/3 and death fixture pass.
Vendoring check passes. D2436–D2445 preserve ten final-build natural deaths;
both source-qualified temporary registries reconcile with none missing.

Recorder/loop PRs independently allocated9 IDs already used by the P
archive. Existing P dossiers are unchanged. P292-DOSSIER-MAP.json maps
all9 imported cases toD2417–D2422 andD2433–D2435, with exact original commits,
source/result fingerprints, unchanged identity/attempt objects and verified
file hashes. Original PR evidence is preserved without clobbering it.

Required task CI gates track-p integration. The subsequent main promotion
is specifically authorized by the owner; no population run is launched.
