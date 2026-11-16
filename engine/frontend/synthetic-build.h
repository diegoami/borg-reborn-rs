#ifndef SYNTHETIC_BUILD_H
#define SYNTHETIC_BUILD_H
/* Declarative Human Warrior tiers, never native saves or Borg memory. */
void synthetic_build_apply(const char *path, char digest[65]);
void synthetic_build_capture(const char *path);
void synthetic_build_digest(const char *path, char digest[65]);
#endif
