/**
 * \file main-borgh.h
 * \brief Hooks for the deterministic headless Borg harness frontend.
 *
 * Only compiled with BORG_HARNESS (cmake -DSUPPORT_BORG_HARNESS=ON).
 * Not part of upstream Angband; added by the borg-rs project.
 *
 * This work is free software; you can redistribute it and/or modify it
 * under the terms of the GNU General Public License as published by the
 * Free Software Foundation, version 2.
 */
#ifndef INCLUDED_MAIN_BORGH_H
#define INCLUDED_MAIN_BORGH_H

#ifdef BORG_HARNESS

#include "h-basic.h"
#include "ui-event.h"

/*
 * Fixed seed for the game RNG (defined in z-rand.c).  When borgh_seed_set
 * is true, Rand_init() seeds from borgh_seed instead of time()/getpid(),
 * and get_mon_num() uses a fixed (non-holiday) date.
 */
extern bool borgh_seed_set;
extern uint32_t borgh_seed;

/*
 * Called by inkey_ex() for every keypress returned by inkey_hack
 * (defined in ui-input.c, set by the harness frontend).
 */
extern void (*borgh_key_hook)(struct keypress k);

#endif /* BORG_HARNESS */

#endif /* INCLUDED_MAIN_BORGH_H */
