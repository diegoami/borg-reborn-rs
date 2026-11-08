/* Player-known, observation-only extensions shared by GUI/headless replay.
 * Native helpers adapted from better-borg-c; see IMPORTED.json. Same dual
 * licence as Angband (docs/copying.rst). No C Borg globals or algorithms. */
#ifndef REPLAY_OBSERVATION_STREAM
#error Define the host recorder stream before including this file
#endif
#define frames REPLAY_OBSERVATION_STREAM
#include "replay-native-inventory.h"
#include "replay-native-spells.h"
#undef frames

static void replay_extra_observations(bool full)
{
    if (getenv("BORG_NO_EXTRA_OBSERVATIONS")) return;
#ifdef REPLAY_ADD_VITALS
    fprintf(REPLAY_OBSERVATION_STREAM,
        ",\"max_hp\":%d,\"max_sp\":%d,\"caster\":%s",
        player->mhp, player->msp, player->class->magic.total_spells ? "true" : "false");
#endif
    replay_inventory(full);
    replay_spells(full);
}
