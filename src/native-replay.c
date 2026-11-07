/* Adapted from diegoami/better-borg-c 08a5cd019, tools/native-replay.c.
 * Native recording of observations and external input. Same licensing as
 * native-replay.h. Replay never supplies game decisions or resets game RNG. */
#include "native-replay.h"
#include "player.h"
#include "game-world.h"
#include "savefile.h"
#include "z-color.h"
#include "object.h"
#include "monster.h"
#include "player-timed.h"
#include "option.h"
#include "z-quark.h"
#include "init.h"
#include <sys/wait.h>
#include <unistd.h>

static FILE *inputs, *clocks, *actions, *frames, *file_inputs, *native_messages;
static FILE *replay_inputs, *replay_clocks, *replay_files;
static const char *directory;
static unsigned long long input_count, frame_count;
static unsigned int *previous_chars;
static int *previous_attrs;
static int previous_width, previous_height;
static uint8_t previous_palette[MAX_COLORS][4];
static unsigned int file_count;
static bool snapshot_child;
static unsigned long long previous_details;
static long key_count;
static long original_pid;
long record_key_count(void) { return key_count; }
long record_pid(void) { return directory ? original_pid : (long)getpid(); }
/* Link wrapping includes the Rust static library's libc time() calls. */
extern time_t __real_time(time_t *target);
time_t __wrap_time(time_t *target) { return record_time(target); }
static bool snapshot(const char *path)
{
    pid_t child = fork();
    int status;
    if (child < 0) return false;
    if (!child) {
        snapshot_child = true;
        _exit(savefile_save(path) ? 0 : 1);
    }
    while (waitpid(child, &status, 0) < 0)
        if (errno != EINTR) return false;
    return WIFEXITED(status) && WEXITSTATUS(status) == 0;
}

bool record_in_snapshot(void) { return snapshot_child; }

static FILE *open_record(const char *base, const char *name, const char *mode)
{
    char path[2048];
    FILE *stream;
    strnfmt(path, sizeof(path), "%s/%s", base, name);
    stream = fopen(path, mode);
    if (!stream) quit_fmt("Cannot open recording %s", path);
    return stream;
}

void record_init(void)
{
    const char *original = getenv("BORG_REPLAY_DIR");
    directory = getenv("BORG_RECORD_DIR");
    if (!directory) return;
    if (getenv("BORG_REPLAY_SYS")) ANGBAND_SYS = getenv("BORG_REPLAY_SYS");
    original_pid = (long)getpid();
    if (original) {
        FILE *pid = open_record(original, "pid.txt", "r");
        if (fscanf(pid, "%ld", &original_pid) != 1 || original_pid < 1)
            quit("Replay RNG initialization PID missing");
        fclose(pid);
    }
    {
        FILE *pid = open_record(directory, "pid.txt", "w");
        fprintf(pid, "%ld\n", original_pid);
        fclose(pid);
    }
    atexit(record_finish);
    inputs = open_record(directory, "events.txt", "w");
    clocks = open_record(directory, "clock.txt", "w");
    actions = open_record(directory, "actions.txt", "w");
    frames = open_record(directory, "frames.jsonl", "w");
    native_messages = open_record(directory, "native-messages.jsonl", "w");
    file_inputs = open_record(directory, "file-inputs.txt", "w");
    {
        FILE *geometry = open_record(directory, "geometry.json", "w");
        fprintf(geometry, "{\"width\":%d,\"height\":%d,\"key_size\":%d,\"soft_cursor\":%d,\"complex_input\":%d}\n", Term->wid, Term->hgt, Term->key_size, Term->soft_cursor, Term->complex_input);
        fclose(geometry);
    }
    if (original) {
        replay_inputs = open_record(original, "events.txt", "r");
        replay_clocks = open_record(original, "clock.txt", "r");
        replay_files = open_record(original, "file-inputs.txt", "r");
        setvbuf(replay_inputs, NULL, _IONBF, 0);
        setvbuf(replay_clocks, NULL, _IONBF, 0);
        setvbuf(replay_files, NULL, _IONBF, 0);
    }
}

/* Capture preferences at READ time, including later borg.txt reloads and
 * explicitly loaded .prf files. Replay does not depend on those external files. */
const char *record_read_path(const char *path)
{
    static char redirected[2048];
    FILE *source, *copy;
    unsigned char buffer[8192];
    size_t count;
    int present;
    const char *original = getenv("BORG_REPLAY_DIR");
    if (!file_inputs || snapshot_child) return path;
    ++file_count;
    if (replay_files) {
        if (fscanf(replay_files, "%d", &present) != 1)
            quit("Replay file input exhausted");
        strnfmt(redirected, sizeof(redirected), "%s/file-input-%06u.bin", original, file_count);
        source = present ? fopen(redirected, "rb") : NULL;
        if (present && !source) quit("Replay file input missing");
    } else {
        source = fopen(path, "rb");
        present = source != NULL;
    }
    fprintf(file_inputs, "%d\n", present);
    fflush(file_inputs);
    if (present) {
        char output[2048];
        strnfmt(output, sizeof(output), "%s/file-input-%06u.bin", directory, file_count);
        copy = fopen(output, "wb");
        if (!copy) quit("Cannot preserve file input");
        while ((count = fread(buffer, 1, sizeof(buffer), source)) != 0)
            if (fwrite(buffer, 1, count, copy) != count) quit("Recording file input write failed");
        if (ferror(source)) quit("Recording file input read failed");
        fclose(copy); fclose(source);
    }
    if (!replay_files) return path;
    if (!present) strnfmt(redirected, sizeof(redirected), "%s/missing-file-%06u", directory, file_count);
    return redirected;
}

time_t record_time(time_t *target)
{
    time_t actual = __real_time(NULL);
    if (clocks && !snapshot_child) {
        long long value = (long long)actual;
        if (replay_clocks && fscanf(replay_clocks, "%lld", &value) != 1)
            quit("Replay calendar input exhausted");
        actual = (time_t)value;
        fprintf(clocks, "%lld\n", value);
        fflush(clocks);
    }
    if (target) *target = actual;
    return actual;
}

/* Terminal input is an external input stream, including empty polls, peek/take
 * semantics and disconnects. Replaying it leaves Borg decision code running. */
bool record_input_replay(ui_event *event, bool wait, bool take, int *result)
{
    int w, t, type, mods, x, y, button, head, tail, disconnect, width, height, mapped;
    unsigned int code;
    if (!replay_inputs) return false;
    if (fscanf(replay_inputs, "%d %d %d %d %u %d %d %d %d %d %d %d %d %d %d",
        &w, &t, result, &type, &code, &mods, &x, &y, &button, &head, &tail,
        &disconnect, &width, &height, &mapped) != 15)
        quit("Replay input exhausted or malformed");
    if (w != wait || t != take || width < 1 || width > 255 || height < 1 || height > 255
        || head < 0 || head >= Term->key_size || tail < 0 || tail >= Term->key_size)
        quit("Replay terminal call/geometry diverged");
    if (width != Term->wid || height != Term->hgt) Term_resize(width, height);
    memset(event, 0, sizeof(*event));
    event->type = type;
    if (type == EVT_MOUSE) {
        event->mouse.x = x; event->mouse.y = y;
        event->mouse.button = button; event->mouse.mods = mods;
    } else {
        event->key.code = code; event->key.mods = mods;
    }
    Term->key_head = head;
    Term->key_tail = tail;
    terms_disconnecting = disconnect;
    Term->mapped_flag = mapped;
    return true;
}

void record_input_result(const ui_event *event, bool wait, bool take, int result)
{
    int mouse = event->type == EVT_MOUSE;
    if (!inputs) return;
    ++input_count;
    fprintf(inputs, "%d %d %d %d %u %d %d %d %d %d %d %d %d %d %d\n",
        wait, take, result, event->type, mouse ? 0 : (unsigned int)event->key.code,
        mouse ? event->mouse.mods : event->key.mods,
        mouse ? event->mouse.x : 0, mouse ? event->mouse.y : 0,
        mouse ? event->mouse.button : 0, Term->key_head, Term->key_tail,
        terms_disconnecting, Term->wid, Term->hgt, Term->mapped_flag);
    fflush(inputs);
    if (wait && !result && character_generated)
        record_frame("input", record_key_count());
}

struct keypress record_borg_key(struct keypress key)
{
    if (actions) {
        ++key_count;
        fprintf(actions, "%ld %ld %d %u %u\n", (long)turn,
            record_key_count(), key.type, (unsigned int)key.code, key.mods);
        fflush(actions);
    }
    return key;
}

/* Saving compacts monsters. The forked saver protects live memory and
 * RNG. Hash the serialized world in the child-produced file, not C padding or
 * pointers. The rolling file is a fingerprint input, not a resumable checkpoint. */
static unsigned long long world_hash(void)
{
    char path[2048];
    unsigned char buffer[8192];
    size_t count, i;
    FILE *saved;
    unsigned long long hash = 14695981039346656037ULL;
    strnfmt(path, sizeof(path), "%s/world-fingerprint.save", directory);
    if (!snapshot(path)) quit("Recording world fingerprint failed");
    saved = fopen(path, "rb");
    if (!saved) quit("Cannot read recording fingerprint");
    while ((count = fread(buffer, 1, sizeof(buffer), saved)) != 0)
        for (i = 0; i < count; i++) hash = (hash ^ buffer[i]) * 1099511628211ULL;
    if (ferror(saved)) quit("Recording fingerprint read failed");
    fclose(saved);
    return hash;
}

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

static unsigned long long diagnostic_hash(void)
{
    const struct object *obj;
    unsigned long long hash = 14695981039346656037ULL;
    int i;
    for (i = 0; i < TMD_MAX; i++) hash = mix_number(hash, player->timed[i]);
    for (obj = player->gear; obj; obj = obj->next) {
        hash = mix_number(hash, obj->kind ? obj->kind->kidx : 0);
        hash = mix_number(hash, obj->known && obj->known->kind ? obj->known->kind->kidx : 0);
        hash = mix_number(hash, obj->kind ? obj->kind->aware : 0);
        hash = mix_number(hash, obj->number); hash = mix_number(hash, obj->weight);
        hash = mix_number(hash, obj->pval); hash = mix_number(hash, obj->timeout);
        hash = mix_number(hash, obj->to_h); hash = mix_number(hash, obj->to_d);
        hash = mix_number(hash, obj->to_a); hash = mix_number(hash, obj->note);
    }
    if (cave) for (i = 1; i < cave->mon_max; i++) {
        const struct monster *mon = &cave->monsters[i];
        if (!mon->race) continue;
        hash = mix_number(hash, i); hash = mix_number(hash, mon->race->ridx);
        hash = mix_number(hash, mon->grid.x); hash = mix_number(hash, mon->grid.y);
        hash = mix_number(hash, mon->hp); hash = mix_number(hash, mon->maxhp);
        hash = mix_number(hash, mon->mspeed);
        hash = mix_number(hash, mflag_has(mon->mflag, MFLAG_VISIBLE));
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

void record_message(const char *text, uint16_t type)
{
    if (!native_messages || snapshot_child) return;
    fprintf(native_messages, "{\"frame\":%llu,\"turn\":%ld,\"type\":%u,\"text\":",
        frame_count, (long)turn, type);
    json_string_to(native_messages, text);
    fprintf(native_messages, "}\n");
    fflush(native_messages);
}

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

void record_frame(const char *phase, long decisions)
{
    int x, y, i, full, comma = 0;
    unsigned long long detail_hash;
    term *screen = angband_term[0];
    if (!frames || !character_generated || !screen) return;
    ++frame_count;
    full = screen->wid != previous_width || screen->hgt != previous_height
        || frame_count % 100 == 1;
    if (screen->wid != previous_width || screen->hgt != previous_height) {
        mem_free(previous_chars); mem_free(previous_attrs);
        previous_chars = mem_zalloc(screen->wid * screen->hgt * sizeof(*previous_chars));
        previous_attrs = mem_zalloc(screen->wid * screen->hgt * sizeof(*previous_attrs));
        previous_width = screen->wid; previous_height = screen->hgt;
    }
    fprintf(frames, "{\"index\":%llu,\"phase\":\"%s\",\"turn\":%ld,\"decisions\":%ld,\"input\":%llu,",
        frame_count, phase, (long)turn, decisions, input_count);
    fprintf(frames, "\"width\":%d,\"height\":%d,\"hp\":%d,\"sp\":%d,\"level\":%d,\"depth\":%d,\"dead\":%d,\"x\":%d,\"y\":%d,",
        screen->wid, screen->hgt, player->chp, player->csp, player->lev,
        player->depth, player->is_dead, player->grid.x, player->grid.y);
    fprintf(frames, "\"max_depth\":%d,\"max_level\":%d,\"exp\":%lu,\"gold\":%lu,\"energy\":%lu,",
        player->max_depth, player->max_lev, (unsigned long)player->exp,
        (unsigned long)player->au, (unsigned long)player->total_energy);
    fprintf(frames, "\"max_hp\":%d,\"max_sp\":%d,", player->mhp, player->msp);
    fprintf(frames, "\"race_idx\":%d,\"class_idx\":%d,\"cheat_live\":%d,\"noscore\":%u,\"cause\":",
        player->race->ridx, player->class->cidx, OPT(player, cheat_live), player->noscore);
    json_string(player->is_dead ? player->died_from : "");
    fprintf(frames, ",");
    if (streq(phase, "tick")) fprintf(frames, "\"world\":null,");
    else fprintf(frames, "\"world\":\"%016llx\",", world_hash());
    fprintf(frames, "\"quick\":%d,\"quick_rng\":%lu,\"rng_index\":%lu,\"rng\":\"",
        Rand_quick, (unsigned long)Rand_value, (unsigned long)state_i);
    for (i = 0; i < RAND_DEG; i++) fprintf(frames, "%08lx", (unsigned long)STATE[i]);
    fprintf(frames, "\",\"full\":%d,\"cells\":[", full);
    for (y = 0; y < screen->hgt; y++) for (x = 0; x < screen->wid; x++) {
        int at = y * screen->wid + x;
        unsigned int c = screen->scr->c[y][x];
        int attr = screen->scr->a[y][x];
        if (full || c != previous_chars[at] || attr != previous_attrs[at]) {
            fprintf(frames, "%s[%d,%u,%d]", comma ? "," : "", at, c, attr);
            comma = 1;
        }
        previous_chars[at] = c; previous_attrs[at] = attr;
    }
    fprintf(frames, "],\"palette\":");
    if (full || memcmp(previous_palette, angband_color_table, sizeof(previous_palette))) {
        fprintf(frames, "[");
        for (i = 0; i < MAX_COLORS; i++) fprintf(frames, "%s[%u,%u,%u]", i ? "," : "",
            angband_color_table[i][1], angband_color_table[i][2], angband_color_table[i][3]);
        fprintf(frames, "]");
        memcpy(previous_palette, angband_color_table, sizeof(previous_palette));
    } else fprintf(frames, "null");
    detail_hash = diagnostic_hash();
    fprintf(frames, ",\"details\":");
    if (full || detail_hash != previous_details) diagnostics();
    else fprintf(frames, "null");
    previous_details = detail_hash;
    replay_extra_observations(full);
    fprintf(frames, "}\n");
    fflush(frames);
}

void record_finish(void)
{
    FILE *done;
    if (!directory) return;
    if (native_messages && ferror(native_messages)) return;
    if (replay_inputs) {
        int ch;
        FILE *streams[] = {replay_inputs, replay_clocks, replay_files};
        int i;
        for (i = 0; i < 3; ++i) {
            do { ch = fgetc(streams[i]); } while (ch != EOF && isspace(ch));
            if (ch != EOF) return; /* Missing footer makes verification fail. */
        }
    }
    /* Python compares COMPLETE streams after process cleanup, including any
     * final external inputs used by shutdown hooks. */
    done = open_record(directory, "record-complete.json", "w");
    fprintf(done, "{\"inputs\":%llu,\"frames\":%llu}\n", input_count, frame_count);
    fclose(done);
}
