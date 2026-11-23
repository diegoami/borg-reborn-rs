# 2026-11-21: a build capture never overwrites a file on Windows (#858)

Found by the review of #851 (finding 6). The bug predates #851.

## Done

- `engine/frontend/synthetic-build.c`: the capture record is created by
  `create_new()`.
  - On POSIX: `fopen(path, "wbx")`, as before.
  - On Windows: `_open(path, _O_CREAT | _O_EXCL | _O_WRONLY | _O_BINARY)`,
    then `_fdopen`. The MinGW/UCRT C runtime ignores fopen's `x`: the
    review's probe under Wine showed an existing file truncated.
- `engine/tests/test_character_build.py`: a capture onto an existing file
  fails with "cannot create synthetic build capture" and leaves the file's
  bytes as they were. Before this, no test covered the refusal.
- The release workflow's `windows` job runs the same check with the real
  `angband.exe`: smoke-warrior-1's seed, the reborn core, a capture at
  level 5 onto an existing file.

## Checked

- Linux: the new test passes, with the file's six other tests. The Windows
  step's commands, run against the Linux build, refuse the file and leave
  it unchanged.
- MinGW: `synthetic-build.c` compiles with `-Wall -Wextra`.
- Windows: the build-only release run started with this change.

## Left

- Nothing.
