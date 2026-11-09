/* Additional read-only combat observations in a separate historical export.
 * Same licensing as native-replay.h. Never changes decisions, RNG or inputs. */
static FILE *combat_stream;
static int combat_previous_speed = -1;
static const char *combat_message_names[] = {
#define MSG(a, b) #a,
#include "list-message.h"
#undef MSG
};

static bool combat_ready(void)
{
    if (!directory || snapshot_child || !character_generated || !player) return false;
    if (!combat_stream) combat_stream = open_record(directory, "combat.jsonl", "w");
    return true;
}

static void combat_string(const char *text)
{
    const unsigned char *p = (const unsigned char *)(text ? text : "");
    fputc('"', combat_stream);
    for (; *p; ++p) {
        if (*p == '"' || *p == '\\') {
            fputc('\\', combat_stream); fputc(*p, combat_stream);
        } else if (*p < 32) fprintf(combat_stream, "\\u%04x", *p);
        else fputc(*p, combat_stream);
    }
    fputc('"', combat_stream);
}

void combat_message(const char *text, uint16_t type)
{
    if (!combat_ready()) return;
    fprintf(combat_stream, "{\"kind\":\"message\",\"frame\":%llu,\"turn\":%ld,\"type\":",
        frame_count + 1, (long)turn);
    combat_string(type < N_ELEMENTS(combat_message_names) ? combat_message_names[type] : "UNKNOWN");
    fprintf(combat_stream, ",\"text\":"); combat_string(text);
    fprintf(combat_stream, "}\n"); fflush(combat_stream);
}

void combat_damage(int requested, int before, int after, const char *cause)
{
    if (before == after || !combat_ready()) return;
    fprintf(combat_stream, "{\"kind\":\"damage\",\"frame\":%llu,\"turn\":%ld,\"before\":%d,\"after\":%d,\"requested\":%d,\"cause\":",
        frame_count + 1, (long)turn, before, after, requested);
    combat_string(cause); fprintf(combat_stream, "}\n"); fflush(combat_stream);
}

void combat_speed(void)
{
    if (!combat_ready() || combat_previous_speed == player->state.speed) return;
    combat_previous_speed = player->state.speed;
    fprintf(combat_stream, "{\"kind\":\"speed\",\"frame\":%llu,\"turn\":%ld,\"speed\":%d,\"gain\":%d,\"move\":%d}\n",
        frame_count, (long)turn, player->state.speed,
        turn_energy(player->state.speed), z_info->move_energy);
    fflush(combat_stream);
}
