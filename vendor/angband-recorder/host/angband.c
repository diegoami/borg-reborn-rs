/* Engine structure adapter, derived from borg-rs 2411e52 / better-borg-c
 * 08a5cd019. GPL-2.0; original dual-licence notices in CREDITS.md. */
#include "native-replay.h"
#include "player.h"
#include "game-world.h"
#include "savefile.h"
#include "init.h"
#include <sys/wait.h>
#include <unistd.h>
#include <errno.h>
static long keys;
long record_key_count(void) { return keys; }
void record_init(void)
{
    int64_t geometry[5] = { Term->wid, Term->hgt, Term->key_size,
        Term->soft_cursor, Term->complex_input };
    if (getenv("BORG_REPLAY_SYS")) ANGBAND_SYS = getenv("BORG_REPLAY_SYS");
    ar_init(ANGBAND_RECORDER_BUILD, geometry);
}
int ar_host_save(const char *path)
{
    pid_t child = fork();
    int status;
    if (child < 0) return -1;
    if (!child) {
        ar_snapshot_enter();
        _exit(savefile_save(path) ? 0 : 1);
    }
    while (waitpid(child, &status, 0) < 0) if (errno != EINTR) return -1;
    return WIFEXITED(status) && WEXITSTATUS(status) == 0 ? 0 : -1;
}
static ar_input event_data(const ui_event *event, bool wait, bool take, int result)
{
    bool mouse = event->type == EVT_MOUSE;
    ar_input data = { wait, take, result, event->type,
        mouse ? 0 : event->key.code, mouse ? event->mouse.mods : event->key.mods,
        mouse ? event->mouse.x : 0, mouse ? event->mouse.y : 0,
        mouse ? event->mouse.button : 0, Term->key_head, Term->key_tail,
        terms_disconnecting, Term->wid, Term->hgt, Term->mapped_flag };
    return data;
}
bool record_input_replay(ui_event *event, bool wait, bool take, int *result)
{
    ar_input data;
    if (!ar_replaying() || getenv("BORG_TOOLKIT_HEADLESS")) return false;
    memset(&data, 0, sizeof(data));
    data.wait = wait; data.take = take;
    ar_poll(&data);
    if (data.head >= Term->key_size || data.tail >= Term->key_size)
        quit("Replay terminal queue diverged");
    if (data.width != Term->wid || data.height != Term->hgt)
        Term_resize(data.width, data.height);
    memset(event, 0, sizeof(*event));
    event->type = data.event_type;
    if (event->type == EVT_MOUSE) {
        event->mouse.x = data.x; event->mouse.y = data.y;
        event->mouse.button = data.button; event->mouse.mods = data.mods;
    } else { event->key.code = data.code; event->key.mods = data.mods; }
    Term->key_head = data.head; Term->key_tail = data.tail;
    terms_disconnecting = data.disconnect; Term->mapped_flag = data.mapped;
    *result = data.result;
    return true;
}
void record_input_result(const ui_event *event, bool wait, bool take, int result)
{
    if (!ar_active()) return;
    if (!ar_replaying() || getenv("BORG_TOOLKIT_HEADLESS")) {
        ar_input data = event_data(event, wait, take, result);
        ar_poll(&data);
    }
    if (wait && !result && character_generated) record_frame("input", keys);
}
struct keypress record_borg_key(struct keypress key)
{
    if (ar_active()) ar_bot(turn, ++keys, key.type, key.code, key.mods);
    return key;
}
static const char *message_names[] = {
#define MSG(a,b) #a,
#include "list-message.h"
#undef MSG
};
void record_message(const char *text, uint16_t type) {
    ar_message(turn, type, text);
    if (character_generated) ar_combat_message(turn, type < N_ELEMENTS(message_names) ? message_names[type] : "UNKNOWN", text);
}
void combat_damage(int requested, int before, int after, const char *cause) {
    if (character_generated) ar_damage(turn, requested, before, after, cause);
}
void record_frame(const char *phase, long decisions)
{
    ar_observation o;
    ar_cell *cells;
    term *screen = angband_term[0];
    int x, y;
    if (!ar_active() || !character_generated || !screen || !player) return;
    memset(&o, 0, sizeof(o));
    o.turn = turn; o.decisions = decisions; o.hp = player->chp; o.sp = player->csp;
    o.level = player->lev; o.depth = player->depth; o.gold = player->au;
    o.dead = player->is_dead; o.x = player->grid.x; o.y = player->grid.y;
    o.max_hp = player->mhp; o.max_sp = player->msp;
    o.quick = Rand_quick; o.quick_rng = Rand_value; o.rng_index = state_i;
    assert(RAND_DEG == 32);
    memcpy(o.rng, STATE, sizeof(o.rng));
    o.width = screen->wid; o.height = screen->hgt;
    cells = mem_alloc(o.width * o.height * sizeof(*cells));
    for (y = 0; y < o.height; ++y) for (x = 0; x < o.width; ++x) {
        cells[y * o.width + x].code = screen->scr->c[y][x];
        cells[y * o.width + x].attr = screen->scr->a[y][x];
    }
    o.cells = cells;
    ar_frame(phase, &o);
    mem_free(cells);
}
