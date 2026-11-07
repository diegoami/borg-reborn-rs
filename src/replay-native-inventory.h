/* Adapted from diegoami/better-borg-c 85b6ba8e3748e061db1ff1206163c27272d2de31, tools/native-inventory.h. */
/* Player-visible inventory observations. Same licensing as native-replay.h.
 * Descriptions use the game's knowledge rules, never ODESC_SPOIL. */
#include "ui-object.h"
#include "ui-menu.h"
#include "obj-desc.h"
#include "obj-gear.h"
#include "player-util.h"
#include "player-calcs.h"

struct replay_item_row {
    char label[2];
    char description[256];
    char slot[80];
    int attr;
    int weight;
};

static unsigned long long previous_inventory;

static void replay_item(struct replay_item_row *row, struct object *obj,
    int slot)
{
    bool kind_seen = obj && obj->kind ? obj->kind->everseen : false;
    bool ego_seen = obj && obj->ego ? obj->ego->everseen : false;
    memset(row, 0, sizeof(*row));
    if (slot >= 0) {
        row->label[0] = all_letters_nohjkl[slot];
        my_strcpy(row->slot, player->body.slots[slot].name, sizeof(row->slot));
    } else if (obj) row->label[0] = gear_to_label(player, obj);
    object_desc(row->description, sizeof(row->description), obj,
        ODESC_PREFIX | ODESC_FULL, player);
    /* object_desc normally marks these as seen. Observing a replay must not
     * change knowledge, even when called before the normal inventory screen. */
    if (obj && obj->kind) obj->kind->everseen = kind_seen;
    if (obj && obj->ego) obj->ego->everseen = ego_seen;
    row->attr = obj ? object_attr(obj) : COLOUR_SLATE;
    row->weight = obj ? obj->number * obj->weight : 0;
}

static void replay_rows(struct replay_item_row *rows, int count)
{
    int i;
    fprintf(frames, "[");
    for (i = 0; i < count; i++) {
        fprintf(frames, "%s{\"label\":", i ? "," : "");
        json_string(rows[i].label);
        fprintf(frames, ",\"description\":"); json_string(rows[i].description);
        fprintf(frames, ",\"slot\":"); json_string(rows[i].slot);
        fprintf(frames, ",\"attr\":%d,\"weight\":%d}", rows[i].attr, rows[i].weight);
    }
    fprintf(frames, "]");
}

static void replay_inventory(bool full)
{
    int i, inv = 0, eq = 0, quiv = 0;
    int capacity = z_info->pack_size + player->body.count + z_info->quiver_size;
    struct replay_item_row *rows = mem_zalloc(capacity * sizeof(*rows));
    unsigned long long hash = 14695981039346656037ULL;
    const unsigned char *bytes;
    size_t k;
    for (i = 0; i < z_info->pack_size; i++) {
        struct object *obj = player->upkeep->inven[i];
        if (obj) replay_item(&rows[inv++], obj, -1);
    }
    for (i = 0; i < player->body.count; i++)
        replay_item(&rows[inv + eq++], slot_object(player, i), i);
    for (i = 0; i < z_info->quiver_size; i++) {
        struct object *obj = player->upkeep->quiver[i];
        if (obj) replay_item(&rows[inv + eq + quiv++], obj, -1);
    }
    bytes = (const unsigned char *)rows;
    for (k = 0; k < capacity * sizeof(*rows); k++)
        hash = (hash ^ bytes[k]) * 1099511628211ULL;
    hash = mix_number(hash, inv); hash = mix_number(hash, eq);
    hash = mix_number(hash, player->upkeep->total_weight);
    hash = mix_number(hash, weight_remaining(player));
    fprintf(frames, ",\"inventory\":");
    if (full || hash != previous_inventory) {
        fprintf(frames, "{\"pack\":"); replay_rows(rows, inv);
        fprintf(frames, ",\"equipment\":"); replay_rows(rows + inv, eq);
        fprintf(frames, ",\"quiver\":"); replay_rows(rows + inv + eq, quiv);
        fprintf(frames, ",\"total_weight\":%d,\"remaining_weight\":%d}",
            player->upkeep->total_weight, weight_remaining(player));
    } else fprintf(frames, "null");
    previous_inventory = hash;
    mem_free(rows);
}
