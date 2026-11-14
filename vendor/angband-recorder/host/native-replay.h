/* Angband 4.2.6 adapter. GPL-2.0; see CREDITS.md. */
#ifndef NATIVE_REPLAY_H
#define NATIVE_REPLAY_H
#include "angband.h"
#include "ui-event.h"
#include "ui-term.h"
#include "recorder.h"
void record_init(void);
void record_frame(const char *phase, long decisions);
bool record_input_replay(ui_event *event, bool wait, bool take, int *result);
void record_input_result(const ui_event *event, bool wait, bool take, int result);
struct keypress record_borg_key(struct keypress key);
long record_key_count(void);
void combat_damage(int requested, int before, int after, const char *cause);
#define record_read_path ar_read_path
#define record_in_snapshot ar_in_snapshot
#define record_pid ar_pid
#define record_finish ar_finish
void record_message(const char *text, uint16_t type);
#endif
