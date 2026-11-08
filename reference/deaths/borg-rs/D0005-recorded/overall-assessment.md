# D0005 overall assessment

Status: observed sequence assessed; exact private decision gates and successful
alternatives remain provisional. Follow-up: [Rust investigation #4](https://github.com/diegoami/borg-rs/issues/4).
This answers the owner's [replay feedback](https://github.com/diegoami/borg-rs/issues/3#issuecomment-6713459074)
without changing the original death record or gameplay.

## Overall picture

An Elf Necromancer, character level 2 with 15 maximum HP and 4 maximum MP,
entered dungeon level 1 at 36014 and first recorded Fang at 36094. Two Nether
Bolt attempts preceded the last Phase Door scroll at 36114. The Phase landing
at (191,28) was separated from the nearby upstairs by walls. The Borg attempted
stair return and several retreats; Fang caught it after those movements. The
final position had an unused legal NW step, one known Cure Light Wounds, and
initially 1 MP. It chose melee and one more failed Nether Bolt, then died to
Fang at 36289, HP -2 and MP 0. Fang still had positive HP.

The immediate cause is Fang's bites during the final melee sequence. The
strongest questions concern **why available recovery and walking alternatives
were not admitted before another attack**, and whether spell confidence was
overstated. There is no demonstrated safe complete escape, successful healing
branch, or isolated C/Rust translation defect. This is not a finding that one
more spell, the NW step, or saving the Phase scroll would have saved the life.

The earliest exposure choices were leaving town with limited stock, pursuing
a known item away from the entry stair, and attacking Fang before spending the
last Phase. Town notes report no sale stock and no money, so restocking was
not a free alternative. At first Fang knowledge the straight corridor toward
the entry stair was occupied by Fang. The landing then exchanged immediate
proximity to Fang for uncertain route connectivity. Later stair flows and
local retreat criteria interacted with a faster enemy and nearby monsters;
eventually attack selection preempted later recovery and generic stairs flow.

### Evidence identity and scope

The actual gameplay source is `9881301ceaad91d0c68a61bed1a8ccdde100e660`,
source SHA-256 `d247f691a3f8a7db91489456c4ccc23646a7cacbcc3816bdf142517d899b3794`,
birth seed 1532917772. [record.json](record.json), its 24 hashed artifacts,
original snapshots, build identity and external raw bundle remain unchanged.

The original capture did not contain the new inventory/spell display fields.
Their evidence here is a **separately qualified historical-source observation
replay**, with every original key, message, player-state/base-frame value,
final native save and scheduled native save matching. The observer-disabled
control also matched the original channels. The native archive body matches
apart from its declared death-log wall-clock line. This qualifies supplemental
observations of this one life; it is neither another collected death nor a
whole-Borg private-state shadow or per-frame whole-RNG comparison.
[analysis-evidence.json](analysis-evidence.json) preserves the qualification,
hashes, external paths and anchor bindings.
[analysis-validation.json](analysis-validation.json) records the curation
checks against original/supplemental frames and immutable artifact hashes.

The [16 new anchors](analysis-anchors/README.md) retain exact frame indices,
screens, player-known inventory, native spell status and visible native
monsters. They are separate from the original `snapshots/`. Hidden diagnostic
monster positions are excluded. Native visibility and terminal memory still
do not reveal every private Borg map/flow/eligibility field.

### Actual resources and spell readiness

Before Phase the pack held the spellbook, seven rations, one known Cure Light
Wounds, two known Potions of Sleep and one Phase Door scroll. After Phase the
same four other pack entries remained through death, including Cure Light
Wounds. Pack capacity and carrying weight were available; this is not evidence
of a capacity constraint. There was no equipped launcher or quiver ammunition,
and no light equipped. The terminal native home list has a Word of Recall at
home, not in the carried pack. Sleep potions are not an escape resource.

| Learned spell | Native cost | Native state in the encounter | Limit |
| --- | --- | --- | --- |
| Nether Bolt | 1 MP | Ready with sufficient MP; 36% native failure. At 0 MP, Low mana and 41% failure. | Ready does not mean the Borg admits or selects it, or that it succeeds. |
| Sense Invisible | 4 MP | Ready at full 4 MP, then Low mana after the first bolt; 42% failure at full MP, 50% in the low-mana tail. | Detection does not remove the visible Fang threat. |

No learned healing, Phase, teleport or Recall spell appears. Phase at 36114
was a consumable scroll, leaving 2/4 MP after landing, not a repeatable spell.
The native engine can allow low-mana casting with consequences; Low mana here
means insufficient current MP, not proof that a command cannot be attempted.
The Borg's own affordability/failure gates are separate.

The terminal Borg spell table estimates Nether Bolt failure at **11%**, versus
native **36%** while affordable in the observed tail. The historical Borg uses
`BORG_LIGHT` for the Necromancer light penalty, while the engine uses native
ambient `square_islit`. The 25-point discrepancy is consistent with existing
[source report #296](https://github.com/diegoami/neo-angband-tracker/issues/296).
It is not a new canonical reproduction or proof that an improved estimate
rescues D0005: the terminal table is not a simultaneous trace of each private
predecision estimate. Preserve this source provenance and coordinate through
the C consumer before selecting any adoption.

## Decision timeline

Coordinates are `(x,y)`. These are actual world turns with recorded key
boundaries, not one decision for every world tick. Values are before the
selected action; results are explicitly marked. A key-phase frame precedes
its queued key. In headless `messages.log`, only messages with key count
strictly below a frame's index precede it; equal-count messages follow it.
The CSV retains 24 boundaries, including prompt/acknowledgement distinctions:
[decision-timeline.csv](decision-timeline.csv).

| Actual turn/phase | Knowledge and resources | Chosen action/result | Alternative to examine | Evidence/confidence |
| --- | --- | --- | --- | --- |
| 35844–36004, pre-exposure | Earlier D1 return; town scary-monster notes, no sale stock/money. | Returns to town, then leaves it. | Different economic/entry policy. No free purchase assumed. | Original raw logs and source; no causal branch. The 30-HP preparedness note concerns D2, not a D1 admission violation. |
| 36014 | Full HP15/MP4; town stair followed by new D1 upstairs (189,29). | Descends; frame2110 confirms arrival. | Remain in town or return to stair sooner. | Exact entry anchor. No known Fang before36094. |
| 36024–36094 | Detect Invisible item known at(190,39); player reaches(189,37). Fang first recorded at(189,34),25HP. | Item flow cost9; first Nether Bolt hits, MP4→3, Fang25→21. | Earlier return or first-sight retreat. | Straight stair corridor goes through Fang; a safe return is unproved. The geometric distance is within the CL2 leash, not an established leash violation. |
| 36104 | HP15/MP3, Fang closing. | Second Nether Bolt fails, MP3→2. | Spend escape earlier or try walking. | Recorded concentration failure. |
| 36114 / landed36124 | HP15/MP2; Fang at(190,36); last Phase scroll. | `Safe to Phase. scary squares:0`, `Danger Level4.1`; reads scroll and lands(191,28). | Phase earlier/later, or alternative walking before consumption. | Scroll exhausted; no native damage from Phase. Safety sampling is not stair-connectivity proof. |
| 36124–36154 | Kobold at(189,26) seen near landing; upstairs(189,29) remembered but wall separated. | NE, then stair flow cost14 and SW, then **NW to(190,27)**. By36154 backs SE from there. | Continue NW branch. | Exact keys/positions prove the first NW step legal. The continuation at(189,26) is occupied by kobold then. |
| 36164–36184 | Still HP15/MP2; Fang out of sight. | NE retreat, stairs flow cost17, NE to(193,26), NW to(192,25). | Different route/flow persistence. | Walking/stair attempts occurred; hidden Fang route is not treated as Borg knowledge. |
| 36194 | Fang reappears at(188,24); soldier ant visible(189,25); HP15/MP2. | Third Nether Bolt fails, MP2→1. | Retreat without another cast, different destination. | Native status Ready; actual Borg spell scoring/gates unobserved. |
| 36204–36224 | Fang follows: (190,24)→(192,25)→(193,26); HP15/MP1. | Retreat SE, SW, SW, ending(191,28); first bite follows. | Select NW earlier or keep a different corridor. | Actual retreat notes/positions; no safe branch demonstrated. |
| 36234 | HP11/MP1; Fang NE(192,27),22HP; known CLW; NW(190,27) empty floor. | Melee selected value26. | Legal NW step; drink CLW; affordable native Nether Bolt. | Carried/legal/Ready are facts; actual admission and success are distinct unresolved questions. |
| 36244 / 36253 | HP11 then8; MP1; same player tile. | Two more melees, one hit reduces Fang22→19. | Recovery/route before another bite. | Damage/result logs; native timed state has food only, no confusion/poison/cut/stun. |
| 36263 | HP4/15, MP1/4, known CLW still carried. | Fourth Nether Bolt, selected value42, fails; last MP consumed. | CLW or NW attempt. | By36273 HP2/MP0. Neither alternative tested as rescue. |
| 36273 / 36282 | HP2 then0; MP0, no Phase; CLW remains. | Melee, then another melee at0HP. | Recovery or attempted walking. | Engine permits0HP living; death requires negative HP. |
| 36289 death phase | HP-2/MP0 at(191,28); Fang alive. | Native death. | None after terminal boundary. | Final frame2210 is phase`death`; earlier key frames at the same turn are distinct. |

### The owner's proposed route

At36234 the immediate terrain/occupancy is:

```text
                 NW(190,27) .   N(191,27) #   NE(192,27) Fang
                  W(190,28) #    (191,28) @    E(192,28) #
                 SW(190,29) #   S(191,29) #   SE(192,29) #
```

The NW step is legal: the Borg actually issued it at36144 and was on(190,27)
at36154. At36234 the further NW grid(189,26) is displayed empty, unlike the
earlier kobold blockage. That supports an unused walking **attempt**, which
deserves comparison with staying. It does not establish a safe complete route.
Fang at(192,27) is within Angband distance2 of the candidate and is faster
(native speed120 versus terminal Borg player-speed trait110). The soldier ant
was last visible nearby at(189,25); its later hidden actual position is not
used as present Borg knowledge or a guaranteed obstruction.

The upstairs at(189,29) is geometrically close but blocked by intervening
walls; its corridor approaches from(189,30), not directly from this pocket.
A permissive eight-neighbor check of the captured terminal terrain, even
allowing occupied monster glyphs as passable terrain, finds no complete known
path from the landing/final pocket to that upstairs within the captured window.
This is a bounded screen observation, not proof that the full private Borg map
or its unknown-cell flow has no route. The actual cost14/17 flow notes show
that the Borg was attempting one. See the per-anchor local terrain checks.

### Source-supported decision mechanisms

All links below name the actual historical source, not a newer moving head.

1. **Recovery admission/priority.** Scary-monster notes set leaving/fleeing.
   The ordinary caution heal/defend block is guarded by
   [`!goal.fleeing`](https://github.com/diegoami/borg-rs/blob/9881301ceaad91d0c68a61bed1a8ccdde100e660/borg-rs/src/borg_caution.rs#L1518).
   In the main dungeon loop, a successful
   [attack selection returns](https://github.com/diegoami/borg-rs/blob/9881301ceaad91d0c68a61bed1a8ccdde100e660/borg-rs/src/borg_think_dungeon.rs#L1589)
   before [later recovery](https://github.com/diegoami/borg-rs/blob/9881301ceaad91d0c68a61bed1a8ccdde100e660/borg-rs/src/borg_think_dungeon.rs#L1643)
   and generic stairs flow. Recovery also has its own
   [danger rejection](https://github.com/diegoami/borg-rs/blob/9881301ceaad91d0c68a61bed1a8ccdde100e660/borg-rs/src/borg_recover.rs#L98).
   The earlier bad-shape recovery path is conditional on no light/cut/poison/no
   food; Necromancer light traits and the observed final statuses do not show
   those triggers. These mechanisms make unused CLW plausible without assuming
   no potion existed. Actual private healing counters, current branches and
   exact rejected gates are not captured.
2. **Retreat/stairs are more restrictive than legal movement.** Back-away
   rejects a destination within2 of a faster currently known enemy
   [here](https://github.com/diegoami/borg-rs/blob/9881301ceaad91d0c68a61bed1a8ccdde100e660/borg-rs/src/borg_caution.rs#L1205).
   The final NW witness satisfies that geometry for Fang, even if the known
   per-candidate adjacency flag leak were fixed. It therefore does **not**
   isolate that old flag leak as the cause. Other retreat paths have danger,
   happy-grid and bounce restrictions; their exact current inputs are absent.
   The [desperate-stairs code](https://github.com/diegoami/borg-rs/blob/9881301ceaad91d0c68a61bed1a8ccdde100e660/borg-rs/src/borg_caution.rs#L865)
   has near-stair attempts before its later faster-foe/shallow-unique exclusion.
   Thus “do not run from Fang” alone cannot explain final rejection: the
   private stair list/distance and flow result are needed, particularly with
   the displayed stair geometrically distance2 away.
3. **Phase safety estimates immediate danger.** The logged4.1 branch
   [admits sampled Phase then scroll](https://github.com/diegoami/borg-rs/blob/9881301ceaad91d0c68a61bed1a8ccdde100e660/borg-rs/src/borg_escape.rs#L978).
   Its scary-square sampling does not prove connected return to the stair,
   or that a faster enemy cannot eventually arrive. A bad eventual landing
   outcome alone does not prove the decision was worse than refusing it.
4. **Spell confidence and attack choice.** The
   [Borg light gate](https://github.com/diegoami/borg-rs/blob/9881301ceaad91d0c68a61bed1a8ccdde100e660/borg-rs/src/borg_magic.rs#L575)
   differs from the [engine ambient-light gate](https://github.com/diegoami/borg-rs/blob/9881301ceaad91d0c68a61bed1a8ccdde100e660/angband/src/player-spell.c#L416),
   matching the existing#296 mechanism. But native36% does not itself make
   the spell inadmissible: the attack spell path allows40% against a nearby
   unique. At1MP, two melee deweighting conditions for Mage/Necromancer no
   longer apply, and uniques get a large melee preference
   [in scoring](https://github.com/diegoami/borg-rs/blob/9881301ceaad91d0c68a61bed1a8ccdde100e660/borg-rs/src/borg_fight_attack.rs#L390).
   These are candidates, not a reconstructed score trace. A supposed10-MP
   reserve is **not** an explanation here: the
   [reserve is cleared below CL35](https://github.com/diegoami/borg-rs/blob/9881301ceaad91d0c68a61bed1a8ccdde100e660/borg-rs/src/borg_magic.rs#L499),
   and the low-level primary-spell simulation avoids normal mana penalties.
   Why melee26 wins at36234–36253 but Nether42 appears at36263 needs exact
   per-action score/gate observations.

## Brainstorming and candidate fit

| Candidate / explanation | Supporting facts | Uncertainty / competing explanation | Cost / possible harm | Existing lead / next test |
| --- | --- | --- | --- | --- |
| Ordinary CLW recovery bypassed while fleeing; attack preempts late recovery | Known potion persists at lowHP; repeated fleeing notes; historical guarded heal block and loop order. | Private potion counter/branch/danger values absent; a potion may only delay death. | Healing spends a turn and scarce stock; Fang can attack again. | [C34](https://github.com/diegoami/better-borg-c/issues/34), [C193](https://github.com/diegoami/better-borg-c/issues/193), [Rust4](https://github.com/diegoami/borg-rs/issues/4): observe admission before single CLW branch. |
| Legal NW retreat excluded by faster-foe/danger/route criteria | Empty NW floor and earlier traversal; candidate within2 of faster Fang; attacks replace retreat. | No full known stair route or exact rejected gate; ant/bounce/happy-grid restrictions may matter. | Fang may catch up; walking could expose another enemy or worsen position. | [C159](https://github.com/diegoami/better-borg-c/issues/159), Rust4: log rejection reason and stair-flow result, then isolated NW branch. |
| Phase landing compromises return | Last escape spent; walls separate landing from upstairs; Fang later reappears. | Random landing; initial route already occupied by Fang; actual later walking/stair attempts occurred. | Refusing/delaying Phase may be immediately worse; preserving scroll is not a free rescue. | Rust4: reconstruct pre-Phase candidate/connectivity model before considering isolated timing branch. |
| Necromancer ambient-light confidence mismatch | Native36 versus terminal Borg11; same25-point historical source difference. | Terminal estimate not simultaneous predecision trace;40% unique gate may still permit casting. Three failures can also be ordinary randomness. | Correct estimates may select melee sooner, with worse outcomes in some states. | Existing [neo296](https://github.com/diegoami/neo-angband-tracker/issues/296), C consumer coordination; observe simultaneous native/Borg fail and grid flags. |
| Low-MP melee weighting / shallow-unique combat preference | At1MP caster melee penalties stop; unique preference; final melee26 versus later Nether42. | Missing alternative scores/targets/preconditions; not proved excessive or mistranslated. | Universal anti-melee change could waste turns/mana or prevent a winning finish. | Rust4: exact competing score/gate trace, separate from recovery/route branches. |
| Underprepared descent, shopping or excessive stair distance | CL2/15HP, limited escape/heal stock; item pursuit. | FullHP/MP, no money or sale stock; D1 source readiness is not the30HP D2 rule; first-sight distance is within geometric leash. No entry-policy violation proved. | Waiting in town may stall growth or meet town danger; requiring unavailable stock can block play. | Defer policy change; record first exposure/resource trade-off in a declared future cohort. |
| Simple failed spells / ordinary combat risk | Three of four tail bolts fail; Fang faster and survives. | Does not explain unused CLW or unused legal step; no isolated branch proves a better result. | Retrying this same seed is not a survival-rate sample. | Retain exact RNG/native-save controls in any diagnostic branch; broader policy needs prospective evidence. |
| Replay/tooling fault or Rust-only translation fault | New native display added after original capture, so qualification matters. | All original channels/native saves match and observer-off control matches; no observed source translation mismatch isolated. | Treating a display addition as a new life or rewriting capture would corrupt evidence. | Qualified supplemental anchors; preserve originals. No defect filed as Rust translation proven. |

## Priorities and next evidence

1. **Next discriminating test: observation-only private gate trace on this
   exact original source/origin**, especially36194,36224,36234,36253,36263 and
   36273. Record `goal.fleeing`, healing/recovery admission and reason,
   carried-CLW Borg counters, avoidance/current and candidate danger, selected
   attack and competing scores/targets, plus all stair-list distances and flow
   results. For NW, distinguish legal terrain from the precise retreat/flow
   rejection. Record simultaneous native failure, Borg failure, light flags
   and spell affordability. Prove unchanged original channels and complete
   native save boundaries again. This separates missing admission from losing
   scoring or a different private map; current native-ready observations cannot.
2. With that trace fixed, compare **separate** original-origin diagnostic
   branches for a CLW at a specified decision and a legal NW step. Retain full
   engine/Borg state and RNG continuation; native checkpoint loading alone
   does not restore private Borg memory. Freeze limits and evaluate both
   survival and return/progress, not just extra turns alive. No combined
   heal/move/spell patch that conceals which mechanism matters; no branch was
   performed in this analysis.
3. Coordinate any confirmed shared mechanism with the existing C consumer
   healing/route leads and existing source#296. Pin any adopted correction to
   an **accepted better-borg-c milestone**, with native controls and Rust
   parity under the [adoption contract](../../C-CONSUMER-ADOPTION.md). M5 at
   `0203227` remains unchanged; no first C gameplay target is selected here.
   Issue#4 records evidence/action tracking, not immediate fix authorization.
4. Defer broad entry/restock/Phase/anti-unique strategy redesign. First obtain
   the gate evidence; a diagnostic rescue is still not population benefit.
   Any later policy trial must declare survival and long-term progress
   criteria and use prospective independent starts.

## Outcome and limits

The owner's entry, first Fang, Phase landing, eventual corner fight and MP
exhaustion observations are supported. Two refinements matter: the Borg did
attempt walking/stairs after Phase, and the tail contains four bolts
(one hit, three failures), substantial movement and five melee attacks rather
than continuous spell spam. The proposed NW route has a proved legal first
step; its safe completion is unproved. One known CLW remained unused.

The structured observational review is complete. Causal rescue, exact private
admission/score attribution and general benefit remain open in
[Rust#4](https://github.com/diegoami/borg-rs/issues/4). No gameplay fix, source
milestone adoption, new life, owner replay acceptance or mortality estimate
follows from this review. This is not a qualified universal-correctness claim.
