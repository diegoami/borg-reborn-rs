/* Observation sinks for unit executables, which link the instrumented core
 * but do not run the borgh frontend. Never linked into the game executable. */
#ifdef BORG_HARNESS
#include "main-borgh.h"

void borgh_setting(int index, const char *name, int value)
{
    (void)index;
    (void)name;
    (void)value;
}

void borgh_decision(uint32_t rng) { (void)rng; }
void borgh_rng(uint32_t rng) { (void)rng; }
#endif
