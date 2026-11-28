# Measuring with the zoo

*Part of the [contributor guide](README.md).*

## Why the zoo decides

[borg-zoo](https://diegoami.github.io/borg-zoo/)
([repository](https://github.com/diegoami/borg-zoo)) decides whether a
change makes the Borg better. One game says little: the same Borg survives
one seed and dies on the next. So **every comparison of settings or builds
over a set of starts goes to the zoo** (RULES 20), and no improvement is
claimed without its measurement (RULES 14). Local games are for named
cases, builds and smoke. Running more than a few dozen games locally needs
the owner's word. A local number given for direction says so.

The zoo builds and runs this repository through a fixed contract:
`harness/build.sh`, `harness/run.sh` and their outputs, described in
[docs/ZOO-INTERFACE.md](../ZOO-INTERFACE.md). The adapter it uses is
pinned in `harness/zoo-adapter-pin.txt`.

## The zoo's words

| Word | What |
| --- | --- |
| **specimen** | an exact build: a commit of this repository, built and identified by its `BUILD_ID` |
| **arm** | a specimen played with given settings: the shipped `pborg.yaml`, or that with some `borg_param_*` overrides |
| **profile** | a named set of overrides an arm plays with |
| **suite** | the frozen list of starts (seed, race, class) every arm plays, so that arms are compared game by game |
| **tier** | the size of a run: a pilot or a small run first, a larger one when it is worth it |
| **gate** | the prespecified endpoint a run is judged on, written before the run |

## Asking for a measurement

1. Push the code to measure. A scratch branch is fine.
2. File an issue in
   [diegoami/borg-zoo](https://github.com/diegoami/borg-zoo/issues) saying:
   - the exact commit;
   - the arms, as `borg_param_*` overrides on the shipped
     `harness/pborg.yaml`;
   - the runs: suite and tier;
   - what you want back, including the endpoint.

   [borg-zoo#510](https://github.com/diegoami/borg-zoo/issues/510) is a
   model.
3. A zoo session dispatches it. Population runs are started by the owner,
   because they hold the account's Actions capacity for hours.

A comparison names exact specimens (RULES 22): never "main" or "the latest".

## Reading a result

The zoo pairs each game of one arm with the same start in the other arm:
- **identical games:** a change that never fires plays exactly as before;
- **rescued and newly killed:** the starts where one arm dies and the other
  survives;
- **paired deaths:** the starts where both arms die, and whether earlier or
  later;
- **progress:** the depth and level reached, and the turns played.

The zoo's death reports show each death's last screens, which can be
replayed. The investigation of a death belongs in borg-cemetery
([investigating.md](investigating.md)).
