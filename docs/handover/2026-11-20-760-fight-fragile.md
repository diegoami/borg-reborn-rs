## 2026-11-20: attack categories and winning fights (#760, #759)

Owner-requested combined implementation on `task/T760-fight-fragile`, from
main `08f94e1f`. #757 and #761 belong to another session and are untouched.

- `attack_on_sight` and `caution_winning_fight` are compiled/shipped 0.
- Attack entries extend the existing scary/ignore database with a required
  reason, class/level limits and independent replacement/removal. The
  initial fast/fragile and breeder entries cite engine data; breeder
  entries preserve the existing stoppable/lost-colony policy.
- Synchronized accepted main `ec0f3b28` (#755); preserved its leash changes
  and resolved only the registry/citation overlap.
- A focused attack planner separates damage from the native utility score.
  The general fight race uses remaining HP once, hit/failure chances,
  finite resources, shared mana and speed-scaled incoming damage. Ties do
  not win. Exact observed HP/max HP is retained at the existing perception
  read; health-bar rounding is not used to certify a win. Zero-HP monsters
  still need one damage point. Finishing damage is capped before hit chance,
  fractional arrow averages are retained, and regeneration is charged. The
  fight rung precedes depth-return/light/countdown flight rungs. Unknown
  estimates fall through to ordinary caution.
- The category proposal was presented before building. The six specified
  baseline deaths reproduced on clean `08f94e1f`, with exactly the supplied
  control key streams, before implementation.
- CALIBRATION describes the hypotheses and conservative modelling bounds.
  No population improvement is claimed.

Still in progress under #760/#759: enabled six-case checks, paired 600-start
validation, canonical Cemetery evidence, smoke/default neutrality and final
CI checks. Update this note with their outcomes before marking the PR ready.
The separate #758 Recall-flight guard follows this change; it must cover
the failed-teleport path, which currently sets flags before the common
caution Recall read, rather than reading Recall directly in borg_escape.

Owner decision: review and merge remain pending. Do not merge either PR
without the owner's word; no reviewer has been chosen by this session.
