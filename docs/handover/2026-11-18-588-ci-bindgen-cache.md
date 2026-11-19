## 2026-11-18: CI caches bindgen-cli (#588)

Owner: "yes do it", after #586's review named the last fixed cost. The rust
job ran `cargo install bindgen-cli --version 0.73.2 --locked` on every run
(about 33 s). Now `actions/cache` keeps `~/.cargo/bin/bindgen`, keyed by the
version, the runner OS and architecture and the runner image (`ImageOS`), and
`cargo install` runs only on a cache miss. A `bindgen --version` step logs
which binary the regeneration check uses.

- The first run on main fills the cache; pull requests read main's cache.
- **Changing the version:** change it in both the key and the install line.
