# Credits and licences

The recorder originated in diegoami/better-borg-c, then was adapted in
 diegoami/borg-rs. This repository rewrites capture/replay in Rust and keeps
Angband structure access in a separate C adapter. It contains no Borg policy.

- better-borg-c `08a5cd0198b2a2a9ceb426d4cc4704fd5364d519`:
  `tools/native-replay.c`, `.h`, `native-observer.c` (forked snapshot).
- better-borg-c `85b6ba8e3748e061db1ff1206163c27272d2de31`:
  `tools/native-inventory.h`, `native-spells.h` (player-known observations).
- better-borg-c `a4084032485a7c7f29153d0b37ce63dede3e4d5a`:
  combat observations and the replay toolkit.
- borg-rs `2411e52f411d6984037a85cdb237bd54dd5b81cb`, branch
  `feature/recordings-with-branches`: engine hooks, PID capture, time linker
  wrapping, native messages, engine-only observations, terminal mapped state.
  The original provenance ledger is `tools/IMPORTED.json` at that commit.
- Vanilla Angband `42cd156ac9f95ddf4311e88dd7ad462292d6d1d6`:
  the context of the hook patch. Original file headers remain in the host
  engine; its copying notice is retained in docs/angband-copying.rst.

The donor recorder headers offer GPL-2.0-or-later or the Angband licence.
The Angband copying notice offers GPL version 2 or the Angband licence.
This crate and its new code are distributed under GPL-2.0-only (LICENSE).
The retained donor observation headers retain their original notices and
licence choices. Copyright and credit are not replaced by this extraction.
