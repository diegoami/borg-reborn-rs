## 2026-11-28: Cache engine builds (#1420)

Rust cache uses the actual Cargo target directories for CI's workspace
tests, release engine, feature builds and recorder. CMake uses ccache
as a compiler launcher, with a separate recorder cache. The existing
compiler probe, build identity and reference comparisons still run.

Cargo caches are keyed by Rust toolchain and job. The C cache key also
includes runner image family and compiler version. The CI cache action
sets the CMake launcher before configuration, and the harness still probes
the real compiler. A dedicated C cache holds the full-suite build, and
shard 1 populates it for later runs.
