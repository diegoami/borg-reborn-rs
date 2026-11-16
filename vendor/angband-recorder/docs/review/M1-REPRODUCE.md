# Reproducing M1

Use an isolated review area. These commands operate on the frozen delivery
snapshots, not today's moving consumer branches. Read M1.md and M1-inputs.json
first. Do not reset/switch an active primary, change its pins, clean worktrees,
reuse a user's live profile or push to a real cemetery.

Prerequisites: Git, Rust/Cargo, C compiler, CMake, Python 3.12+, patch, Linux
64-bit and libX11 development headers. GUI controls additionally need Xvfb,
xdotool and fonts. Note compiler/toolchain versions in the report. Missing
prerequisites and skips are qualifications, never proof of a passing check.

## Isolated inputs

```sh
M1_ROOT=$(mktemp -d /tmp/angband-recorder-M1.XXXXXXXX)
git clone https://github.com/diegoami/angband-recorder.git "$M1_ROOT/angband-recorder"
git -C "$M1_ROOT/angband-recorder" switch --detach 8a3352d8ef01a933c1e93e43f6fa3f1d9959cc04
git clone https://github.com/diegoami/borg-rs.git "$M1_ROOT/borg-rs"
git -C "$M1_ROOT/borg-rs" switch --detach 10b5b50e792329275bd2a9a1094fbafc45998cba
git clone https://github.com/diegoami/borg-reborn-rs.git "$M1_ROOT/borg-reborn-rs"
git -C "$M1_ROOT/borg-reborn-rs" switch --detach a794755072a14e3e0207df6c6cebf5845b400e8f
git clone https://github.com/diegoami/better-borg-c.git "$M1_ROOT/better-borg-c"
git -C "$M1_ROOT/better-borg-c" switch --detach 4ab1fc2620137ee892e9f436ed9bf63633f39a97
mkdir -p "$M1_ROOT/proof"
export BORG_PLAY_DATA_DIR="$M1_ROOT/proof/play"
export BORG_DEATH_RECORDS_DIR="$M1_ROOT/proof/deaths"
export BORG_DEATH_WORKFLOW_DIR="$M1_ROOT/proof/workflow"
```

If you inspect the player-cemetery bootstrap, fetch only its pinned commit into
a separate empty checkout (3fe2092dbcfce9fb6fed91fee9f2f27c97829e03). Its later
live datasets are not M1 inputs. Do not use that real remote for push controls;
`test_cemetery_publish.py` constructs private temporary bare remotes.

Keep the review-package documents separately: the code candidate above predates
the package-only commit linked on issue #2. The package does not alter runtime.

## Standalone recorder

```sh
cd "$M1_ROOT/angband-recorder"
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets --features ffi -- -D warnings
git clone --bare https://github.com/angband/angband.git "$M1_ROOT/vanilla.git"
python3 tests/round-trip.py --engine "$M1_ROOT/vanilla.git" --output "$M1_ROOT/proof/vanilla"
```

The round-trip script takes `/tmp/angband-validation.lock` itself. **Do not wrap
it in another flock**. Inspect result.json, the actual streams/save comparisons
and the changed-RNG rejection. Audit malformed/truncated/leftover inputs and
failure finalization beyond the positive control. Existing CI is a starting
point, not an independent verdict.

## borg-rs consumer and publication

```sh
cd "$M1_ROOT/borg-rs"
python3 borg-rs/check-recorder-pin.py recorder-pin.json vendor/angband-recorder
cargo test --release --locked
cargo clippy --release --locked --all-targets -- -D warnings
python3 -m unittest discover -s tools
```

The Python suite takes the shared lock for actual game work and includes the
real archived-recording restore/replay control. Its publication tests use bare
local Git remotes, not production. Supplement them with targeted failure cases:
staged/unstaged changes, unpushed commits, another pending death, push/commit
failure, a changed binary/file, unsafe archive members, size limits and modes.
Restore requires exact binary hashes, not just a version name.

Build the optional recorder profile, then test frequent checkpoint isolation:

```sh
flock /tmp/angband-validation.lock cmake -S angband -B "$M1_ROOT/proof/borg-on" \
  -DBORG_IMPL=rust -DSUPPORT_BORG=ON -DSUPPORT_BORG_RECORDING=ON \
  -DSUPPORT_BORG_HARNESS=ON -DSUPPORT_TEST_FRONTEND=ON -DBUILD_DOC=OFF
flock /tmp/angband-validation.lock cmake --build "$M1_ROOT/proof/borg-on" -j4
python3 harness/check-recorder.py --binary "$M1_ROOT/proof/borg-on/game/angband" \
  --output "$M1_ROOT/proof/borg-checkpoints"
```

`harness/check-recorder.py` locks itself. For recorder-off parity, keep the
original seeds/budgets/assertions and use the original source entry points:

```sh
BORG_IMPL=c flock /tmp/angband-validation.lock bash harness/build.sh
BORG_IMPL=rust flock /tmp/angband-validation.lock bash harness/build.sh
BORG_IMPL=c flock /tmp/angband-validation.lock bash harness/suite.sh smoke "$M1_ROOT/proof/c-smoke"
BORG_IMPL=rust flock /tmp/angband-validation.lock bash harness/suite.sh smoke "$M1_ROOT/proof/rust-smoke"
python3 harness/diff --suite smoke --expect-impl rust "$M1_ROOT/proof/rust-smoke" "$M1_ROOT/proof/c-smoke"
```

For private human/Borg GUI coverage, the existing control is
`python3 harness/regress/gui-replay/run.py OUTPUT --project-root ISOLATED_BORG_CLONE`.
Use only a private display/profile. Preparation and verification contain nested
self-locking calls, so wrapping the entire GUI control in an outer flock can
deadlock. Inspect and coordinate the actual live-play phase explicitly; do not
assume the build lock alone covers it. If you cannot arrange that safely, mark
GUI evidence inspected-only. Review human-only start/resume, optional Borg
activation, backup publication and interruption. Report exact cases actually
run; a few short GUI sessions do not establish every human-game path.

## Reborn and C boundaries

At the pinned reborn clone, run its pin checker, workspace tests/clippy and
`engine/tests`. Its source hash/pristine-tree contracts must hold. Build
`cmake -S engine -B PRIVATE_BUILD -DSUPPORT_BORG_RECORDING=ON` under the shared
lock, then its `harness/check-recorder.py` (self-locking) for native streams,
frequent checkpoint saves and final save equality. The default
`harness/build.sh` must explicitly retain recording OFF and reproduce
`harness/reference/smoke-rust`. The actual-main full perception/boundary/death
run is linked in M1-evidence.json; state whether you reran it or inspected it.

For the C removal, inspect the **first-parent** diff:

```sh
git -C "$M1_ROOT/better-borg-c" diff fcee6ba74b62ef2737e982217886c1eb61132e20 \
  4ab1fc2620137ee892e9f436ed9bf63633f39a97 -- src/
cd "$M1_ROOT/better-borg-c"
python3 -m unittest discover -s tools
flock /tmp/angband-validation.lock python3 tools/verify-native.py
```

The src/ diff should be empty. Inspect the actual observer-only prepare/run and
visible launcher/save/death paths too: the normal native gate does not by itself
prove all launcher behavior. Use private names/profiles and preserve any natural
death through the component's protocol, with its real source/build identity.
Do not start a bug-carry queue or a production cohort.

## Reporting

Keep exact commands, commits, toolchains, failures and output paths. Do not
change seeds, caps, timeouts, expected outcomes or tests to get green. Never
interrupt another lane's lock holder. A blocked/unavailable check stays
explicitly unverified. Return the verdict and M1-F findings requested by
M1-PROMPT.md; do not turn author receipts into your own reproduced evidence.
