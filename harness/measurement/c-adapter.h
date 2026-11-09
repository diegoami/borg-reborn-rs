/* Harness-only compiler include for borg.c. No gameplay or RNG draws. */
#include "borg/borg.h"
#include "borg/borg-init.h"
#include "borg/borg-think.h"
#include "main-borgh.h"
static bool borgh_think_adapter(void)
{
    bool result;
    int i;
    for (i = 0; i < BORG_MAX_SETTINGS; ++i)
        borgh_setting(i, borg_settings[i].setting_string, borg_cfg[i]);
    borgh_decision(borg_rand_local);
    result = borg_think();
    for (i = 0; i < BORG_MAX_SETTINGS; ++i)
        borgh_setting(i, borg_settings[i].setting_string, borg_cfg[i]);
    borgh_rng(Rand_value);
    return result;
}
#define borg_think() borgh_think_adapter()
