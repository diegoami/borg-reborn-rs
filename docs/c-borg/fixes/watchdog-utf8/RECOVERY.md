# Raw evidence recovery

`archive-receipt.json` identifies the persistent ignored archive and its full
SHA256/member verification. `artifacts/watchdog-utf8/manifest.json` lists all
114 member hashes. Raw evidence is local; no hosted upload is claimed.

The tar contains seven complete run directories, the immutable pristine/M2
build identities and binaries/source archives, before/fixed runner bytes,
concurrent cohort driver, unit-test logs, exact native UTF-8 prefix, and local
native-gate result/logs. All recorded run-file and binary hashes were checked;
all archive members were verified by reading the completed tar back.

Extract into a fresh directory for inspection. Rebuild through the report's
pinned source/harness commands rather than assuming a Linux binary is portable.
The archive's before runner is used only for the failing control; the merged
harness pin on issue220 is the zoo's production entry point. The zoo catalogue
and its original failed cell remain unchanged.
