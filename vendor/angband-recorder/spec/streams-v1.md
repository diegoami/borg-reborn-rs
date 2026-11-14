# angband-recorder-streams@1

This is the initial per-process directory format, not the planned branch-tree
container. UTF-8 JSON is newline terminated. Integer inputs preserve native
values; the v1 C ABI supports 64-bit Linux, Angband's 32-word RNG and terminal
geometry from 1 to 255 in each dimension.

`recording.json` identifies schema, exact host build, original seeding PID and
geometry `[width,height,key_size,soft_cursor,complex_input]`. Replay requires
matching schema/build/geometry and uses the recorded PID.

`inputs.jsonl` is ordered across all external call kinds. Each object has
`kind`: `poll` (the 15-field Input struct), `clock` (signed seconds), `file`
(presence and SHA-256), `exists` (boolean), or `bot` (turn, ordinal, type, key,
modifiers). Every poll is recorded, including empty polls and peek calls.
Replay validates call kinds and poll wait/take semantics, supplies external
results and checks the bot's recomputed keys. In a deterministic headless Borg
frontend, `BORG_TOOLKIT_HEADLESS` recomputes and checks polls as well, preserving
frontend side effects. It never supplies bot decisions or changes the RNG.

Each `file` event increments the read ordinal, starting at 1. Present files
are stored as `file-input-NNNNNN.bin`; absent files have no blob. Capture and
replay both open these immutable bytes. SHA-256 is checked before replay.
Read paths are not portable identities; event order is the v1 identity.

`frames.jsonl` contains observation-only command/wait/input/tick/exit frames:
index, turn, key ordinal, input-poll count, player vitals, full RNG, screen
size and cell deltas `[offset,codepoint,attribute]`. `full` resets the screen
at startup, resize and every hundred frames. Player-known inventory/spells
and separately labelled diagnostic state are host observations. `world` is
FNV-1a-64 of an engine save written by a forked child; tick frames omit it.
It is a fingerprint, not an uninterrupted continuation checkpoint. Hidden
diagnostic state is never supplied to a bot.

`messages.jsonl` contains frame, turn, type and text. Compatibility outputs
are `events.txt`, `clock.txt`, `pid.txt`, `actions.txt`, `file-inputs.txt`,
`geometry.json`, `native-messages.jsonl`, `combat.jsonl`, and
`record-complete.json`. They retain the existing borg-rs field layouts.

`complete.json` is written only after successful stream flush/sync and full
replay consumption. Input exhaustion, malformed events, action mismatch,
unread events or write errors fail the process. Incomplete output is retained
without a completion receipt. Verification compares input bytes, read blobs,
messages, frames and receipt. A host executable hash and local launch recipe
are stored separately in `launch.json` by the CLI; the executable itself is
not bundled. This initial local format is not a portability guarantee.

The next container version must implement the owner's tree and rolling-save
model in docs/OWNER-HANDOVER.md. Each node starts from its own save in a fresh
process. No promise of replaying an arbitrary suffix from a live-process save
is made. Slim Git recordings and interactive in-game navigation remain
separate feature work after extraction.
