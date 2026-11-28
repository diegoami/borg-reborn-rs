## 2026-11-28: Cache engine builds (#1420)

Rust cache uses the actual Cargo target directories for CI's workspace
tests, release engine, feature builds and recorder. CMake uses ccache
as a compiler launcher, with a separate recorder cache. The existing
compiler probe, build identity and reference comparisons still run.

Cargo caches are keyed by Rust toolchain and job. The C cache key also
includes runner image family and compiler version. Local and zoo builds
use ccache when it is installed and can select a launcher explicitly with
`CMAKE_C_COMPILER_LAUNCHER`. The harness still probes the real compiler.
CI restores Cargo targets and C compiler entries before building.
