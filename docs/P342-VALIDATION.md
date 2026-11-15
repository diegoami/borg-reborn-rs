# M13/main synchronization with reviewed P changes (#342)

The owner accepted #331 at `69b99484` and delegated promotion. GitHub
rejected that merge after main advanced through #332 to
`70b48cc10047f394114500eec6747e5fce486070`. This sync preserves M13's donor
integration and P's strict configuration reads. Source merges automatically;
only handover and generated archive indexes conflict. Both handover histories
are retained and indexes are regenerated from the complete archive.

Relative to current main, the only borg-core changes are the reviewed
shell reader and the parameter loader's use of that reader. All M13 modules,
defaults, retired parameters, engine adapters and pin remain from main.
Upstream stays `42cd156ac`. No additional donor refresh is performed.

The 44 incoming main dossiers are qualified with unchanged original records
and payloads. Their original directories are also retained byte for byte
under `docs/validation/m13/source-deaths`; P342-M13-DOSSIER-MAP.json maps
both locations. Existing legacy and qualified P packets are unchanged.
Donor control packets remain in their separate validation archive.

Clean measured revision: `69fa6a920d4b33e759cc5180aea383e68f0d578a`.
Source SHA-256:
`e1c2e4d9e818dffeb157fed57d7f2be1dc239de1a6f6a98ca224a57a6c2edb4a`.
Binary SHA-256:
`1c9d194481219c08fc8b875b43a8ec75b1b7748a78229c983757b831854a7902`.
Later evidence/document edits do not change game inputs.

Formatting, Clippy and required Rust tests pass: 156 + 4 + 5 original tests
and 321 core tests (four ignored). Vendor verification passes against the
M13 donor pin. All 60 engine tests pass with one local binding-tool skip,
with native zoo/upstream checks required. Script tests (19) and tools (87)
pass. CI runs the complete required checks separately.

All 45 full-suite default vendored-core play-log hash triples match the
accepted receiver-full evidence in M13-VALIDATION.json. Deterministic summary
fields match too; wall_seconds is excluded. The trace has 242,641 contiguous
classified rows. Original-core smoke matches all three references; the
extra-option save/load fixture passes. Three native policy-on class controls
(0, 1, 6) reconstruct the exact accepted borg.txt hashes and match all play
logs against M13 evidence. No policy replay corpus or new population
experiment is run; no improvement claim is made.

The first comparison incorrectly included wall_seconds and therefore failed
on elapsed time despite equal logs; the corrected deterministic comparison
passes. A first policy-control command used unsupported run.sh flags and
was rejected before launching a game. Fresh native controls use private
configurations matching the recorded hashes; those successful attempts do
not conceal the initial command failure. Inherited patch-file whitespace
warnings are retained rather than altering M13's accepted patch bytes.

Thirteen new natural deaths are exported with the actual combined build,
qualified and mapped in P342-VALIDATION-DOSSIER-MAP.json. Reconciliation finds
none missing or unrecorded. The archive now verifies 2,522 cases. Original
build identities are never relabelled. Canonical cemetery contents and
ownership are unchanged.

#331's independent review remains evidence for its exact old head. The sync
PR records a fresh combined pin and CI results; the owner-authorized
promotion and any required review disposition must identify that pin.
