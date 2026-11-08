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

/*
 * Called by message_add() for every message added to the message log,
 * repeats included, before the log is changed (defined in message.c, set
 * by the harness frontend).  It must only read.
 */
extern void (*borgh_message_hook)(const char *str, uint16_t type);

/* Optional checkpoint capture at the top-level game command boundary.
 * Defined by ui-game.c; NULL outside the headless harness. */
extern void (*borgh_command_hook)(void);

/* Harness build adapters; copies only, never retains Borg-owned pointers. */
extern void borgh_setting(int index, const char *name, int value);
extern void borgh_decision(uint32_t rng);
extern void borgh_rng(uint32_t rng);
extern void (*borgh_turn_hook)(void);

#endif /* BORG_HARNESS */

#endif /* INCLUDED_MAIN_BORGH_H */
