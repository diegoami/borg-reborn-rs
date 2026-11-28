## Cache engine builds (#1420)

Rust cache uses the actual Cargo target directories for CI's workspace
tests, release engine, feature builds and recorder. CMake uses ccache
as a compiler launcher, with a separate recorder cache. The existing
compiler probe, build identity and reference comparisons still run.
