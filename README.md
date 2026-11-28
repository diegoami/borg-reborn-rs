<!-- Cemetery ownership: normal death dossiers now live in diegoami/borg-cemetery. -->

# borg-reborn-rs

[Angband](https://github.com/angband/angband) 4.2.6 with a **Borg**: a
computer player that plays the game for you while you watch. The Borg is
written in Rust. The game is upstream Angband, unchanged.

There are two ways to get it: download a ready package, or build it from
this repository.

## Option 1: download a package

On the [Releases page](https://github.com/diegoami/borg-reborn-rs/releases),
pick the package for your system:

| System | Package | To start |
| --- | --- | --- |
| Linux x86_64 | `borg-reborn-rs-VERSION-linux-x86_64.tar.gz` | `tar -xzf` it, `cd` into the folder, `./angband` |
| Windows 10/11 | `borg-reborn-rs-VERSION-windows-x86_64.zip` | *Extract all*, then double-click `angband.exe` |

Also download `SHA256SUMS.txt` and check the package before you unpack it:

```bash
sha256sum -c --ignore-missing SHA256SUMS.txt
```

Nothing is installed. Saves go to `lib/save/` inside the folder. The
package's `PLAYING.txt` says what each system needs. No release has been
published yet, so for now use option 2.
[docs/RELEASING.md](docs/RELEASING.md) lists what a package contains.

## Option 2: build from source

You need Linux (Ubuntu or Debian), about 2 GB of disk and these packages:

```bash
sudo apt-get install -y git python3 cmake build-essential libssl-dev libx11-dev xfonts-base
```

You also need Rust, from [rustup](https://rustup.rs/).

**1. Download.** Clone this repository. It already contains Angband:
`engine/angband/` is upstream Angband at commit
[`574b618c`](https://github.com/angband/angband/tree/574b618c3bfec595f3503a58f8e488b1c4ae3a46),
byte for byte, so you don't download the game separately.

```bash
git clone https://github.com/diegoami/borg-reborn-rs.git
```

**2. Patch and build.** The build makes a copy of `engine/angband/` and
applies the ten small patches in [`engine/patches/`](engine/patches/) to it.
The patches add the hooks the Borg needs, plus the Ctrl-Z prompt that
selects it. The build then compiles the game and links the Rust Borg into
it. The original tree is never modified.
[docs/ENGINE.md](docs/ENGINE.md) explains each patch. This command patches
and builds the game with a window, in about a minute:

```bash
cd borg-reborn-rs && ./watch-borg.sh --build-only
```

To see the patched source on its own, run
`engine/apply-patches.sh /tmp/angband-patched`.

**3. Play.** Start the game:

```bash
LANG=C.UTF-8 ./watch-borg.sh --name MyBorg
```

## Let the Borg play

1. Create a character, or load the one you saved. With `watch-borg.sh`, the
   same `--name` loads the same save.
2. Press **Ctrl-Z**. Dismiss the warnings with Enter, then answer **y** to
   "Are you sure you want to use the borg commands?".
3. At the `PBorg command` prompt, press **z**. The Borg plays.
4. Press any key to stop it and take the controls back. Ctrl-Z, z starts it
   again.

At the prompt, **?** lists the Borg's commands and **B** switches between
two Borgs:
- **PBorg**: this project's Borg, the default.
- **CBorg**: Angband's own C Borg.

A game the Borg has played is marked as such, as with Angband's own Borg.

## Read next

| If you want to | Read |
| --- | --- |
| watch, change the Borg's settings, record a game, see how it dies and how it is measured | **[the play guide](docs/user/GUIDE.md)** |
| help: build and test, change how the Borg plays, investigate a game, measure with the zoo | **[the contributor guide](docs/contributing/README.md)** |
| understand how the Borg works: the original C Borg, the core we ship, the redesign | [docs/architecture/](docs/architecture/README.md) |
| learn the game itself, as the Borg sees it | [the Angband Strategy Manual](docs/strategy/README.md), and Angband's own manual in [engine/angband/docs/](engine/angband/docs/) |
| see every document | [docs/README.md](docs/README.md) |
| work on the repository as a session (the rules, the plan, the handover) | [AGENTS.md](AGENTS.md) |

How well the Borg plays is measured by
[borg-zoo](https://diegoami.github.io/borg-zoo/), and every recorded death is
kept in [borg-cemetery](https://github.com/diegoami/borg-cemetery). The
other related projects are listed in
[the contributor guide](docs/contributing/repositories.md).

## Licence

Licensed under GPLv2 (see `LICENSE`), like borg-rs. Angband itself is dual-licensed: GPLv2 or the
Angband licence. Files copied from other projects keep their headers and
their provenance is listed in [docs/PROVENANCE.md](docs/PROVENANCE.md).
