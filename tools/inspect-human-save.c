/* Linked against the historical image, never against today's struct layout.
 * GPL-2.0-or-later or Angband licence; see angband/docs/copying.rst. */
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
extern bool __real_savefile_load(const char *, bool);
extern void borgh_human_dump(FILE *);
bool __wrap_savefile_load(const char *path, bool cheat)
{
    if (!__real_savefile_load(path, cheat)) _Exit(2);
    borgh_human_dump(stdout);
    fflush(stdout);
    _Exit(0);
}
