#!/usr/bin/env bash
# Which order the C compiler evaluates a call's arguments in (#855).
#   harness/arg-order.sh [CC]      prints rtl or ltr
# The engine draws random numbers inside function arguments
# (loc(randint0(w), randint0(h)), mon-make.c, generate.c), and C leaves
# their order to the compiler: GCC on x86-64 evaluates right to left (rtl),
# clang everywhere and GCC on arm64 left to right (ltr). The references
# (harness/reference/) and the zoo's games are rtl's; an ltr build plays
# other games from the same seed. CC defaults to $CC, then cc, then gcc;
# compiled with -O2, as the Release build.
set -euo pipefail
CC_BIN="${1:-${CC:-$(command -v cc || command -v gcc)}}"
TMP=$(mktemp -d "${TMPDIR:-/tmp}/borg-arg-order.XXXXXXXX")
trap 'rm -rf "$TMP"' EXIT
cat > "$TMP/probe.c" <<'C'
#include <stdio.h>
static int calls;
__attribute__((noinline)) static int draw(void) { return ++calls; }
struct loc { int x, y; };
__attribute__((noinline)) static struct loc loc(int x, int y) { struct loc l = { x, y }; return l; }
int main(void) { struct loc l = loc(draw(), draw()); puts(l.x == 1 ? "ltr" : "rtl"); return 0; }
C
"$CC_BIN" -O2 -o "$TMP/probe" "$TMP/probe.c"
"$TMP/probe"
