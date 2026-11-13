/*
 * The engine headers the Rust Borg's bindings are generated from
 * (docs/SPEC.md §3.4; engine/bindings/generate.sh). Only the symbols of
 * docs/ENGINE-READS.md are kept (the allowlist in generate.sh).
 */
#include "angband.h"
#include "cave.h"
#include "cmd-core.h"
#include "game-input.h"
#include "effects.h"
#include "game-world.h"
#include "init.h"
#include "monster.h"
#include "obj-desc.h"
#include "obj-gear.h"
#include "obj-knowledge.h"
#include "obj-util.h"
#include "object.h"
#include "option.h"
#include "player.h"
#include "player-calcs.h"
#include "player-spell.h"
#include "player-timed.h"
#include "store.h"
#include "ui-event.h"
#include "ui-input.h"
#include "ui-term.h"
#include "z-dice.h"
#include "z-rand.h"
