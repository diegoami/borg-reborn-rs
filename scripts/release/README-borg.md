
## The Borg in this package

This Angband comes with **the Borg reborn**, a computer player written in
Rust ([borg-reborn-rs](https://github.com/diegoami/borg-reborn-rs)). The
package also includes Angband's native CBorg. PBorg uses the **borg-rs
core**, a Rust translation of CBorg with this project's settings.

**This is an early preview** (a pre-release). The Borg often dies in the
first few dungeon levels. PLAYING.txt says what the package needs and how to
start it.

### Letting the Borg play

1. Start the game from its folder: `./angband` on Linux, or double-click
   `angband.exe` on Windows.
2. Create a character, or load your save.
3. Press **Ctrl-Z**, dismiss the warnings, and answer **y** to "Are you sure
   you want to use the borg commands?".
4. At the PBorg prompt, press **z**: the Borg plays.
5. Press any key to stop it and take the controls back. Ctrl-Z and z start
   it again.

A game the Borg has played is marked as such, as with Angband's own Borg.

PBorg is selected at launch. At the Ctrl-Z prompt, press **?** to see the
commands, including **B** to switch Borgs. CBorg settings live in
`lib/user/borg.txt`.

### Where things are

- Saves: `lib/save/`.
- The Borg's settings: `lib/user/pborg.yaml`. Every setting is commented; edit
  the file to change what the Borg does. `borg/` holds the shipped settings
  again, for reference.
- What the package was built from: `RELEASE.txt`. The licences of every
  part: `LICENSES/`.

The source of this release, and how the Borg is measured, are at
https://github.com/diegoami/borg-reborn-rs.
