## 2026-11-20 — Town inventory loop after Fox Form (#775)

On post-#757 main (`09a7090650bf837702d5ab3345a7401532114cf8`), Druid
seed 934061548 reproduced the zoo key stream (`3fc3fabb…`). At turn 65,803
the inventory refresh queued `}`, `i`, `y`, `y` before the shape decision
queued `v`. The revert's `r` answered the uninscribe prompt and canceled
that command. The leftover `i` opened the inventory menu, which consumed
7,758 repetitions of the queued uninscribe and walk keys until an Escape
from `clock_wrap_reset` closed it.

The Borg now defers carried-item inscription cleanup until its normal
shape. Inventory observation still runs during Fox Form, and the next
normal refresh can clear the inscription. The native replay test checks
the reported start against the zoo's 1,000-key frozen-turn gate.

The pinned review prompt and before/after measurements are in the PR.
