# Config-parser robustness carries (sources 141, 142, 144): verified product scenarios

Three corrections to `src/borg/borg-init.c`, adopted as consecutive patches in one PR
with a before-build snapshot at each step (so every verifier compares exactly one change):

| patch | consumer | source | applied | source commit |
| --- | --- | --- | --- | --- |
| 0057 | [257](https://github.com/diegoami/better-borg-c/issues/257) | [141](https://github.com/diegoami/neo-angband-tracker/issues/141) cfghang (CBUG-0125) | `052129ae5b7e072513b80ac2626d82792e6a8463` | `45dfc9b1f4930b28841d0c38b73cc4cd3171aab0` |
| 0058 | [258](https://github.com/diegoami/better-borg-c/issues/258) | [142](https://github.com/diegoami/neo-angband-tracker/issues/142) cfgminus (CBUG-0126) | `0ceeaadbb62a0fd2d4d09e38816023de21a4e630` | `6bc43e71a4b36189ba7246abe53f114d6259469c` |
| 0059 | [259](https://github.com/diegoami/better-borg-c/issues/259) | [144](https://github.com/diegoami/neo-angband-tracker/issues/144) cfgmissing (CBUG-0128) | `e8a50ea9f5976afd075bcbb4d54dcf9d564b5df3` | `8cbee69447cda2526b4810fd6ea74677921ff1d1` |

Each is a `cherry-pick -x` of the reviewed C-only source commit, applied cleanly in sequence.

## Defects

* **141, hang:** an integer setting whose value starts with an unexpected character (a
  word, or a number followed by a suffix the scanner does not consume) made the
  `borg_proc_setting()` loop spin forever, hanging Borg initialisation. The value now
  stops at the first unexpected character and the default stays.
* **142, sign:** the parser negated the stored value after the loop even when no number
  followed the minus sign, so a bare minus flipped a stale or default value (127 became
  -127, which changes the preparedness gate). It now negates only when a number was read.
* **144, NULL:** the configuration was allocated only after reopening `borg.txt`, so an
  unreadable or unwritable file left it NULL and the Borg crashed. Defaults are now
  allocated first and initialisation returns with a warning when the reopen fails.

None is exercised by the default, valid `borg.txt` (no survival claim; the v1 cohort was
not rerun because its configuration is valid and unchanged).

## Pins and provenance

Before: `origin/main` `20d7d4e6203d976a6fd294818d6e74c73eb94ad7`. Evidence fixtures unchanged (heads `096ee66813dad5a32f9468d5fff2c48d2a46077c`,
`0a21f5ce0cfdadfb1d3520480425a337bab5f752`, `f37cc9b515e361913a6c680451c4ceab62e3f51c`; runtime files
byte-identical to the run pins `a3164a61b`, `18f79a3cc`, `1ff9ca1e5`, asserted). Provenance: for 0057 the
source diff applied to the before module equals the product module; for 0058/0059, which stack on earlier
patches in the same function, the source change (source parent to source fix) merged with `git merge-file`
onto the before module equals the product module (the plain source diff no longer applies there). Each
verifier also asserts that `borg-init.c` is the only `src`/`lib`/build-file difference from its before commit.

## Verification (`artifacts/cfghang`, `cfgminus`, `cfgmissing`; 2026-11-08)

```sh
git remote add source-tracker https://github.com/diegoami/neo-angband-tracker.git
git fetch source-tracker 'refs/heads/task/cfg*'
# per step: build the before commit, snapshot build/native, build the fix commit, then
flock /tmp/angband-validation.lock python3 tools/verify-cfghang.py --before-build SNAP0 --fix 052129ae5b7e072513b80ac2626d82792e6a8463 --output artifacts/cfghang/scenarios
flock /tmp/angband-validation.lock python3 tools/verify-cfgminus.py --before-build SNAP1 --fix 0ceeaadbb62a0fd2d4d09e38816023de21a4e630 --output artifacts/cfgminus/scenarios
flock /tmp/angband-validation.lock python3 tools/verify-cfgmissing.py --before-build SNAP2 --fix e8a50ea9f5976afd075bcbb4d54dcf9d564b5df3 --output artifacts/cfgmissing/scenarios
flock /tmp/angband-validation.lock python3 tools/verify-native.py --jobs 10   # on the tip
```

| source | mode | before | after |
| --- | --- | --- | --- |
| 141 | `word`, `suffix` | exit 2 (watchdog), 0/1, `hung=1` | exit 0, **1/1**, default kept (`stop_dlevel=128`) |
| 141 | `valid` | 1/1, `stop_dlevel=7` | same, log byte-identical |
| 142 | `minus` | 0/1, `no_deeper=-127`, `prepared20=0` | **1/1**, `no_deeper=127`, `prepared20=1` |
| 142 | `valid` (100), `negative` (-5) | 1/1 | same, logs byte-identical |
| 144 | `missing` | exit 3 (crash), 0/1 | exit 0, **1/1**, defaults (`stop_dlevel=128`) |
| 144 | `present` | 1/1 | same, log byte-identical |

All rows equal the source evidence's recorded rows value for value. Native gate on the tip
`e8a50ea9f`: **965/965 unit + 5/5 frontend**; tooling 121 pass. Each `result.json` has
`verified: true`.

## Scope and limits

* The three verifiers ran stepwise (each against its own before-build); the full native gate ran
  once on the stacked tip, not after each patch.
* Fixtures isolate parsing and initialisation, not play. Normal phases only (the fixtures
  observe the crash and hang as exit codes; no sanitizer phase). Prior-fixture matrix not rerun.
* Not exercised by the valid default `borg.txt`; no cohort run.

## Exact next work

PR from `task/carry-cfg`, hosted checks, merge; mark 257/258/259 carried/open+patch-active;
receipts on each, on sources 141/142/144 and on checkpoint 1. Next: countsell-134, drainedive-235,
prepreason-234, swapacid-199, spastic-146, glyphflow-345, respois-184/185.
