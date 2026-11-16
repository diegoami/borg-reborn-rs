# angband-recorder-streams@2

Supersedes [streams-v1](streams-v1.md), which remains the record of the
original format. V2 is what implementation `SCHEMA = "angband-recorder-streams@2"`
writes and requires. UTF-8 JSON is newline terminated. Integer inputs preserve
native values; the v2 C ABI supports 64-bit Linux, Angband's 32-word RNG and
terminal geometry from 1 to 255 in each dimension, key queues of 1 to 65535
keys.

## Identity

`recording.json` is `{schema, build, pid, geometry:[width,height,key_size,soft_cursor,complex_input]}`.
`schema` is exactly `angband-recorder-streams@2`. `build` is the host build
label **plus** the SHA-256 of the executing image (`label:sha256:<hex>`),
written by the recorder itself from `/proc/self/exe`; a replay refuses a
different executable even when the label matches. `pid` is the recorded RNG
seeding PID, 1 to 2147483647. Replay requires the same schema, build and
geometry.

## Mandatory files

Every recording contains exactly these files plus one blob per present
captured read and, when a zone file was sealed, `timezone.tzif`:

`environment.json`, `recording.json`, `inputs.jsonl`, `frames.jsonl`,
`messages.jsonl`, `complete.json`, `events.txt`, `clock.txt`, `pid.txt`,
`actions.txt`, `file-inputs.txt`, `geometry.json`, `native-messages.jsonl`,
`combat.jsonl`, `record-complete.json`.

Verification rejects missing, extra or non-regular entries against the
expected set derived from the streams; `messages.jsonl` and
`native-messages.jsonl` must be byte-identical aliases, and the
compatibility text streams must match the events that produced them.

## Environment sealing

`environment.json` is
`{variables: {name: value}, timezone: {kind: File, sha256} | {kind: Posix, spec}}`,
with `deny_unknown_fields`. Captured variables: `TZ`, `TZDIR`,
`PARSE_ERROR_LIMIT`, `LANG`, `LC_ALL`, `LC_CTYPE`, `BORG_TOOLKIT_HEADLESS`,
`BORG_VISIBLE_SEED`, `BORG_NO_EXTRA_OBSERVATIONS`, `BORG_REPLAY_SYS`,
`BORG_REPLAY_WIDTH/HEIGHT/KEY_SIZE/SOFT_CURSOR/COMPLEX_INPUT`, and every
`ANGBAND_X11_*` variable. Replay removes these names from the ambient
environment and reinstates the captured values before the first engine code
runs, then calls `tzset()`. A named zone is resolved through the captured
`TZDIR` (default `/usr/share/zoneinfo`); when the zone file is found its
bytes are sealed as `timezone.tzif` and its SHA-256 recorded; otherwise the
POSIX `TZ` specification is stored and must not contain `/`, NUL or an
ambient-resolvable `timezone.tzif` entry. `PARSE_ERROR_LIMIT` and the other
variables are behavior inputs like any other and are restored verbatim.

## Inputs

`inputs.jsonl` is ordered across all external call kinds; each object has
`kind` with `deny_unknown_fields`:

- `poll` — the 15-field native terminal poll. Domains: `wait`, `take`,
  `result`, `disconnect`, `mapped` ∈ {0,1}; `event_type` ∈ the engine's
  event mask values {0,1,2,4,8,16,32,64,128,256}; `code` 0..2^32-1;
  `mods` 0..31; `x`,`y`,`button` 0..255; `0 <= head,tail < key_size`;
  `width`,`height` 1..255; mouse fields are zero unless the event is a
  mouse event; a successful poll (`result == 1`) carries no key. Values
  outside a native domain are rejected before any C narrowing can occur.
- `clock` — signed `time()` seconds.
- `file` — presence and SHA-256 of a read-time captured file; present files
  are stored as `file-input-NNNNNN.bin` and absent files must have neither
  blob nor digest.
- `exists` — boolean existence result. Existence checks of the session's
  own savefile during save are live session state (stat()ed directly in
  both record and replay), not recorded inputs.
- `newer` — boolean `file_newer` result (v2: recovery decisions are inputs).
- `directory_open` / `directory_read` — save-menu enumeration results;
  `directory_read` carries `only_files`, a 1..65536 `capacity` and the
  optional entry name (no NUL, shorter than the capacity).
- `terminal_init` — the `[width,height,key_size,soft_cursor,complex_input]`
  the frontend actually initialized with; must equal the recording geometry.
- `bot` — turn, ordinal, type, key, modifiers; type in the event mask
  domain, modifiers 0..31. Replay recomputes and checks the bot's keys and
  never supplies them.

Replay validates call order and poll wait/take semantics; input exhaustion,
divergence, malformed events and unread events fail without a completion
receipt. In a deterministic headless frontend `BORG_TOOLKIT_HEADLESS`
additionally recomputes polls.

## Observations

`frames.jsonl` — command/wait/input/tick/exit frames with index, turn, key
ordinal, input-poll count, vitals, full RNG, screen size and cell deltas;
full resets at startup, resize and every hundred frames. `messages.jsonl`
(frame, turn, type, text) and `combat.jsonl` are observation streams. Host
extensions may add keys but must be JSON objects and may not replace core
observation keys. `world` is the FNV-1a-64 fingerprint of a forked child's
save and is a fingerprint only.

## Receipts and verification

`complete.json` and `record-complete.json` are hard-linked receipts
`{inputs, frames}` written only after flush, sync and full replay
consumption; `complete.json.inputs` counts all events, `record-complete.json.inputs`
counts polls, and both frame counters must match the stream length. A failed
session removes its receipts and retains incomplete output without them.
`verify SOURCE REPLAY` validates both directories structurally, compares
the file sets and every compared file byte for byte. `replay --verify`
additionally re-executes the exact recorded executable: the CLI checks
`launch.json`'s executable hash, validates the source, and runs the game
with all write directories (`user`, `save`, `panic`, `archive`, `scores`)
redirected into the replay output's `profile/`, placed **before** every
engine argument so `-u` resolves against the private save directory.

This format still has no portability guarantee; the branch-tree container
remains future work as described in [streams-v1](streams-v1.md).
