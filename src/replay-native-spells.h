/* Adapted from diegoami/better-borg-c 85b6ba8e3748e061db1ff1206163c27272d2de31, tools/native-spells.h. */
/* Player-known spell availability. Same licensing as native-replay.h. */
#include "cave.h"
#include "player-spell.h"
#include "player-timed.h"
#include "player-util.h"

struct replay_spell_row {
    const char *name;
    const char *status;
    char book[256];
    char book_label[2];
    char label[2];
    int level;
    int mana;
    int fail;
    bool castable;
};

static unsigned long long previous_spells;

static struct object *replay_spell_book(const struct class_book *book,
    bool *on_floor)
{
    struct object *obj;
    *on_floor = false;
    for (obj = player->gear; obj; obj = obj->next)
        if (player_object_to_book(player, obj) == book) return obj;
    if (cave) for (obj = square_object(cave, player->grid); obj; obj = obj->next) {
        if (obj->known && obj->known->kind == obj->kind
            && player_object_to_book(player, obj) == book) {
            *on_floor = true;
            return obj;
        }
    }
    return NULL;
}

static void replay_spells(bool full)
{
    int i, book, count = 0;
    int total = player->class->magic.total_spells;
    struct replay_spell_row *rows = mem_zalloc(MAX(1, total) * sizeof(*rows));
    unsigned long long hash = 14695981039346656037ULL;
    const unsigned char *bytes;
    size_t k;
    for (book = 0; book < player->class->magic.num_books; book++) {
        const struct class_book *b = &player->class->magic.books[book];
        bool on_floor;
        struct object *obj = replay_spell_book(b, &on_floor);
        for (i = 0; i < b->num_spells; i++) {
            const struct class_spell *s = &b->spells[i];
            struct replay_spell_row *row;
            unsigned int flags = player->spell_flags ? player->spell_flags[s->sidx] : 0;
            if (!(flags & (PY_SPELL_LEARNED | PY_SPELL_FORGOTTEN))) continue;
            row = &rows[count++];
            row->label[0] = all_letters_nohjkl[i];
            row->name = s->name; row->level = s->slevel;
            row->mana = s->smana; row->fail = spell_chance(s->sidx);
            if (obj) {
                struct replay_item_row description;
                replay_item(&description, obj, -1);
                my_strcpy(row->book, description.description, sizeof(row->book));
                row->book_label[0] = on_floor ? '-' : description.label[0];
            }
            if (player->is_dead) row->status = "Dead";
            else if (flags & PY_SPELL_FORGOTTEN) row->status = "Forgotten";
            else if (!obj) row->status = "No book";
            else if (!player_can_cast(player, false)) {
                row->status = player->timed[TMD_CONFUSED] ? "Confused" : "Cannot see";
            } else {
                row->castable = spell_okay_to_cast(player, s->sidx);
                row->status = s->smana > player->csp ? "Low mana" : "Ready";
            }
        }
    }
    bytes = (const unsigned char *)rows;
    for (k = 0; k < MAX(1, total) * sizeof(*rows); k++)
        hash = (hash ^ bytes[k]) * 1099511628211ULL;
    hash = mix_number(hash, player->csp); hash = mix_number(hash, player->msp);
    fprintf(frames, ",\"spells\":");
    if (full || hash != previous_spells) {
        fprintf(frames, "{\"class\":"); json_string(player->class->name);
        fprintf(frames, ",\"mana\":%d,\"max_mana\":%d,\"caster\":%d,\"learned\":[",
            player->csp, player->msp, total > 0);
        for (i = 0; i < count; i++) {
            struct replay_spell_row *row = &rows[i];
            fprintf(frames, "%s{\"name\":", i ? "," : ""); json_string(row->name);
            fprintf(frames, ",\"label\":"); json_string(row->label);
            fprintf(frames, ",\"book\":"); json_string(row->book);
            fprintf(frames, ",\"book_label\":"); json_string(row->book_label);
            fprintf(frames, ",\"level\":%d,\"mana\":%d,\"fail\":%d,\"castable\":%d,\"status\":",
                row->level, row->mana, row->fail, row->castable);
            json_string(row->status); fprintf(frames, "}");
        }
        fprintf(frames, "]}");
    } else fprintf(frames, "null");
    previous_spells = hash;
    mem_free(rows);
}
