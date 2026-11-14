/* Read-only Angband observations adapted from borg-rs 2411e52 and
 * better-borg-c 08a5cd019 / 85b6ba8e. See CREDITS.md for licences.
 * No input capture, replay or format ownership is implemented here. */
#define _GNU_SOURCE
#include "native-replay.h"
#include "player.h"
#include "game-world.h"
#include "object.h"
#include "monster.h"
#include "player-timed.h"
#include "option.h"
#include "z-quark.h"
#include "z-color.h"
#include "init.h"
#include "mon-util.h"
static FILE *frames;
static unsigned long long mix_number(unsigned long long hash, long long value)
{
    unsigned long long bits = (unsigned long long)value;
    int i;
    for (i = 0; i < 8; i++) {
        hash = (hash ^ (bits & 255)) * 1099511628211ULL;
        bits >>= 8;
    }
    return hash;
}


static void json_string_to(FILE *stream, const char *text)
{
    const unsigned char *p = (const unsigned char *)(text ? text : "");
    fputc('"', stream);
    for (; *p; p++) {
        if (*p == '"' || *p == '\\') fprintf(stream, "\\%c", *p);
        else if (*p < 32) fprintf(stream, "\\u%04x", *p);
        else fputc(*p, stream);
    }
    fputc('"', stream);
}

static void json_string(const char *text) { json_string_to(frames, text); }

static void diagnostics(void)
{
    const struct object *obj;
    int comma = 0, i;
    fprintf(frames, "{\"scope\":\"diagnostic actual state; never supplied to Borg inputs\",\"gear\":[");
    for (obj = player->gear; obj; obj = obj->next) {
        fprintf(frames, "%s{\"actual_kind\":", comma ? "," : ""); comma = 1;
        json_string(obj->kind ? obj->kind->name : "");
        fprintf(frames, ",\"known_kind\":");
        json_string(obj->known && obj->known->kind ? obj->known->kind->name : "");
        fprintf(frames, ",\"aware\":%d,\"quantity\":%d,\"weight\":%d,\"pval\":%d,\"timeout\":%d,\"to_h\":%d,\"to_d\":%d,\"to_a\":%d,\"inscription\":",
            obj->kind ? obj->kind->aware : 0, obj->number, obj->weight, obj->pval,
            obj->timeout, obj->to_h, obj->to_d, obj->to_a);
        json_string(quark_str(obj->note)); fprintf(frames, "}");
    }
    fprintf(frames, "],\"monsters\":["); comma = 0;
    if (cave) for (i = 1; i < cave->mon_max; i++) {
        const struct monster *mon = &cave->monsters[i];
        if (!mon->race) continue;
        fprintf(frames, "%s{\"id\":%d,\"name\":", comma ? "," : "", i); comma = 1;
        json_string(mon->race->name);
        fprintf(frames, ",\"x\":%d,\"y\":%d,\"hp\":%d,\"max_hp\":%d,\"speed\":%d,\"visible\":%d}",
            mon->grid.x, mon->grid.y, mon->hp, mon->maxhp, mon->mspeed,
            mflag_has(mon->mflag, MFLAG_VISIBLE));
    }
    fprintf(frames, "],\"timed\":{"); comma = 0;
    for (i = 0; i < TMD_MAX; i++) if (player->timed[i]) {
        if (comma) fprintf(frames, ",");
        comma = 1;
        json_string(timed_effects[i].name);
        fprintf(frames, ":%d", player->timed[i]);
    }
    fprintf(frames, "},\"borg\":{}}");
}

#define REPLAY_OBSERVATION_STREAM frames
#include "replay-observations.h"
#undef REPLAY_OBSERVATION_STREAM


void ar_host_extensions(FILE *) __attribute__((weak));
const char *ar_host_extra(bool full)
{
    char *buffer = NULL;
    size_t length = 0;
    frames = open_memstream(&buffer, &length);
    if (!frames) quit("Cannot allocate observation buffer");
    fprintf(frames, "{\"max_depth\":%d,\"max_level\":%d,\"exp\":%lu,\"energy\":%lu,\"race_idx\":%d,\"class_idx\":%d,\"cheat_live\":%d,\"noscore\":%u,\"cause\":",
        player->max_depth, player->max_lev, (unsigned long)player->exp,
        (unsigned long)player->total_energy, player->race->ridx, player->class->cidx,
        OPT(player, cheat_live), player->noscore);
    json_string(player->is_dead ? player->died_from : "");
    fprintf(frames, ",\"details\":"); diagnostics();
    replay_extra_observations(full);
    if (ar_host_extensions) ar_host_extensions(frames);
    fprintf(frames, ",\"palette\":[");
    { int i; for (i=0;i<MAX_COLORS;++i) fprintf(frames,"%s[%u,%u,%u]",i?",":"",angband_color_table[i][1],angband_color_table[i][2],angband_color_table[i][3]); }
    fprintf(frames, "]}");
    if (fclose(frames)) quit("Cannot finish observations");
    frames = NULL;
    return buffer;
}
void ar_host_free_extra(const char *buffer) { free((void *)buffer); }
