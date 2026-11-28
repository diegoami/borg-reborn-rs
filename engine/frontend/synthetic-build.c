/* Synthetic P1 build projection. Native mechanics are in player.c,
 * player-calcs.c:calc_hitpoints and obj-make.c:object_prep at engine/UPSTREAM.
 * Deliberately supports only Human Warriors and mundane base-kind items.
 * Unsupported properties fail, never silently become weaker invented gear. */
#include "angband.h"
#include "init.h"
#include "game-world.h"
#include "object.h"
#include "obj-gear.h"
#include "obj-knowledge.h"
#include "obj-make.h"
#include "obj-pile.h"
#include "obj-util.h"
#include "player.h"
#include "player-calcs.h"
#include "synthetic-build.h"
#include <errno.h>
#include <openssl/sha.h>
#ifdef _WIN32
#include <fcntl.h>
#include <io.h>
#include <sys/stat.h>
#endif

#define BUILD_LIMIT 16384
#define ITEMS_LIMIT 48
#define ENGINE_PIN SYNTHETIC_ENGINE_PIN

static void fail(void) { quit("borgh: invalid or unsupported synthetic character build"); }

/* A new file, never an existing one (it is left as it was). #858: the
 * Windows C runtime ignores fopen's "x" and truncates an existing file;
 * _O_EXCL is honoured there. */
static FILE *create_new(const char *path)
{
#ifdef _WIN32
    int fd = _open(path, _O_CREAT | _O_EXCL | _O_WRONLY | _O_BINARY, _S_IREAD | _S_IWRITE);
    FILE *fp;
    if (fd < 0) return NULL;
    fp = _fdopen(fd, "wb");
    if (!fp) _close(fd);
    return fp;
#else
    return fopen(path, "wbx");
#endif
}

void synthetic_build_digest(const char *path, char digest[65])
{
    unsigned char bytes[BUILD_LIMIT + 1], hash[SHA256_DIGEST_LENGTH];
    FILE *fp = fopen(path, "rb");
    size_t size;
    int i;
    if (!fp) fail();
    size = fread(bytes, 1, sizeof(bytes), fp);
    if (ferror(fp) || fclose(fp) || size > BUILD_LIMIT) fail();
    SHA256(bytes, size, hash);
    for (i = 0; i < SHA256_DIGEST_LENGTH; ++i) sprintf(digest + i * 2, "%02x", hash[i]);
    digest[64] = '\0';
}

static bool plain(const struct object *obj)
{
    int i;
    struct object *base;
    bool equal;
    if (obj->artifact || obj->ego || obj->curses || obj->brands || obj->slays)
        return false;
    base = object_new();
    object_prep(base, obj->kind, 0, MINIMISE);
    equal = !base->curses && !base->brands && !base->slays &&
        of_is_equal(obj->flags, base->flags) && obj->weight == base->weight &&
        obj->dd == base->dd && obj->ds == base->ds && obj->ac == base->ac;
    for (i = 0; i < OBJ_MOD_MAX; ++i)
        equal = equal && !obj->modifiers[i] && !base->modifiers[i];
    for (i = 0; i < ELEM_MAX; ++i)
        equal = equal && obj->el_info[i].res_level == base->el_info[i].res_level &&
            obj->el_info[i].flags == base->el_info[i].flags;
    object_free(base);
    return equal;
}

void synthetic_build_capture(const char *path)
{
    struct object *obj;
    int i, count = 0, pack_count = 0;
    FILE *fp;
    if (player->race->ridx != 0 || player->class->cidx != 0 ||
        player->lev > 15 || player->lev != player->max_lev || player->msp != 0)
        fail();
    for (i = 0; i < STAT_MAX; ++i)
        if (player->stat_cur[i] != player->stat_max[i]) fail();
    for (obj = player->gear; obj; obj = obj->next)
        if (!plain(obj) || ++count > ITEMS_LIMIT || obj->number < 1 || obj->number > 40 ||
            abs(obj->to_h) > 20 || abs(obj->to_d) > 20 || abs(obj->to_a) > 20 || obj->pval < 0 || obj->timeout < 0 ||
            (!object_is_equipped(player->body, obj) && ++pack_count > z_info->pack_size)) fail();
    fp = create_new(path);
    if (!fp) quit("borgh: cannot create synthetic build capture (preserve existing file)");
    fprintf(fp, "schema:borg-reborn-rs/character-build@1\nengine:%s\n", ENGINE_PIN);
    fprintf(fp, "identity:%d:%d:%d\n", player->race->ridx, player->class->cidx, player->lev);
    fprintf(fp, "observed:%ld:%d:%d\n", (long)turn, player->chp, player->csp);
    fprintf(fp, "vitals:%ld:%ld:%d:%d\n", (long)player->exp, (long)player->au, player->mhp, player->msp);
    fprintf(fp, "stats");
    for (i = 0; i < STAT_MAX; ++i) fprintf(fp, ":%d", player->stat_max[i]);
    fprintf(fp, "\nhp-rolls");
    for (i = 0; i < PY_MAX_LEVEL; ++i) fprintf(fp, ":%d", player->player_hp[i]);
    fprintf(fp, "\nknowledge:%d:%d:%d\n", player->obj_k->to_h, player->obj_k->to_d, player->obj_k->to_a);
    for (obj = player->gear; obj; obj = obj->next)
        fprintf(fp, "item:%d:%d:%d:%d:%d:%d:%d:%d:%d:%d\n", obj->tval, obj->sval,
            obj->number, object_is_equipped(player->body, obj) ? object_slot(player->body, obj) : -1, obj->to_h, obj->to_d, obj->to_a,
            obj->pval, obj->timeout, obj->kind->aware ? 1 : 0);
    fprintf(fp, "end\n");
    if (ferror(fp) || fclose(fp)) quit("borgh: synthetic build capture write failed");
}

/* Strict fixed-order record; no ignored directives, duplicate fields, comments,
 * empty lines or native item payloads. All values fit the native fields. */
static void numbers(char *line, const char *label, int count, long *values)
{
    int i;
    char *next, *end;
    if (!line || strncmp(line, label, strlen(label)) || line[strlen(label)] != ':') fail();
    next = line + strlen(label) + 1;
    for (i = 0; i < count; ++i) {
        if (!*next || (*next != '-' && (*next < '0' || *next > '9'))) fail();
        errno = 0;
        values[i] = strtol(next, &end, 10);
        if (end == next || errno || (*end != ':' && *end != '\0')) fail();
        if (i == count - 1) { if (*end) fail(); }
        else if (*end != ':') fail();
        next = end + (*end ? 1 : 0);
    }
}

void synthetic_build_apply(const char *path, char digest[65])
{
    char bytes[BUILD_LIMIT + 1], *line, *context;
    unsigned char hash[SHA256_DIGEST_LENGTH];
    long identity[3], observed[3], vitals[4], stats[STAT_MAX], hp[PY_MAX_LEVEL], knowledge[3];
    long items[ITEMS_LIMIT][10];
    struct object *prepared[ITEMS_LIMIT];
    bool slots[64] = {false};
    size_t size;
    int i, j, count = 0, pack_count = 0;
    FILE *fp = fopen(path, "rb");
    if (!fp) quit("borgh: cannot read synthetic character build");
    size = fread(bytes, 1, sizeof(bytes), fp);
    if (ferror(fp) || fclose(fp) || !size || size > BUILD_LIMIT || bytes[size - 1] != '\n') fail();
    for (i = 0; i < (int)size; ++i)
        if (!bytes[i] || bytes[i] == '\r' || (bytes[i] == '\n' && (!i || bytes[i-1] == '\n'))) fail();
    SHA256((unsigned char *)bytes, size, hash);
    for (i = 0; i < SHA256_DIGEST_LENGTH; ++i) sprintf(digest + i * 2, "%02x", hash[i]);
    digest[64] = '\0';
    bytes[size] = '\0';
    line = strtok_r(bytes, "\n", &context);
    if (!line || !streq(line, "schema:borg-reborn-rs/character-build@1")) fail();
    line = strtok_r(NULL, "\n", &context);
    if (!line || !streq(line, "engine:" ENGINE_PIN)) fail();
    numbers(strtok_r(NULL, "\n", &context), "identity", 3, identity);
    numbers(strtok_r(NULL, "\n", &context), "observed", 3, observed);
    numbers(strtok_r(NULL, "\n", &context), "vitals", 4, vitals);
    numbers(strtok_r(NULL, "\n", &context), "stats", STAT_MAX, stats);
    numbers(strtok_r(NULL, "\n", &context), "hp-rolls", PY_MAX_LEVEL, hp);
    numbers(strtok_r(NULL, "\n", &context), "knowledge", 3, knowledge);
    /* The record's race and class are the started character's (the harness
     * starts the race and class the record names; the kit, experience, hit
     * points and mana are then replaced and checked against the native
     * calculation). Spell knowledge is not restored. */
    if (identity[0] != player->race->ridx || identity[1] != player->class->cidx ||
        identity[2] < 1 || identity[2] > 15 || player->body.count > 64 ||
        vitals[0] < 0 || vitals[0] > 100000000 || vitals[1] < 0 || vitals[1] > 100000000 ||
        vitals[2] < 1 || vitals[2] > 10000 || vitals[3] < 0 || vitals[3] > 10000 || observed[0] < 0 ||
        observed[1] < 0 || observed[1] > vitals[2] || observed[2] < 0 || observed[2] > vitals[3]) fail();
    if ((identity[2] > 1 && vitals[0] < player_exp[identity[2]-2] * player->expfact / 100L) ||
        vitals[0] >= player_exp[identity[2]-1] * player->expfact / 100L) fail();
    for (i = 0; i < STAT_MAX; ++i) if (stats[i] < 3 || stats[i] > 118) fail();
    if (hp[0] != player->hitdie) fail();
    for (i = 1; i < PY_MAX_LEVEL; ++i)
        if (hp[i] <= hp[i-1] || hp[i] - hp[i-1] > player->hitdie) fail();
    for (i = 0; i < 3; ++i) if (knowledge[i] < 0 || knowledge[i] > 1) fail();
    while ((line = strtok_r(NULL, "\n", &context)) && !streq(line, "end")) {
        struct object *obj;
        struct object_kind *kind;
        int slot;
        if (count == ITEMS_LIMIT) fail();
        numbers(line, "item", 10, items[count]);
        for (i = 0; i < 10; ++i)
            if (items[count][i] < -32768 || items[count][i] > 32767) fail();
        kind = NULL;
        for (i = 1; i < z_info->k_max; ++i)
            if (k_info[i].tval == items[count][0] && k_info[i].sval == items[count][1]) kind = &k_info[i];
        if (!kind || items[count][2] < 1 || items[count][2] > 40 ||
            items[count][3] < -1 || items[count][3] >= player->body.count ||
            items[count][8] < 0 || items[count][9] < 0 || items[count][9] > 1) fail();
        obj = object_new();
        object_prep(obj, kind, 0, MINIMISE);
        if (!plain(obj)) fail();
        obj->number = items[count][2];
        obj->to_h = items[count][4]; obj->to_d = items[count][5]; obj->to_a = items[count][6];
        obj->pval = items[count][7]; obj->timeout = items[count][8];
        /* Bound the supported ordinary tier rather than expose wizard kits. */
        if (abs(obj->to_h) > 20 || abs(obj->to_d) > 20 || abs(obj->to_a) > 20 || obj->pval < 0) fail();
        slot = items[count][3];
        if (slot >= 0) {
            int natural = wield_slot(obj);
            if (slots[slot] || obj->number != 1 || natural < 0 || natural >= player->body.count ||
                player->body.slots[slot].type != player->body.slots[natural].type) fail();
            slots[slot] = true;
        } else if (++pack_count > z_info->pack_size) fail();
        for (i = 0; i < count; ++i)
            if (prepared[i]->kind == kind && items[i][9] != items[count][9]) fail();
        prepared[count++] = obj;
    }
    if (!line || !streq(line, "end") || strtok_r(NULL, "\n", &context) || !count) fail();
    /* All parsing/validation precedes replacing the birth kit. Reuse native
     * gear/bonus mechanics, with no simulated observations or saved state. */
    for (i = 0; i < player->body.count; ++i) player->body.slots[i].obj = NULL;
    object_pile_free(NULL, NULL, player->gear);
    object_pile_free(NULL, NULL, player->gear_k);
    player->gear = player->gear_k = NULL;
    player->upkeep->equip_cnt = player->upkeep->total_weight = 0;
    memset(player->upkeep->inven, 0, (z_info->pack_size + 1) * sizeof(struct object *));
    memset(player->upkeep->quiver, 0, z_info->quiver_size * sizeof(struct object *));
    player->upkeep->inven_cnt = player->upkeep->quiver_cnt = 0;
    player->lev = player->max_lev = identity[2];
    player->exp = player->max_exp = vitals[0]; player->au = vitals[1];
    for (i = 0; i < STAT_MAX; ++i)
        player->stat_max[i] = player->stat_cur[i] = player->stat_birth[i] = stats[i];
    for (i = 0; i < PY_MAX_LEVEL; ++i) player->player_hp[i] = hp[i];
    player->obj_k->to_h = knowledge[0]; player->obj_k->to_d = knowledge[1]; player->obj_k->to_a = knowledge[2];
    for (j = 0; j < count; ++j) {
        struct object *obj = prepared[j];
        obj->known = object_new();
        object_set_base_known(player, obj);
        obj->kind->aware = items[j][9] != 0;
        obj->known->notice |= OBJ_NOTICE_ASSESSED;
        obj->known->pval = obj->pval;
        if (obj->kind->aware) obj->known->effect = obj->effect;
        obj->origin = ORIGIN_BIRTH;
        gear_insert_end(player, obj);
        player->upkeep->total_weight += obj->number * object_weight_one(obj);
        if (items[j][3] >= 0) {
            player->body.slots[items[j][3]].obj = obj;
            player->upkeep->equip_cnt++;
        }
    }
    update_player_object_knowledge(player);
    player->upkeep->update |= PU_BONUS | PU_INVEN | PU_HP | PU_MANA | PU_TORCH;
    update_stuff(player);
    if (player->mhp != vitals[2] || player->msp != vitals[3] || pack_is_overfull()) fail();
    player->chp = player->mhp; player->csp = player->msp;
    player->chp_frac = player->csp_frac = 0;
}
