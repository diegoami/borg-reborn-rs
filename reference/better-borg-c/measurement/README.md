# Native Borg measurement contract

Owner-authorised work is tracked in consumer #3. Source defects remain on
neo-angband-tracker/checkpoint207; the first carry is consumer #4/source #82.

Accumulated [milestone2 review package](IMPACT-MILESTONE2.md) links all seven
carried corrections, [paired deep results](DEEP-IMPACT-V2.md) and
[versioned death dossiers](../deaths/README.md), including unfavorable outcomes
and replay limits. External review remains pending; bugs take priority.
The subsequent bug-fix wave is collected in
[milestone3](https://github.com/diegoami/better-borg-c/milestone/3) and
[IMPACT-MILESTONE3.md](IMPACT-MILESTONE3.md): 27 more verified carries
(patches 0008-0034), their interactions, process/tooling context and review
questions. Owner-led accumulated review completed on 2026-11-07: milestone2
approved, milestone3 approved with evidence-durability/metadata findings;
follow-up issue130 records the corrections. This is not an upstream
incorporation approval.
Operational lessons from the 2026-11-06/07 carry wave are recorded in
[LESSONS-LEARNED.md](LESSONS-LEARNED.md); they refine verification practice
without changing any gate, fixture, assertion or baseline.

The immutable unmodified baseline is Angband commit
`e473720ed88a0f0c7ca66815cef3000eb4a38bc3`. `prepare` exports an exact Git archive,
records its SHA256 and builds a separate tree. The archive is never patched.
Source exports, binaries, saves and run directories belong under ignored
`artifacts/`, never in Git. An uninstrumented native build is retained separately.

`tools/measure-native.py` reuses the native test frontend, ordinary UI character
birth, ordinary Borg activation (`Ctrl-z`, `z`), production game loop and RNG.
The fixed initial character is Human Warrior, default point buy and ordinary
birth options. The supplied Borg configuration disables cheat death and winner
respawn; other settings retain native defaults. No favorable intervention,
modified damage or observation-driven gameplay input is supplied.

The disposable observer build is explicitly **instrumented**, not byte-identical
to the pristine binary. The driver applies checked single-site insertions to an
export: initialise the normal WELL RNG before `Rand_init` (which then does not
reseed), count calls to actual `borg_think`, observe command boundaries and world
turn increments, log existing Borg notes and stop at the cap or termination.
No extra random draws, substituted decisions or changed game rules. Borg seeds
its own RNG through its existing initialisation draw. Both RNG states are logged.
Snapshot saves run in a POSIX fork child: native saving compacts the monster
list, so it must never execute in the live measurement process. The child writes
evidence then exits; parent memory, RNG state and monster order are unchanged.
Snapshots and logging are evidence operations; paired `--no-saves` runs
check for save interference. Generated snapshot build IDs explicitly name the
source pin and observer. A run records build/source/tool/data/binary hashes,
options, configuration, seed, command/environment, limits and wall time.

A game `turn` is the engine world counter (`game-world.c`), not an action count.
`energy` is `player->total_energy`; `decisions` counts `borg_think` invocations,
including calls returning false in its internal loop. These are separate from
monotonic wall seconds. The cap is an absolute engine turn (birth normally starts
at turn1), checked immediately after increment; it may interrupt a multi-turn
rest. Turn-cap means alive at that boundary, not completion.

Normal results are death, successful completion and turn-cap survival. Signals
are crashes; missing/invalid/nonzero completion is a harness failure. A wall
limit is `wall_timeout`, never death/survival. The separate stall watchdog reports
`stalled` after its declared interval without observed turn progress, preserving
the last state; this is an operational observation requiring investigation, not
a proven gameplay loop. Limits are frozen with the cohort. Every attempted run
keeps a result, input, terminal log, trace and hashes; failures remain in totals.

The observer captures native Borg notes, recent native messages, final state and
the native Borg map/inventory report. It saves a birth snapshot and a recent
living checkpoint at command boundaries at least1000 world turns apart when
commands are available. It does not promise checkpoints during long unbroken
rests, nor after a crash. Native `close_game()` can write a dead-character save,
but native savers omit dungeon/objects/monsters when `player->is_dead`; loading
that file is not resurrection. The observer stops before interactive death UI,
preserving terminal evidence plus the most recent living checkpoint instead.
A checkpoint does not store Borg's complete in-memory knowledge. Loading it is
for diagnosis; exact continuation is not claimed. Recreate from the frozen seed,
configuration and source/tool pins for a trajectory reproduction.

Do not enlarge or change a cohort after seeing outcomes. Repeated runs must
match the decision trace and both RNG states before a seed is called reproducible.
If seed repetition fails, freeze a hashed start save plus required state instead.
Paired baseline/fixed starts do not guarantee identical later encounters: changed
decisions can change subsequent random draw consumption.

Copyable entry (from repository root, native prerequisites in README):

```sh
flock /tmp/angband-validation.lock python3 tools/measure-native.py prepare --output artifacts/measurement/pristine
flock /tmp/angband-validation.lock python3 tools/measure-native.py prepare --observe --output artifacts/measurement/baseline
flock /tmp/angband-validation.lock python3 tools/measure-native.py run --build artifacts/measurement/baseline --output artifacts/measurement/pilot --seed 11 --cap 2000 --timeout 120
```

The frozen cohort command (after a pilot and full-length save/non-save control):

```sh
flock /tmp/angband-validation.lock python3 tools/measure-cohort.py --build artifacts/measurement/baseline --output artifacts/measurement/cohort-baseline --phase baseline
```

A `--load` run is a diagnostic check of a retained living save. For a birth save,
loading reproduced the initial state and the2000-turn pilot on the tested pin;
mid-run Borg-memory continuation remains unproven.

Directories must be new; a failed build may use `prepare --retry` to preserve its
previous receipts while rebuilding. Do not overwrite run directories. Keep a
persistent evidence archive under the primary checkout's ignored `artifacts/`,
not only in a disposable task worktree. Record its hash/location and publish
per-run metrics/hashes to the adoption issue/checkpoint. Hosted artifacts are a
second copy with explicit retention, not permanent storage. Recovery must use
committed scripts/pins and public evidence, never workstation `/tmp` paths.

Native monster selection consults local calendar date for seasonal monsters
(December24–26). Future results record actual UTC/local/timezone start/end
context; no clock override is applied. Legacy records explicitly remain
unrecorded rather than borrowing today's date. Seed/save identity alone does
not establish replay equivalence across calendar contexts.

## Owner replay queue and fresh death collection

`./replay_next_death.py` opens the next recorded encounter not explicitly marked
reviewed by the owner; `--list` shows legacy/missing and reviewed families.
`./produce_death.py` runs bounded fresh recorded games and stops at the first
natural death or declared limits. See [usage and limits](DEATH-QUEUE.md). These
commands do not perform causal analysis or mark review complete on viewer exit.
