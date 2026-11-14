/* Adapted from better-borg-c / borg-rs. GPL-2.0; see CREDITS.md. */
#ifndef ANGBAND_RECORDER_H
#define ANGBAND_RECORDER_H
#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>
/* Linux 64-bit v1 ABI. No Angband or Borg structures cross this boundary. */
typedef struct ar_input {
    int64_t wait, take, result, event_type, code, mods, x, y, button;
    int64_t head, tail, disconnect, width, height, mapped;
} ar_input;
typedef struct ar_cell { uint32_t code; int32_t attr; } ar_cell;
typedef struct ar_observation {
    int64_t turn, decisions, hp, sp, level, depth, gold, dead, x, y;
    int64_t max_hp, max_sp, quick, quick_rng, rng_index;
    uint32_t rng[32];
    int32_t width, height;
    const ar_cell *cells;
} ar_observation;
void ar_init(const char *build, const int64_t *geometry);
int64_t ar_pid(void);
bool ar_active(void);
bool ar_replaying(void);
bool ar_in_snapshot(void);
void ar_snapshot_enter(void);
void ar_poll(ar_input *input);
bool ar_exists(bool actual);
const char *ar_read_path(const char *path);
void ar_bot(int64_t turn, int64_t count, int64_t type, uint32_t code, uint8_t mods);
void ar_frame(const char *phase, const ar_observation *observation);
void ar_message(int64_t turn, uint16_t type, const char *message);
void ar_finish(void);
uint64_t ar_frame_count(void);
void ar_combat_message(int64_t turn, const char *name, const char *text);
void ar_damage(int64_t turn, int requested, int before, int after, const char *cause);
/* The host implements this callback by fork/save/wait. It must not alter the
 * parent game's memory. Returns 0 on success. Child calls ar_snapshot_enter. */
int ar_host_save(const char *path);
const char *ar_host_extra(bool full);
void ar_host_free_extra(const char *buffer);
#endif
