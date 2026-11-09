# Commenting on a death

Open [the death registry](README.md), select **Discuss** for the case, and leave
a comment on its linked GitHub investigation. Each report also has a
**Discuss this death** link. This is the durable discussion outside chat; a
comment does not need to be a code review or a new issue.

- [D0001: stegocentipede — discussion22](https://github.com/diegoami/better-borg-c/issues/22)
- [D0002: gallant — discussion34](https://github.com/diegoami/better-borg-c/issues/34)

Useful comment format (plain language is fine):

```text
Case: D0002
Observation: too many enemies for this character's level/resources.
Moment: room entry or the earliest visible dangerous group.
Suggestion/question: could it have avoided exposure or retreated earlier?
```

You can propose an explanation without proving it. The agent checks the case's
actual build, trajectory, inventory and what the Borg knew before acting.
Different builds and replay attempts keep their own identities. A terminal
picture alone does not prove how it entered the situation or whether a proposed
escape would work.

## When comments are checked

The consumer agent checks new or edited comments on every case investigation
and its linked open action issues at the start of each active work session and
before stopping. The checkpoint records which threads/latest comment IDs were
read and what needs follow-up. New owner comments receive an acknowledgement,
evidence triage and an action/finding link; acknowledgement does not mean the
underlying investigation is complete. Agent-generated receipts are not new
owner requests and do not require repeated replies.

This is part of active work. There is no unattended comment-processing agent
running between sessions. The weekly GitHub donor inventory collects fork
metadata; it does not analyze death comments. A new confirmed or credible bug
takes priority under the existing bug-first policy.

## Replay inspection windows

Owner's current default offsets are **5,12,25,40,60,82,105 engine turns before
death**, denser near the end. Ten is intentionally omitted from the new profile;
older10/15/20 snapshots remain historical evidence. [Profile](window-profile-v2.json).
These are engine turns, not Borg decisions, energy or wall seconds. Capture the
actual native phase/state and require a matching original-start prefix; do not
interpolate or claim that every offset is a distinct player action.

Follow important events earlier than105 with separately labeled anchors, such
as the last escape item or entry into a crowd. Use the case's actual historical
build/start/options, not the latest corrected Borg. Diagnostic window caps and
replays are not additional independent survivors or a changed gameplay cohort.

## From comment to action

1. **Record.** Keep the original GitHub comment. Chat feedback is copied to the
   case investigation with an owner-feedback heading and link before being
   treated as tracked. Add its link/summary to assessment.json's feedback list.
2. **Triage.** State what the recording supports, uncertainties, alternatives
   and whether this is a suspected bug, policy suggestion or expected behavior.
   Keep owner observations distinct from verified native facts.
3. **Track.** Add a specific action with a stable key and issue link to
   assessment.json. A focused new issue is useful for a reusable policy or
   separate mechanism; avoid another source report for the same defect.
4. **Investigate.** Use the earliest relevant known state and ordinary legal
   alternatives. Record resource costs, unavailable information, RNG divergence,
   failed/unfavorable results and whether the comparison is a diagnostic branch.
5. **Route.** Confirmed reusable original-C defects go to neo-angband-tracker
   with source207 ownership before source fix and separate consumer carry.
   Broader inventory/exposure policy suggestions remain proposals until assessed;
   an owner priority does not become a proved cause or an untested threshold.
6. **Report back.** Reply on the original investigation with the finding and
   experiment/fix links. Update the dossier assessment/actions and regenerate
   the registry. An investigation can end as not-a-bug or inconclusive; it is
   not silently marked fixed. Group shared actions across cases, not replays.

Use a plain action status such as **recorded**, **investigating**, **needs evidence**,
**proposed**, **verified**, **not a bug** or **inconclusive**, with a short reason.
Verification links to actual native evidence; a merged policy alone does not
prove a survival benefit. Original evidence/builds stay immutable when an
assessment evolves. Bug corrections always take priority.

## Current owner feedback

D0001: prioritize usable escape/healing/survival stock over spare gear. Its
terminal pack had19occupied/23slots (four free), and Phase Door was carried and
consumed earlier. [Proposal41](https://github.com/diegoami/better-borg-c/issues/41)
tracks reserve/retention/restock priorities; [22](https://github.com/diegoami/better-borg-c/issues/22)
tracks the actual stock/retreat/Speed decisions. Slot pressure or departure
with zero scrolls is not established from the terminal dump.

D0002: exposure to too many enemies too early for level3/resources.
[Action on34](https://github.com/diegoami/better-borg-c/issues/34#issuecomment-6685824152)
reconstructs first known danger and walk/flow/descent/teleport entry. A recorded
Phase Door precedes the critical crowd; intentional walking into it is not
assumed. No new crowd threshold or claimed rescue is implemented from one case.

## Evaluation and implementation are separate

Follow [the improvement workflow](../measurement/DEATH-IMPROVEMENT-WORKFLOW.md)
for death-derived investigations/enhancements and the separate prospective
implementation, survival and progress evidence gate. Local enhancement154 links
source370; neither filing nor a rescued replay proves a benefit.

## Every processed death includes brainstorming and an overall picture

Use [the review template](REVIEW-TEMPLATE.md) to connect preparedness, threat
valuation, exposure, escape and recovery rather than discuss only the final blow.
List competing mechanisms, candidate fit/costs, confidence and next evidence.
Explicitly assess attempted walking retreat, especially with no Phase Door;
a guaranteed-safe route is not required to consider the attempt.
The [gallant family assessment](D0004-named-headless/overall-assessment.md) links
the theft-weight158 and room-retreat159 analysis candidates under investigation34.

## Owner replay queue

Use `./replay_next_death.py` for the next available encounter not marked reviewed
by the owner; `--list` includes legacy/missing movies and `--mark-reviewed Dnnnn`
records explicit completion. Quitting the viewer does not mark it done or close
investigations. `./produce_death.py` collects fresh disposable recorded games
within declared bounds; it cannot guarantee a natural death. See
[queue/collector usage](../measurement/DEATH-QUEUE.md).

D0006 owner review is on [177](https://github.com/diegoami/better-borg-c/issues/177):
identification180, ammunition/melee readiness181, Recall/Home and two delayed
DeepDescent jumps183, shared walking-retreat159 and heal-eligibility34.
[Overall picture](D0006-collected/overall-assessment.md) distinguishes verified
trajectory from untested rescues. HP mismatch178 is stale sidebar rendering
during successful rest regeneration; original recording/state stays intact.

D0007 owner review remains on [163](https://github.com/diegoami/better-borg-c/issues/163#issuecomment-6712916845):
[overall picture](D0007-collected/overall-assessment.md) qualifies early room loot,
first Phase spell, actual stair-return attempts and the jelly/soldier finish.
[193](https://github.com/diegoami/better-borg-c/issues/193) investigates native-ready
Phase Door versus Borg admission; shared159 covers retreat/loot priority and34
healing eligibility. Five known cures and2MP remain late; no safe rescue proved.

D0008 owner review is on [163](https://github.com/diegoami/better-borg-c/issues/163#issuecomment-6714388294),
with focused [212](https://github.com/diegoami/better-borg-c/issues/212) and an
[overall assessment](D0008-collected/overall-assessment.md). Exact original and
stderr-only diagnostic replays establish repeated upstairs retreat/re-exposure,
three CLW consumed, two Phase left and no melee. Qualified C cache transitions
match the owner's linked [Rust report5](https://github.com/diegoami/borg-rs/issues/5)
while retaining separate actual lives. Canonical [source404](https://github.com/diegoami/neo-angband-tracker/issues/404)
tracks unchecked emergency-flow cache reuse; source156 and shared159 remain
separate guard/escape-intent actions. No fix or successful rescue is claimed.

D0009 chat feedback is recorded on [163](https://github.com/diegoami/better-borg-c/issues/163#issuecomment-6717711058),
with focused [235](https://github.com/diegoami/better-borg-c/issues/235) and
[overall picture](D0009-collected/overall-assessment.md). Qualified original replay
and native messages show full-pack pickup retries before food exhaustion, then
unsuccessful D5/D6 escape cycling. Canonical [source419](https://github.com/diegoami/neo-angband-tracker/issues/419)
tracks quiver-aware capacity; shared41/180/183/159 remain separate food,
identification,Recall and return investigations. Five unidentified edible
mushrooms remain at death; no food/Recall spell exists. No correction/rescue
claimed, and owner-review completion is not inferred from this feedback alone.

D0010 owner feedback is recorded on [163](https://github.com/diegoami/better-borg-c/issues/163#issuecomment-6718075726):
[overall picture](D0010-collected/overall-assessment.md) qualifies the normal-mode
ranger's Phase, repeated desperate stair retreat and three kobold hits8/6/2.
Usable bow/eleven arrows remained; [239](https://github.com/diegoami/better-borg-c/issues/239)
examines legal archery/route admission without assuming a safe route or a bug.
The owner's saved manual alternative is fully verified, alive7303/CL2/HP12;
six confirmed arrow hits and a melee yeek finish killed both enemies. This is a
save/load diagnostic, not an independent Borg life or a population benefit.
[240](https://github.com/diegoami/better-borg-c/pull/240) supplies the qualified
native fighting log and valid wide-window reader. Saving did not mark review done.

## Lessons for a redesign

Findings across reviewed deaths and the owner's verified manual takeovers are
consolidated as design requirements in [REDESIGN-LESSONS.md](REDESIGN-LESSONS.md),
intended for the owner's ground-up borg-reborn-rs redesign. Update it in the same
PR whenever a death review adds or changes evidence.
