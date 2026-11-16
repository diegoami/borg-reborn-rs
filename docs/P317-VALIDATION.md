# Strict borg.txt file reads (#317)

Only a missing file means settings are absent. Other I/O errors and invalid
UTF-8 stop configuration with exit status 2 and a path/error diagnostic.
The parameter loader shares this reader. Unix directory paths preserve
OS bytes. Valid files, missing-file defaults and lazy environment priority
retain their behavior.

Fear and errand settings are reread on command entry; core selection and
parameters remain process-cached. Reader subprocess tests cover a valid
file becoming invalid. Native startup tests cover both cores, missing and
valid files, invalid UTF-8, directory read errors, environment overrides,
parameter loading and non-UTF-8 directory paths. Live interactive GUI
re-entry and automatic save recovery were not tested or implemented.
An error encountered on re-entry terminates without an automatic save.

The measured clean build used revision
`754593a033487f8f28f48ca9e90856fdffa561c3`.
Source SHA-256:
`69c447072d3822bef7dc9f9998cadfc113452ff406858f98c81bd0f44a567726`.
Binary SHA-256:
`eac6ca5bbb26517a14fe901831c84db0fe37abdaa3212ea8ca48ecde2add1f07`.
Upstream remains `42cd156ac`. Later evidence/document edits do not change
build inputs.

Required formatting, Clippy and Rust tests pass: 156 + 4 + 5 original-core
checks and 146 core tests, with one ignored. All 59 engine tests pass,
with one local binding-regeneration skip. Script tests (19) and tool tests
(78) pass. The vendored-file check passes. The pinned zoo contract and
upstream checks are required in this native validation.

All 45 default vendored-core cases match the immutable v12 reference in
keys, messages and states; 295,464 contiguous classified trace rows pass.
Original-core smoke (three cases) and its death fixture pass. These checks
preserve default play, rather than claiming a performance improvement.

Ten measured natural deaths are retained as D2476–D2485 with the actual
source and build identity. Reconciliation finds none missing or unexported.
The registry is explicitly historical/isolated; the canonical cemetery is
untouched. #318 will qualify these provisional IDs at the import boundary
and retain the original records and payload bytes.

#307 was promoted with owner authorization. This follow-up requires its
own green task CI into track-p and a separate milestone promotion review.
