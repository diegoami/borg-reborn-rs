#!/usr/bin/env python3
"""
Generate the data appendices (A1-A10) of the Angband strategy manual
(docs/c-manual/appendices/) directly from the game data files
lib/gamedata/*.txt and a few tables in src/*.c of an Angband source tree.

Usage (from the repository root):

    python3 docs/c-manual/tools/gen_appendices.py [--src ANGBAND_SRC] [--out DIR]

--src defaults to $ANGBAND_SRC, else to ./angband-src.  The manual was
generated from the `c-borg-reference` branch of this repository (angband/angband
e473720ed), e.g. `git worktree add ../angband-src origin/c-borg-reference`.  --out defaults to ../appendices relative
to this script.  Python 3 standard library only; output is deterministic for
a given source tree.

Every formula used here is a transcription of the C code; the function or
file it mirrors is named in a comment next to it and in the legend of the
generated appendix.
"""

import argparse
import os
import re
import subprocess
import sys
from collections import OrderedDict, defaultdict

HERE = os.path.dirname(os.path.abspath(__file__))
DEFAULT_SRC = os.environ.get("ANGBAND_SRC", "angband-src")
DEFAULT_OUT = os.path.normpath(os.path.join(HERE, "..", "appendices"))
SCRIPT_REL = "docs/c-manual/tools/gen_appendices.py"

SRC = DEFAULT_SRC
OUT = DEFAULT_OUT
COMMIT = ""
WARNINGS = []


def warn(msg):
    if msg not in WARNINGS:
        WARNINGS.append(msg)


# ---------------------------------------------------------------------------
# Generic parsing
# ---------------------------------------------------------------------------

def gd(fname):
    return os.path.join(SRC, "lib", "gamedata", fname)


def read_lines(fname):
    with open(gd(fname), encoding="utf-8") as f:
        for raw in f:
            line = raw.rstrip("\n").rstrip("\r")
            if not line.strip() or line.lstrip().startswith("#"):
                continue
            if ":" not in line:
                continue
            key, val = line.split(":", 1)
            yield key.strip(), val


def parse_records(fname, start_keys=("name",)):
    """Return records as lists of (key, value), split at any start key."""
    recs, cur = [], None
    for key, val in read_lines(fname):
        if key in start_keys:
            cur = [(key, val)]
            recs.append(cur)
        elif cur is not None:
            cur.append((key, val))
    return recs


def first(rec, key, default=None):
    for k, v in rec:
        if k == key:
            return v
    return default


def every(rec, key):
    return [v for k, v in rec if k == key]


def split_flags(vals):
    out = []
    for v in vals:
        out.extend(x.strip() for x in re.split(r"[|]", v) if x.strip())
    return out


def joined_text(vals):
    """Join desc: lines the way the game does (raw concatenation)."""
    s = "".join(vals)
    return re.sub(r"\s+", " ", s).strip()


def c_div(a, b):
    """C integer division (truncates toward zero)."""
    if b == 0:
        return 0
    q = abs(a) // abs(b)
    return q if (a >= 0) == (b >= 0) else -q


# ---------------------------------------------------------------------------
# Markdown helpers
# ---------------------------------------------------------------------------

_MD_ESC = re.compile(r"([\\|_*\[\]<>~`])")


def md(s):
    """Escape text for a markdown table cell / paragraph.

    Backtick code spans written by this script are kept (only `|` is escaped
    inside them, which GitHub tables require); everything else is escaped.
    """
    if s is None:
        return ""
    s = str(s).replace("\n", " ")
    if s.count("`") >= 2 and s.count("`") % 2 == 0:
        parts = s.split("`")
        out = []
        for i, part in enumerate(parts):
            if i % 2:
                out.append("`" + part.replace("|", "\\|") + "`")
            else:
                out.append(_MD_ESC.sub(r"\\\1", part))
        return "".join(out)
    return _MD_ESC.sub(r"\\\1", s)


def code(s):
    s = str(s)
    if "`" in s:
        return "`` " + s + " ``"
    return "`" + s + "`" if s else ""


def cell(s, empty="-"):
    s = md(s)
    return s if s else empty


def gh_anchor(text):
    """GitHub-style heading anchor (lower-case, drop punctuation)."""
    a = text.strip().lower()
    a = re.sub(r"[^\w\- ]", "", a, flags=re.UNICODE)
    return a.replace(" ", "-")


class Doc:
    """Accumulates the lines of one appendix and tracks heading anchors."""

    def __init__(self, fname, title, files, summary):
        self.fname = fname
        self.title = title
        self.files = files
        self.summary = summary
        self.body = []
        self.toc = []
        self.anchors = defaultdict(int)

    def anchor_for(self, heading):
        base = gh_anchor(heading)
        n = self.anchors[base]
        self.anchors[base] += 1
        return base if n == 0 else "%s-%d" % (base, n)

    def h(self, level, text, toc=True):
        a = self.anchor_for(text)
        self.body.append("%s %s\n" % ("#" * level, text))
        if toc and level <= 3:
            self.toc.append((level, text, a))
        return a

    def p(self, *lines):
        self.body.extend(lines)

    def table(self, header, aligns, rows):
        self.body.append("| " + " | ".join(header) + " |")
        self.body.append("|" + "|".join(aligns) + "|")
        for r in rows:
            self.body.append("| " + " | ".join(r) + " |")
        self.body.append("")

    def write(self, toc_depth=2):
        files = ", ".join("`%s`" % f for f in self.files)
        commit = (" at commit `%s`" % COMMIT) if COMMIT else ""
        head = [
            "# %s\n" % self.title,
            "> **Generated file - do not edit by hand.** Produced by "
            "`%s` from %s of the Angband source tree%s. Regenerate with "
            "`python3 %s --src <angband-source-root>` (run from the repository "
            "root).\n" % (SCRIPT_REL, files, commit, SCRIPT_REL),
        ]
        head += self.summary + [""]
        head.append("## Contents\n")
        for level, text, a in self.toc:
            if level > toc_depth + 1:
                continue
            indent = "  " * (level - 2)
            head.append("%s- [%s](#%s)" % (indent, text, a))
        head.append("")
        text = "\n".join(head + self.body).rstrip() + "\n"
        text = re.sub(r"\n{3,}", "\n\n", text)
        path = os.path.join(OUT, self.fname)
        with open(path, "w", encoding="utf-8") as f:
            f.write(text)
        n = text.count("\n")
        print("wrote %s (%d lines)" % (path, n))
        return n


# ---------------------------------------------------------------------------
# Dice and expressions (src/z-dice.c, src/z-expression.c, src/z-rand.c)
# ---------------------------------------------------------------------------

BASE_NAMES = {
    "SPELL_POWER": "power", "PLAYER_LEVEL": "plev", "DUNGEON_LEVEL": "dlev",
    "MAX_SIGHT": "sight", "WEAPON_DAMAGE": "weapon_roll", "PLAYER_HP": "chp",
    "MONSTER_PERCENT_HP_GONE": "target_%HP_lost",
}


def expr_text(base, ops):
    """Readable form of an expression: base then left-to-right ops."""
    s = BASE_NAMES.get(base, base.lower())
    toks = ops.split()
    i, last = 0, None
    while i < len(toks):
        op = toks[i]
        if op in ("n", "N"):
            s = "-(%s)" % s
            i += 1
            continue
        n = toks[i + 1] if i + 1 < len(toks) else "?"
        if (op in "+-" and n == "0") or (op in "*/" and n == "1"):
            i += 2
            continue
        if op in "*/" and last in ("+", "-"):
            s = "(%s)" % s
        s = "%s%s%s" % (s, op, n)
        last = op
        i += 2
    return s


def expr_eval(base_val, ops):
    """Mirror expression_evaluate(): left-to-right integer arithmetic."""
    toks = ops.split()
    v = base_val
    i = 0
    while i < len(toks):
        op = toks[i]
        if op in ("n", "N"):
            v = -v
            i += 1
            continue
        n = int(toks[i + 1])
        if op == "+":
            v += n
        elif op == "-":
            v -= n
        elif op == "*":
            v *= n
        elif op == "/":
            v = c_div(v, n)
        i += 2
    return v


_DICE_RE = re.compile(
    r"^(?P<neg>-)?(?:(?P<base>\$?[A-Z]+|\d+)(?=$|\+|[mM]))?\+?"
    r"(?:(?P<num>\$?[A-Z]+|\d+)?d(?P<sides>\$?[A-Z]+|\d+))?"
    r"(?:[mM](?P<mb>\$?[A-Z]+|\d+))?$")


def parse_dice(s):
    """Parse a dice string into dict(base, num, sides, m) of str/int or None."""
    s = s.strip().replace(" ", "")
    m = _DICE_RE.match(s)
    if not m:
        return None
    d = m.groupdict()
    if d["sides"] is not None and d["num"] is None:
        d["num"] = "1"
    return d


def dice_value(part, vars_):
    if part is None:
        return 0
    if part.startswith("$"):
        return vars_.get(part[1:], 0)
    if part.isdigit():
        return int(part)
    return 0


def dice_max(s, vars_, level=0):
    """Maximum of a dice string (randcalc MAXIMISE; m_bonus full at level)."""
    d = parse_dice(s)
    if d is None:
        return None
    b = dice_value(d["base"], vars_)
    n = dice_value(d["num"], vars_)
    sd = dice_value(d["sides"], vars_)
    mb = dice_value(d["mb"], vars_)
    v = b + (n * sd if n > 0 and sd > 0 else 0)
    if mb:
        # m_bonus_calc(MAXIMISE) returns the full bonus
        v += mb
    return -v if d["neg"] else v


def dice_avg(s, vars_):
    d = parse_dice(s)
    if d is None:
        return None
    b = dice_value(d["base"], vars_)
    n = dice_value(d["num"], vars_)
    sd = dice_value(d["sides"], vars_)
    v = b + (n * (sd + 1) / 2.0 if n > 0 and sd > 0 else 0)
    return -v if d["neg"] else v


def dice_text(dice, exprs):
    """Readable dice: '$Dd4' with D=(plev-1)/5+3 -> '((plev-1)/5+3)d4'."""
    if dice is None:
        return ""
    s = dice
    for var, (base, ops) in exprs.items():
        t = expr_text(base, ops)
        if re.fullmatch(r"[\w%]+", t):
            rep = t
        else:
            rep = "(" + t + ")"
        s = s.replace("$" + var, rep)
    return s


# ---------------------------------------------------------------------------
# Effects (src/list-effects.h, src/effects*.c)
# ---------------------------------------------------------------------------

class Effect:
    __slots__ = ("name", "sub", "rad", "other", "dice", "exprs", "yx", "msg")

    def __init__(self, spec):
        parts = spec.split(":")
        self.name = parts[0].strip()
        self.sub = parts[1].strip() if len(parts) > 1 else ""
        self.rad = parts[2].strip() if len(parts) > 2 else ""
        self.other = parts[3].strip() if len(parts) > 3 else ""
        self.dice = None
        self.exprs = OrderedDict()
        self.yx = None
        self.msg = None


def parse_effects(rec, eff="effect", dice="dice", expr="expr", yx="effect-yx",
                  start=None, stop=None):
    """Collect Effect objects from a record (optionally between two keys)."""
    out = []
    active = start is None
    for k, v in rec:
        if start is not None and k == start:
            active = True
            continue
        if stop is not None and k in stop:
            active = False
        if not active:
            continue
        if k == eff:
            out.append(Effect(v))
        elif k == dice and out:
            out[-1].dice = v.strip()
        elif k == expr and out:
            var, base, ops = (v.split(":", 2) + ["", ""])[:3]
            out[-1].exprs[var.strip()] = (base.strip(), ops.strip())
        elif k == yx and out:
            out[-1].yx = v.strip()
        elif k == "effect-msg" and out:
            out[-1].msg = v.strip()
    return out


class Ctx:
    """Name tables used to describe effects in words."""
    proj = {}       # code -> dict
    timed = {}      # code -> desc
    summon = {}     # code -> desc
    effdesc = {}    # effect name -> description template
    stats = {"STR": "strength", "INT": "intelligence", "WIS": "wisdom",
             "DEX": "dexterity", "CON": "constitution"}


def load_effect_descs():
    path = os.path.join(SRC, "src", "list-effects.h")
    txt = open(path, encoding="utf-8").read()
    txt = re.sub(r"\s+", " ", txt)
    for m in re.finditer(r'EFFECT\((\w+),[^"]*?"((?:[^"\\]|\\.)*)"', txt):
        Ctx.effdesc[m.group(1)] = m.group(2)


def pname(code_):
    p = Ctx.proj.get(code_)
    return p["name"] if p else code_.lower().replace("_", " ")


def pdesc(code_):
    p = Ctx.proj.get(code_)
    return p["desc"] if p else code_.lower().replace("_", " ")


_ENV_OF = {"LIGHT_WEAK": "of weak light", "DARK_WEAK": "of weak darkness",
           "KILL_WALL": "of stone to mud"}


def ofp(code_):
    """'of fire' for elements, 'that <desc>' for monster/environs projections."""
    p = Ctx.proj.get(code_)
    if p is None:
        return "of " + code_.lower().replace("_", " ")
    if p["type"] == "element":
        return "of " + p["name"]
    if code_ in _ENV_OF:
        return _ENV_OF[code_]
    return "that " + p["desc"]


def tname(code_):
    return Ctx.timed.get(code_, code_.lower().replace("_", " "))


def heal_text(e):
    d = parse_dice(e.dice or "")
    if d and d["mb"]:
        rest = (e.dice or "").split("m")[0].split("M")[0].rstrip("+")
        rest = dice_text(rest, e.exprs) or "0"
        return "heal %s HP or %s%% of HP lost, whichever is more" % (rest, d["mb"])
    return "heal %s HP" % dice_text(e.dice, e.exprs)


def plus_rad(e, default=None):
    r = e.rad or (default if default is not None else "0")
    try:
        o = int(e.other or 0)
    except ValueError:
        o = 0
    if o > 0:
        grow = "plev" if o == 1 else "plev/%d" % o
        return "radius %s" % (grow if r == "0" else "%s+%s" % (r, grow))
    return "radius %s" % r


def effect_phrase(e, dice_override=None):
    """One effect in words."""
    dt = dice_text(e.dice if e.dice is not None else dice_override, e.exprs)
    n, sub = e.name, e.sub
    dmg = (" for %s" % dt) if dt else ""
    if n == "HEAL_HP":
        if e.dice is None and dice_override:
            e2 = Effect("HEAL_HP")
            e2.dice = dice_override
            return heal_text(e2)
        return heal_text(e)
    if n == "CURE":
        return "cure %s" % tname(sub)
    if dt.startswith("-"):
        dt = "(" + dt + ")"
    if n == "TIMED_INC":
        return "+%s turns of %s" % (dt or "?", tname(sub))
    if n == "TIMED_INC_NO_RES":
        return "+%s turns of %s (unresistable)" % (dt or "?", tname(sub))
    if n == "TIMED_DEC":
        return "reduce %s by %s" % (tname(sub), dt or "?")
    if n == "TIMED_SET":
        return "set %s to %s turns" % (tname(sub), dt or "?")
    if n == "MON_TIMED_INC":
        return "monster gains %s turns of %s" % (dt or "?", sub.lower())
    if n == "NOURISH":
        how = {"INC_BY": "nourish +%s", "DEC_BY": "make hungrier by %s",
               "SET_TO": "set food to %s", "INC_TO": "nourish up to %s"}
        return (how.get(sub, "nourish %s") % (dt or "?")) + " (x food-value)"
    if n == "RESTORE_STAT":
        return "restore %s" % sub
    if n == "GAIN_STAT":
        return "increase %s" % sub
    if n == "DRAIN_STAT":
        return "drain %s" % sub
    if n == "LOSE_RANDOM_STAT":
        return "lose a random stat other than %s" % sub
    if n == "DAMAGE":
        return "%s damage to you" % (dt or "?")
    if n in ("BALL",):
        return "ball %s (%s)%s" % (ofp(sub), plus_rad(e, "2"), dmg)
    if n == "BREATH":
        w = max(int(e.other or 0), 20) if (e.other or "0").lstrip("-").isdigit() else e.other
        return "breath %s (%s-degree cone)%s" % (ofp(sub), w, dmg)
    if n == "ARC":
        w = max(int(e.other or 0), 20)
        rng = (", range %s" % e.rad) if e.rad and e.rad != "0" else ""
        return "arc %s (%s-degree%s)%s" % (ofp(sub), w, rng, dmg)
    if n == "SHORT_BEAM":
        return "short beam %s (length %s)%s" % (
            ofp(sub), plus_rad(e, "0").replace("radius ", ""), dmg)
    if n == "BOLT":
        return "bolt %s%s" % (ofp(sub), dmg)
    if n == "BEAM":
        return "beam %s%s" % (ofp(sub), dmg)
    if n == "BOLT_OR_BEAM":
        adj = ""
        if e.other and e.other != "0":
            adj = ", beam chance %+d" % int(e.other)
        return "bolt or beam %s%s%s" % (ofp(sub), adj, dmg)
    if n == "LINE":
        return "line %s%s" % (ofp(sub), dmg)
    if n == "SPOT":
        return "ball %s on yourself (%s)%s" % (ofp(sub), plus_rad(e, "0"), dmg)
    if n == "SPHERE":
        return "sphere %s around you (radius %s)%s" % (ofp(sub), e.rad or "0", dmg)
    if n == "STRIKE":
        return "strike %s at target (radius %s)%s" % (ofp(sub), e.rad or "0", dmg)
    if n == "STAR":
        return "beams %s in all 8 directions%s" % (ofp(sub), dmg)
    if n == "STAR_BALL":
        return "balls %s (radius %s) in all 8 directions%s" % (ofp(sub), e.rad or "0", dmg)
    if n == "SWARM":
        d = parse_dice(e.dice or "")
        cnt = d["mb"] if d and d["mb"] else "?"
        base = (e.dice or "").split("m")[0].split("M")[0].rstrip("+")
        return "swarm of %s balls %s (radius %s) for %s each" % (
            dice_text(cnt if not cnt.startswith("$") else cnt, e.exprs),
            ofp(sub), e.rad or "0", dice_text(base, e.exprs))
    if n == "LASH":
        return ("lash (length %s); element = `lash-type` of the first blow's effect "
                "in `blow_effects.txt`; damage = first blow + half of each other "
                "blow (`effect_handler_LASH()`)" % (e.rad or "?"))
    if n in ("PROJECT_LOS", "PROJECT_LOS_AWARE"):
        return "%s in line of sight%s" % (pdesc(sub), (" (power %s)" % dt) if dt else "")
    if n in ("TOUCH", "TOUCH_AWARE"):
        return "%s on adjacent grids (radius %s)%s" % (
            pdesc(sub), e.rad or "1", (" (power %s)" % dt) if dt else "")
    if n in ("BOLT_STATUS", "BOLT_AWARE"):
        return "bolt that %s%s" % (pdesc(sub), (" (power %s)" % dt) if dt else "")
    if n == "BOLT_STATUS_DAM":
        return "bolt that %s%s" % (pdesc(sub), dmg)
    if n == "ALTER":
        return "line that %s" % pdesc(sub)
    if n == "TELEPORT":
        d = parse_dice(e.dice or "")
        who = "the target away" if sub == "AWAY" else "self"
        if d and d["mb"]:
            return "teleport %s %s%% of the level's larger dimension" % (who, d["mb"])
        return "teleport %s up to %s grids" % (who, dt or "?")
    if n == "TELEPORT_TO":
        return "caster teleports next to the target" if sub == "SELF" else \
            "teleport the target next to the caster"
    if n == "TELEPORT_LEVEL":
        return "teleport level (up or down)"
    if n == "SUMMON":
        what = Ctx.summon.get(sub, sub.lower())
        return "summon %s x %s" % (dt or "1", what)
    if n == "DESTRUCTION":
        return "destruction (radius %s)" % (e.rad or "?")
    if n == "EARTHQUAKE":
        return "earthquake (radius %s)" % (e.rad or "?")
    if n == "ENCHANT":
        what = {"TOHIT": "to-hit", "TODAM": "to-dam", "TOBOTH": "to-hit and to-dam",
                "TOAC": "armour class"}.get(sub, sub)
        return "enchant %s (power %s)" % (what, dt or "?")
    if n == "RECHARGE":
        return "recharge a wand/staff (power %s)" % (dt or "?")
    if n == "REMOVE_CURSE":
        return "remove a curse (power %s)" % (dt or "?")
    if n == "RESTORE_MANA":
        return ("restore %s SP" % dt) if dt else "restore all SP"
    if n == "DRAIN_MANA":
        return "drain %s mana" % (dt or "some")
    if n == "GLYPH":
        return "inscribe a %s" % ("glyph of warding" if sub == "WARDING" else "decoy" if sub == "DECOY" else sub.lower())
    if n == "SHAPECHANGE":
        return "change into %s shape" % sub
    if n == "MELEE_BLOWS":
        return "%s blows on an adjacent monster; each damaging blow also %s (power %s)" % (
            dt or "?", pdesc(sub), e.rad or "0")
    if n == "SWEEP":
        return "%s blows against all adjacent monsters" % (dt or "?")
    if n == "MOVE_ATTACK":
        return "move up to 4 grids and make up to %s blows" % (dt or "?")
    if n == "GAIN_EXP":
        return "gain %s experience" % (dt or "?")
    if n == "MON_HEAL_HP":
        return "monster heals %s HP" % (dt or "?")
    if n == "MON_HEAL_KIN":
        return "heals a nearby kin monster by %s HP" % (dt or "?")
    if n == "WAKE":
        return "wake up nearby monsters"
    if n == "DARKEN_AREA":
        return "darken the area"
    if n == "TAP_UNLIFE":
        return "drain %s mana from nearest undead, damaging it" % (dt or "?")
    if n == "CURSE":
        return "damage a monster directly%s" % dmg
    if n in Ctx.effdesc:
        t = Ctx.effdesc[n]
        if "%" not in t:
            extra = []
            if e.yx:
                y, x = (e.yx.split(":") + ["?"])[:2]
                extra.append("area %s x %s" % (y, x))
            if sub:
                extra.append(sub.lower())
            if dt:
                extra.append(dt)
            return t + ((" (" + ", ".join(extra) + ")") if extra else "")
    s = n.lower()
    if sub:
        s += " " + sub.lower()
    if dt:
        s += " " + dt
    return s


def describe_effects(effects):
    """Readable sequence; handles RANDOM/SELECT groups and SET_VALUE."""
    out = []
    i = 0
    set_value = None
    while i < len(effects):
        e = effects[i]
        if e.name in ("RANDOM", "SELECT"):
            try:
                cnt = int(dice_max(e.dice or "0", {}) or 0)
            except (TypeError, ValueError):
                cnt = 0
            sub = effects[i + 1:i + 1 + cnt]
            inner = describe_effects(sub)
            word = "one at random of" if e.name == "RANDOM" else "your choice of"
            out.append("%s {%s}" % (word, " / ".join(inner)))
            i += 1 + cnt
            continue
        if e.name == "SET_VALUE":
            set_value = e.dice
            i += 1
            continue
        if e.name == "CLEAR_VALUE":
            set_value = None
            i += 1
            continue
        if e.name in ("SCRAMBLE_STATS", "UNSCRAMBLE_STATS"):
            out.append("scramble stats" if e.name == "SCRAMBLE_STATS" else "unscramble stats")
            i += 1
            continue
        ph = effect_phrase(e, None if e.dice is not None else set_value)
        out.append(ph)
        i += 1
    return out


def effects_text(effects):
    return "; ".join(describe_effects(effects))


# ---------------------------------------------------------------------------
# Loaders
# ---------------------------------------------------------------------------

def load_constants():
    c = {}
    for k, v in read_lines("constants.txt"):
        parts = v.split(":")
        c["%s:%s" % (k, parts[0])] = parts[-1]
    return c


def load_projections():
    out = OrderedDict()
    for r in parse_records("projection.txt", ("code",)):
        c = first(r, "code").strip()
        out[c] = {
            "code": c,
            "name": (first(r, "name") or c.lower()).strip(),
            "type": (first(r, "type") or "").strip(),
            "desc": (first(r, "desc") or "").strip(),
            "divisor": int((first(r, "divisor") or "0").strip() or 0),
            "cap": int((first(r, "damage-cap") or "0").strip() or 0),
            "num": (first(r, "numerator") or "").strip(),
            "den": (first(r, "denominator") or "").strip(),
        }
    return out


def load_timed():
    out = OrderedDict()
    for r in parse_records("player_timed.txt"):
        n = first(r, "name").strip()
        out[n] = {
            "name": n,
            "desc": (first(r, "desc") or "").strip(),
            "grades": [g.split(":") for g in every(r, "grade")],
            "fail": [f.split(":") for f in every(r, "fail")],
            "flags": split_flags(every(r, "flags")),
            "resist": [x.strip() for x in every(r, "resist")],
            "brand": [x.strip() for x in every(r, "brand")],
            "slay": [x.strip() for x in every(r, "slay")],
            "synonym": [x.strip() for x in every(r, "flag-synonym")],
            "on_end": (first(r, "on-end") or "").strip(),
            "lower": (first(r, "lower-bound") or "").strip(),
            "begin": parse_effects(r, eff="on-begin-effect", dice="effect-dice",
                                   expr="effect-expr"),
            "end": parse_effects(r, eff="on-end-effect", dice="effect-dice",
                                 expr="effect-expr"),
        }
    return out


def load_summons():
    out = OrderedDict()
    for r in parse_records("summon.txt"):
        n = first(r, "name").strip()
        out[n] = {
            "name": n, "desc": (first(r, "desc") or "").strip(),
            "uniques": (first(r, "uniques") or "0").strip(),
            "bases": [b.strip() for b in every(r, "base")],
            "race_flag": (first(r, "race-flag") or "").strip(),
            "fallback": (first(r, "fallback") or "").strip(),
        }
    return out


def load_race_flag_types():
    path = os.path.join(SRC, "src", "list-mon-race-flags.h")
    out = OrderedDict()
    for m in re.finditer(r'RF\((\w+),\s*(\w+),\s*"([^"]*)"\)',
                         open(path, encoding="utf-8").read()):
        out[m.group(1)] = (m.group(2), m.group(3))
    return out


def load_spell_types():
    path = os.path.join(SRC, "src", "list-mon-spells.h")
    out = OrderedDict()
    for m in re.finditer(r'RSF\((\w+),\s*([^)]*)\)', open(path, encoding="utf-8").read()):
        if m.group(1) in ("NONE", "MAX"):
            continue
        out[m.group(1)] = [x.strip() for x in m.group(2).split("|")]
    return out


def load_monster_spells(spell_types):
    spells = OrderedDict()
    for r in parse_records("monster_spell.txt"):
        name = first(r, "name").strip()
        levels = [{"power": 0, "lore": "", "save": ""}]
        for k, v in r:
            if k == "power-cutoff":
                levels.append({"power": int(v), "lore": "", "save": ""})
            elif k == "lore":
                levels[-1]["lore"] += v
            elif k == "message-save":
                levels[-1]["save"] += v
        spells[name] = {
            "name": name,
            "hit": int((first(r, "hit") or "100").strip()),
            "effects": parse_effects(r),
            "levels": levels,
            "types": spell_types.get(name, []),
        }
    return spells


def spell_level(sp, power):
    """mon_spell_lore_description(): advance while power >= next cutoff."""
    lv = sp["levels"][0]
    for nxt in sp["levels"][1:]:
        if power >= nxt["power"]:
            lv = nxt
        else:
            break
    return lv


def spell_is(sp, t):
    return ("RST_" + t) in sp["types"]


def spell_has_damage(sp):
    # RST_DAMAGE = RST_BOLT | RST_BALL | RST_BREATH | RST_DIRECT (mon-spell.h)
    return any(spell_is(sp, t) for t in ("BOLT", "BALL", "BREATH", "DIRECT"))


def mon_spell_max_damage(sp, mon, projs, hp=None):
    """Mirror mon_spell_dam(..., MAXIMISE) of src/mon-spell.c.

    Breaths: breath_dam() = min(hp / divisor, damage_cap) with hp = average HP
    (what monster lore shows).  Others: nonhp_dam(): sum of the effects' dice
    (TIMED_INC dice excluded); LASH: full first blow plus half of each other
    blow, blow dice maximised at the race level.
    """
    if not spell_has_damage(sp):
        return None
    if spell_is(sp, "BREATH"):
        el = sp["effects"][0].sub
        p = projs.get(el)
        h = mon["hp"] if hp is None else hp
        if not p or not p["divisor"] or h is None:
            return None
        return min(c_div(h, p["divisor"]), p["cap"])
    total = 0
    for e in sp["effects"]:
        if e.name == "LASH":
            for i, b in enumerate(mon["blows"]):
                bd = b[2]
                if not bd:
                    continue
                v = dice_max(bd, {}, mon["depth"]) or 0
                total += v if i == 0 else c_div(v, 2)
        elif e.dice and e.name != "TIMED_INC":
            vars_ = {}
            for var, (base, ops) in e.exprs.items():
                if base == "SPELL_POWER":
                    bv = mon["power"]
                elif base == "MAX_SIGHT":
                    bv = 20
                else:
                    bv = 0
                vars_[var] = expr_eval(bv, ops)
            v = dice_max(e.dice, vars_)
            if v is None:
                warn("unparsed monster spell dice %r in %s" % (e.dice, sp["name"]))
                v = 0
            total += v
    return total


def load_bases():
    out = OrderedDict()
    for r in parse_records("monster_base.txt"):
        n = first(r, "name").strip()
        out[n] = {
            "name": n, "glyph": (first(r, "glyph") or "?"),
            "flags": split_flags(every(r, "flags")),
            "desc": (first(r, "desc") or "").strip(),
        }
    return out


def mon_hp_max(avg):
    """mon_hp(MAXIMISE) of src/mon-make.c: avg + 4 * std_dev."""
    sd = c_div(c_div(avg * 10, 8) + 5, 10)
    if avg > 1:
        sd += 1
    return avg + 4 * sd


def load_monsters(bases):
    mons = []
    for r in parse_records("monster.txt"):
        name = first(r, "name").strip()
        bname = (first(r, "base") or "").strip()
        b = bases.get(bname)
        if b is None:
            warn("monster %s has unknown base %r" % (name, bname))
            b = {"glyph": "?", "flags": []}
        flags = list(b["flags"])
        for k, v in r:
            if k == "flags":
                for f in split_flags([v]):
                    if f not in flags:
                        flags.append(f)
            elif k == "flags-off":
                for f in split_flags([v]):
                    if f in flags:
                        flags.remove(f)
        spells = split_flags(every(r, "spells"))
        depth = int((first(r, "depth") or "0").strip())
        hp = first(r, "hit-points")
        blows = []
        for v in every(r, "blow"):
            p = v.split(":")
            blows.append((p[0].strip(), p[1].strip() if len(p) > 1 else "",
                          p[2].strip() if len(p) > 2 else ""))
        innate = first(r, "innate-freq")
        freq = first(r, "spell-freq")
        mons.append({
            "name": name, "base": bname,
            "glyph": (first(r, "glyph") or b["glyph"]).strip() or "?",
            "color": (first(r, "color") or "").strip(),
            "speed": int((first(r, "speed") or "110").strip()),
            "hp": int(hp) if hp is not None else None,
            "ac": int((first(r, "armor-class") or "0").strip()),
            "depth": depth,
            "rarity": int((first(r, "rarity") or "0").strip()),
            "exp": int((first(r, "experience") or "0").strip()),
            "sleep": (first(r, "sleepiness") or "").strip(),
            "hearing": (first(r, "hearing") or "").strip(),
            "smell": (first(r, "smell") or "").strip(),
            "light": (first(r, "light") or "").strip(),
            "blows": blows, "flags": flags,
            "innate": int(innate) if innate else None,
            "freq": int(freq) if freq else None,
            "power": int(first(r, "spell-power")) if first(r, "spell-power") else depth,
            "spells": spells,
            "drops": every(r, "drop"), "drop_base": every(r, "drop-base"),
            "friends": [("", x) for x in every(r, "friends")] +
                       [("base", x) for x in every(r, "friends-base")],
            "mimic": every(r, "mimic"),
            "shapes": [x.strip() for x in every(r, "shape")],
            "plural": (first(r, "plural") or "").strip(),
            "desc": joined_text(every(r, "desc")),
        })
    return mons


def resolve_spell_freqs(m, mspells):
    """parse_monster_spells(): a missing frequency defaults to 1_IN_4."""
    innate = m["innate"]
    freq = m["freq"]
    has_innate = any(spell_is(mspells[s], "INNATE") for s in m["spells"] if s in mspells)
    has_other = any(not spell_is(mspells[s], "INNATE") for s in m["spells"] if s in mspells)
    if has_innate and not innate:
        innate = 4
    if has_other and not freq:
        freq = 4
    return innate, freq


# ---------------------------------------------------------------------------
# Object data
# ---------------------------------------------------------------------------

def tv_norm(t):
    return t.strip().lower().replace("armour", "armor")


def clean_name(n):
    n = n.strip()
    if "|" in n:
        n = re.sub(r"\|([^|]*)\|([^|]*)\|", r"\1", n)
    return n.replace("&", "").replace("~", "").strip()


def load_object_bases():
    out = OrderedDict()
    for r in parse_records("object_base.txt"):
        v = first(r, "name")
        tv, _, nm = v.partition(":")
        out[tv_norm(tv)] = {
            "name": nm.replace("~", "").strip(),
            "flags": split_flags(every(r, "flags")),
            "break": (first(r, "break") or "").strip(),
            "max_stack": (first(r, "max-stack") or "").strip(),
        }
    return out


def load_objects():
    out = []
    for r in parse_records("object.txt"):
        out.append({
            "name": clean_name(first(r, "name")),
            "tval": tv_norm(first(r, "type") or ""),
            "level": int((first(r, "level") or "0").strip()),
            "weight": int((first(r, "weight") or "0").strip()),
            "cost": int((first(r, "cost") or "0").strip()),
            "alloc": (first(r, "alloc") or "").strip(),
            "attack": (first(r, "attack") or "").strip(),
            "armor": (first(r, "armor") or "").strip(),
            "charges": (first(r, "charges") or "").strip(),
            "time": (first(r, "time") or "").strip(),
            "pile": (first(r, "pile") or "").strip(),
            "power": (first(r, "power") or "").strip(),
            "pval": (first(r, "pval") or "").strip(),
            "flags": split_flags(every(r, "flags")),
            "values": split_flags(every(r, "values")),
            "slays": [x.strip() for x in every(r, "slay")],
            "brands": [x.strip() for x in every(r, "brand")],
            "curses": [x.strip() for x in every(r, "curse")],
            "effects": parse_effects(r),
            "msg": (first(r, "msg") or "").strip(),
            "desc": joined_text(every(r, "desc")),
        })
    return out


def load_activations():
    out = OrderedDict()
    for r in parse_records("activation.txt"):
        n = first(r, "name").strip()
        out[n] = {
            "name": n, "aim": (first(r, "aim") or "0").strip(),
            "level": (first(r, "level") or "").strip(),
            "power": (first(r, "power") or "").strip(),
            "effects": parse_effects(r),
            "desc": joined_text(every(r, "desc")),
        }
    return out


def load_slays():
    out = OrderedDict()
    for r in parse_records("slay.txt", ("code",)):
        c = first(r, "code").strip()
        out[c] = {"name": (first(r, "name") or "").strip(),
                  "mult": (first(r, "multiplier") or "").strip(),
                  "flag": (first(r, "race-flag") or first(r, "base") or "").strip()}
    return out


def load_brands():
    out = OrderedDict()
    for r in parse_records("brand.txt", ("code",)):
        c = first(r, "code").strip()
        out[c] = {"name": (first(r, "name") or "").strip(),
                  "mult": (first(r, "multiplier") or "").strip(),
                  "verb": (first(r, "verb") or "").strip(),
                  "resist": (first(r, "resist-flag") or "").strip(),
                  "vuln": (first(r, "vuln-flag") or "").strip()}
    return out


def load_obj_props():
    out = []
    for r in parse_records("object_property.txt"):
        out.append({
            "name": first(r, "name").strip(),
            "type": (first(r, "type") or "").strip(),
            "subtype": (first(r, "subtype") or "").strip(),
            "id": (first(r, "id-type") or "").strip(),
            "code": (first(r, "code") or "").strip(),
            "power": (first(r, "power") or "").strip(),
            "desc": joined_text(every(r, "desc")),
            "msg": (first(r, "msg") or "").strip(),
        })
    return out


def load_curses():
    out = OrderedDict()
    for r in parse_records("curse.txt"):
        n = first(r, "name").strip()
        out[n] = {
            "name": n, "types": [tv_norm(x) for x in every(r, "type")],
            "combat": (first(r, "combat") or "").strip(),
            "weight": (first(r, "weight") or "").strip(),
            "effects": parse_effects(r),
            "time": (first(r, "time") or "").strip(),
            "flags": split_flags(every(r, "flags")),
            "values": split_flags(every(r, "values")),
            "msg": (first(r, "msg") or "").strip(),
            "desc": joined_text(every(r, "desc")),
            "conflict": [x.strip() for x in every(r, "conflict")],
            "conflict_flags": split_flags(every(r, "conflict-flags")),
        }
    return out


def fmt_values(vals):
    """'SPEED[2+M8]' -> 'SPEED 2+M8'; RES_X[1] -> 'rRES X'."""
    out = []
    for v in vals:
        m = re.match(r"([A-Z_]+)\[(.*)\]", v)
        if not m:
            out.append(v)
            continue
        k, x = m.group(1), m.group(2)
        if k.startswith("RES_"):
            el = k[4:]
            lvl = x.strip()
            word = {"1": "resist", "2": "resist x2", "3": "immune",
                    "-1": "vulnerable"}.get(lvl, "res level " + lvl)
            out.append("%s %s" % (word, pname(el)))
        else:
            if re.fullmatch(r"-?\d+", x) and not x.startswith("-"):
                x = "+" + x
            out.append("%s %s" % (k, x))
    return out


def slay_text(codes, slays):
    out = []
    for c in codes:
        s = slays.get(c)
        out.append(("slay %s x%s" % (s["name"], s["mult"])) if s else "slay " + c)
    return out


def brand_text(codes, brands):
    out = []
    for c in codes:
        b = brands.get(c)
        out.append(("brand %s x%s" % (b["name"], b["mult"])) if b else "brand " + c)
    return out


def fmt_alloc(a):
    if not a:
        return "not random"
    p = a.split(":")
    if len(p) < 2:
        return a
    return "%s (x%s)" % (p[1].strip().replace(" to ", "-"), p[0].strip())


def fmt_weight(w):
    return "%d.%d lb" % (w // 10, w % 10)


MODS_NOTE = (
    "- **Modifier units** (`calc_bonuses()` / `calc_blows()` in "
    "`src/player-calcs.c`): `BLOWS` = whole extra blows; `SHOTS` = tenths of "
    "a shot per turn (`SHOTS +10` = one extra shot); `MIGHT` adds to the "
    "launcher multiplier; `SPEED` in speed points (+10 = twice as fast); "
    "`INFRA` in 10-foot steps; `LIGHT` = light radius; `TUNNEL`, `STEALTH`, "
    "`SEARCH` and the five stats add directly. `M8` in a value = a "
    "level-dependent bonus of 0..8 (see the `object.txt` header).")


# ---------------------------------------------------------------------------
# Monster formatting
# ---------------------------------------------------------------------------

IMM_FLAGS = ["IM_ACID", "IM_ELEC", "IM_FIRE", "IM_COLD", "IM_POIS", "IM_NETHER",
             "IM_WATER", "IM_PLASMA", "IM_NEXUS", "IM_DISEN"]
VULN_FLAGS = ["HURT_LIGHT", "HURT_ROCK", "HURT_FIRE", "HURT_COLD"]
PROT_FLAGS = ["NO_FEAR", "NO_STUN", "NO_CONF", "NO_SLEEP", "NO_HOLD", "NO_SLOW"]

BEHAV_LABELS = OrderedDict([
    ("INVISIBLE", "INVISIBLE"), ("COLD_BLOOD", "cold-blooded"),
    ("EMPTY_MIND", "empty-mind (no ESP)"), ("WEIRD_MIND", "weird-mind (ESP 1 in 10)"),
    ("MULTIPLY", "BREEDS"), ("REGENERATE", "regenerates"),
    ("PASS_WALL", "pass-wall"), ("KILL_WALL", "kill-wall"), ("SMASH_WALL", "smash-wall"),
    ("NEVER_MOVE", "never-moves"), ("NEVER_BLOW", "no melee"),
    ("RAND_25", "erratic 25%"), ("RAND_50", "erratic 50%"),
    ("GROUP_AI", "group-AI"), ("SMART", "smart"), ("STUPID", "stupid"),
    ("SPIRIT", "spirit"), ("POWERFUL", "powerful"), ("UNAWARE", "unaware (mimic)"),
    ("MIMIC_INV", "mimic-invisible"), ("FRIGHTENED", "frightened"),
    ("TAKE_ITEM", "takes items"), ("KILL_ITEM", "destroys items"),
    ("KILL_BODY", "kill-body"), ("MOVE_BODY", "move-body"),
    ("OPEN_DOOR", "opens doors"), ("BASH_DOOR", "bashes doors"),
    ("EVIL", "evil"), ("UNDEAD", "undead"), ("DEMON", "demon"), ("DRAGON", "dragon"),
    ("ANIMAL", "animal"), ("ORC", "orc"), ("TROLL", "troll"), ("GIANT", "giant"),
    ("NONLIVING", "nonliving"), ("METAL", "metal"),
])


def fmt_speed(s):
    d = s - 110
    return "+%d" % d if d >= 0 else "%d" % d


def fmt_blow(b):
    meth, eff, dmg = b
    s = meth.lower()
    if eff and eff != "HURT":
        s += " " + eff.lower()
    if dmg:
        s += " " + dmg
    return s


def fmt_blows(m):
    return ", ".join(fmt_blow(b) for b in m["blows"]) or "-"


def mon_spell_items(m, mspells, projs, with_damage=True):
    """Return (innate list, other list) of 'NAME (dmg)' strings."""
    inn, oth = [], []
    for s in m["spells"]:
        sp = mspells.get(s)
        if sp is None:
            warn("monster %s has unknown spell %s" % (m["name"], s))
            oth.append(s)
            continue
        t = s
        if with_damage:
            d = mon_spell_max_damage(sp, m, projs)
            if d:
                t += " %d" % d
        (inn if spell_is(sp, "INNATE") else oth).append(t)
    return inn, oth


def fmt_spells(m, mspells, projs):
    if not m["spells"]:
        return "-"
    innate, freq = resolve_spell_freqs(m, mspells)
    inn, oth = mon_spell_items(m, mspells, projs)
    parts = []
    if inn:
        parts.append("innate 1/%d: %s" % (innate, ", ".join(inn)))
    if oth:
        parts.append("spells 1/%d: %s" % (freq, ", ".join(oth)))
    return "; ".join(parts)


def fmt_defences(m):
    f = m["flags"]
    out = [x.replace("IM_", "im-").lower() for x in IMM_FLAGS if x in f]
    out += [x.lower().replace("_", "-") for x in VULN_FLAGS if x in f]
    out += [x.lower().replace("_", "-") for x in PROT_FLAGS if x in f]
    return ", ".join(out) or "-"


def fmt_friends(m):
    out = []
    for kind, f in m["friends"]:
        p = f.split(":")
        chance, num, who = p[0], p[1] if len(p) > 1 else "?", p[2] if len(p) > 2 else "?"
        role = (" (%s)" % p[3].strip()) if len(p) > 3 and p[3].strip() else ""
        who = who.strip()
        if kind == "base":
            who = "any " + who
        out.append("%s%% %s %s%s" % (chance.strip(), num.strip(), who, role))
    return out


def fmt_behaviour(m, with_friends=True):
    f = m["flags"]
    out = [lab for fl, lab in BEHAV_LABELS.items() if fl in f]
    if with_friends and m["friends"]:
        out.append("friends: " + "; ".join(fmt_friends(m)))
    return ", ".join(out) or "-"


def fmt_drops(m):
    f = m["flags"]
    out = []
    for x in ("DROP_20", "DROP_40", "DROP_60", "DROP_1", "DROP_2", "DROP_3", "DROP_4"):
        if x in f:
            out.append(x)
    for x in ("DROP_GOOD", "DROP_GREAT", "ONLY_ITEM", "ONLY_GOLD"):
        if x in f:
            out.append(x)
    for d in m["drops"]:
        p = d.split(":")
        if len(p) >= 3:
            rng = ""
            if len(p) >= 5:
                rng = " x%s-%s" % (p[3], p[4])
            out.append("%s %s %s%%%s" % (p[0], p[1], p[2], rng))
    for d in m["drop_base"]:
        p = d.split(":")
        rng = (" x%s-%s" % (p[2], p[3])) if len(p) >= 4 else ""
        out.append("any %s %s%%%s" % (p[0], p[1] if len(p) > 1 else "?", rng))
    for x in m["mimic"]:
        out.append("mimics " + x.replace(":", " "))
    return ", ".join(out) or "none"


# ---------------------------------------------------------------------------
# A1 / A2 / A3 : monsters
# ---------------------------------------------------------------------------

MON_FILES = ["lib/gamedata/monster.txt", "lib/gamedata/monster_base.txt",
             "lib/gamedata/monster_spell.txt", "lib/gamedata/projection.txt",
             "src/list-mon-spells.h"]

SPELL_LEGEND = [
    "- **Spells**: `innate 1/N` is the innate-attack frequency (breaths, "
    "arrows, boulders, spit: chance 1 in N per monster turn, i.e. 100/N %); "
    "`spells 1/N` is the spell frequency (`parse_monster_innate_freq()` / "
    "`parse_monster_spell_freq()` in `src/mon-init.c`; a monster with spells "
    "but no frequency line gets 1/4). The number after a spell is its "
    "**maximum damage** for this monster, computed as monster recall does "
    "(`mon_spell_lore_damage()` in `src/mon-spell.c`): breaths are "
    "`min(HP / divisor, cap)` at *average* HP (`breath_dam()`; divisors and "
    "caps in [A3](A3-monster-spells.md#breath-table)); other attacks are the "
    "maximum roll of their dice at the monster's spell power (`nonhp_dam()`). "
    "Before resistance.",
]


def depth_bands():
    return [(0, 0)] + [(lo, lo + 4) for lo in range(1, 100, 5)] + [(101, 127)]


def band_label(lo, hi):
    if hi == 0:
        return "Town (level 0)"
    return "Levels %d-%d" % (lo, hi)


def gen_a1(mons, mspells, projs, bases):
    doc = Doc("A1-monsters-by-depth.md", "Appendix A1. Monsters by depth", MON_FILES, [
        "Every **non-unique** monster race, grouped in 5-level depth bands by "
        "native depth, with its combat statistics, melee blows, spells "
        "(with computed maximum damage), defences and tactically relevant "
        "flags. Uniques are in [A2](A2-uniques.md); spell definitions and breath "
        "rules are in [A3](A3-monster-spells.md). Narrative advice is in "
        "[Chapter 8](../08-monsters.md).",
    ])
    nonu = [m for m in mons if "UNIQUE" not in m["flags"] and m["name"] != "<player>"]
    nonu.sort(key=lambda m: (m["depth"], m["name"].lower()))
    doc.h(2, "Legend")
    doc.p(
        "- **DL**: native depth (`depth:`); monsters are generated mostly at or "
        "below it, occasionally out of depth.",
        "- **Sym**: display glyph (from the monster or its base in `monster_base.txt`).",
        "- **Spd**: speed relative to normal (+10 = twice as many turns as a "
        "speed-normal character).",
        "- **HP**: *average* hit points (`hit-points:`). Non-uniques roll "
        "`Rand_normal(avg, sd)` with `sd = (avg*10/8+5)/10 (+1 if avg > 1)` "
        "and a maximum of `avg + 4*sd` (`mon_hp()` in `src/mon-make.c`). "
        "`shape` marks shape-change forms (rarity 0, never generated alone) "
        "that keep the HP of the monster that changed.",
        "- **AC**: armour class. **Exp**: experience *per level of the monster*: "
        "killing it gives `exp * DL / player level` (`monster.txt` header).",
        "- **R**: rarity; generation weight is `100 / rarity` (0 = never generated normally).",
        "- **Blows**: `method effect dice` per blow; plain damage blows show no effect.",
    )
    doc.p(*SPELL_LEGEND)
    doc.p(
        "- **Defences** (effects in `src/project-mon.c`): `im-acid/elec/fire/cold/pois` "
        "divide that element's damage by 9; `im-nexus/disen/plasma` reduce it to "
        "`3/(1d6+6)`; `im-nether` likewise (undead take no nether damage at all and "
        "other evil monsters half); `im-water` negates water. A monster that "
        "breathes an element also resists it (`project_monster_breath()`). "
        "`hurt-fire`/`hurt-cold` double fire/cold damage, `hurt-light` doubles "
        "light damage and makes weak light hurt, `hurt-rock` means stone-to-mud "
        "damages it. `no-*`: immune to fear/stun/confusion/sleep/hold/slow.",
        "- **Flags**: inherited from the base and own flags (`flags-off` removed): "
        "detection (`INVISIBLE`; `cold-blooded` = not seen by infravision; "
        "`empty-mind` = never seen by telepathy; `weird-mind` = seen by telepathy "
        "only 1 time in 10), movement, AI, race; `friends:` gives "
        "`chance% number name` of the group it is generated with.",
        "",
    )
    for lo, hi in depth_bands():
        rows = [m for m in nonu if lo <= m["depth"] <= hi]
        if not rows:
            continue
        doc.h(2, band_label(lo, hi))
        trows = []
        for m in rows:
            hp = "shape" if m["hp"] is None else str(m["hp"])
            trows.append([
                str(m["depth"]), md(m["name"]), code(m["glyph"]), fmt_speed(m["speed"]),
                hp, str(m["ac"]), str(m["exp"]), str(m["rarity"]),
                cell(fmt_blows(m)), cell(fmt_spells(m, mspells, projs)),
                cell(fmt_defences(m)), cell(fmt_behaviour(m)),
            ])
        doc.table(["DL", "Monster", "Sym", "Spd", "HP", "AC", "Exp", "R", "Blows",
                   "Spells", "Defences", "Flags"],
                  ["---:", "---", ":-:", "---:", "---:", "---:", "---:", "---:",
                   "---", "---", "---", "---"], trows)

    doc.h(2, "Monster bases (families)")
    doc.p("Flags listed here are inherited by every monster of the base "
          "(`monster_base.txt`), unless removed with `flags-off:`.\n")
    by_base = defaultdict(list)
    for m in mons:
        by_base[m["base"]].append(m)
    trows = []
    for bname, b in bases.items():
        ms = by_base.get(bname, [])
        if ms:
            rng = "%d-%d" % (min(x["depth"] for x in ms), max(x["depth"] for x in ms))
        else:
            rng = "-"
        nu = sum(1 for x in ms if "UNIQUE" in x["flags"])
        trows.append([md(bname), code(b["glyph"]), md(b["desc"]), str(len(ms) - nu),
                      str(nu), rng, cell(", ".join(b["flags"]))])
    doc.table(["Base", "Sym", "Family", "Non-uniques", "Uniques", "Depths", "Inherited flags"],
              ["---", ":-:", "---", "---:", "---:", "---", "---"], trows)
    return doc.write()


def gen_a2(mons, mspells, projs, quest_arts=()):
    uniq = [m for m in mons if "UNIQUE" in m["flags"]]
    uniq.sort(key=lambda m: (m["depth"], m["name"].lower()))
    doc = Doc("A2-uniques.md", "Appendix A2. Uniques", MON_FILES, [
        "One card per unique monster (%d in total), in order of native depth: "
        "statistics, melee, every spell with its maximum damage, defences and "
        "flags, escorts, drops and the in-game description. Uniques always have "
        "exactly their listed HP (`src/mon-make.c`: \"Uniques get a fixed amount "
        "of HP\"), so their breath damage is exact at full health and drops as "
        "they are wounded. Tactics are in [Chapter 8](../08-monsters.md)." % len(uniq),
    ])
    doc.h(2, "Legend")
    doc.p(
        "- **Exp** is per monster level: the kill is worth `exp * depth / player level`.",
        "- **Sleep**: initial sleepiness (0 = always awake); **Hearing**/**Smell**: "
        "detection ranges; **R**: rarity (0 = only appears via shape change or quest).",
        "- **Spells** list each spell with the lore text at this unique's spell "
        "power, its maximum damage (breaths at full HP) and, for saving-throw "
        "spells, *save* (a successful saving throw negates it, "
        "`do_mon_spell()` in `src/mon-spell.c`).",
        "- **Drops** (`mon_create_drop_count()` in `src/mon-make.c`): `DROP_1` = 1 "
        "object, `DROP_2` = 1-3, `DROP_3` = 2-4, `DROP_4` = 2-6, and each of "
        "`DROP_20/40/60` adds one object with that % chance (they add up); objects "
        "are made at level `max((monster level + depth) / 2, monster level)`; "
        "`DROP_GOOD`/`DROP_GREAT` = good/great objects, `ONLY_ITEM`/`ONLY_GOLD` "
        "restrict to items or gold; `drop:` lines are specific items with % chance "
        "and count range; `any <tval> N%` is a `drop-base:` line.",
        "- **Biggest hit** in the index is the larger of the strongest single spell "
        "and the total of one full melee round (max dice).",
        "",
    )
    doc.h(2, "Quick index")
    trows = []
    for m in uniq:
        trows.append([str(m["depth"]), "[%s](#%s)" % (md(m["name"]), gh_anchor(m["name"])),
                      code(m["glyph"]), fmt_speed(m["speed"]),
                      str(m["hp"]) if m["hp"] is not None else "shape",
                      str(m["ac"]), md(biggest_hit(m, mspells, projs))])
    doc.table(["DL", "Unique", "Sym", "Spd", "HP", "AC", "Biggest hit"],
              ["---:", "---", ":-:", "---:", "---:", "---:", "---"], trows)
    doc.h(2, "Cards")
    for m in uniq:
        a = doc.h(3, m["name"], toc=False)
        assert a == gh_anchor(m["name"]), m["name"]
        f = m["flags"]
        hp = str(m["hp"]) if m["hp"] is not None else "as the form it changed from"
        doc.p("- **Depth** %d, **speed** %s, **HP** %s, **AC** %d, **exp** %d/level, "
              "**rarity** %d, **sleep** %s, **hearing** %s%s%s" % (
                  m["depth"], fmt_speed(m["speed"]), hp, m["ac"], m["exp"], m["rarity"],
                  m["sleep"] or "-", m["hearing"] or "-",
                  (", **smell** %s" % m["smell"]) if m["smell"] else "",
                  (", **light** %s" % m["light"]) if m["light"] else ""))
        doc.p("- **Melee**: %s" % md(fmt_blows(m)))
        if m["spells"]:
            innate, freq = resolve_spell_freqs(m, mspells)
            fr = []
            if innate and any(spell_is(mspells[s], "INNATE") for s in m["spells"] if s in mspells):
                fr.append("innate 1/%d" % innate)
            if freq and any(not spell_is(mspells[s], "INNATE") for s in m["spells"] if s in mspells):
                fr.append("spells 1/%d" % freq)
            doc.p("- **Spells** (%s, spell power %d):" % (", ".join(fr), m["power"]))
            for s in m["spells"]:
                sp = mspells.get(s)
                if not sp:
                    doc.p("  - `%s`" % s)
                    continue
                lv = spell_level(sp, m["power"])
                d = mon_spell_max_damage(sp, m, projs)
                txt = lv["lore"].strip()
                if d:
                    txt += ", max %d damage" % d
                if lv["save"]:
                    txt += " (save)"
                doc.p("  - `%s`: %s" % (s, md(txt)))
        else:
            doc.p("- **Spells**: none")
        doc.p("- **Defences**: %s" % md(fmt_defences(m)))
        doc.p("- **Flags**: %s" % md(fmt_behaviour(m, with_friends=False)))
        other = [x for x in f if x not in BEHAV_LABELS and x not in IMM_FLAGS
                 and x not in VULN_FLAGS and x not in PROT_FLAGS
                 and not x.startswith("DROP_") and x not in ("ONLY_ITEM", "ONLY_GOLD")]
        if other:
            doc.p("- **Other flags**: %s" % md(", ".join(other)))
        if m["friends"]:
            doc.p("- **Escorts / friends**: %s" % md("; ".join(fmt_friends(m))))
        if m["shapes"]:
            doc.p("- **Can change into**: %s" % md(", ".join(m["shapes"])))
        drops = fmt_drops(m)
        if "QUESTOR" in f and m["depth"] == 100 and quest_arts:
            drops += "; always the quest artifacts %s (`QUEST_ART` kinds, " \
                     "`mon_create_drop()` in `src/mon-make.c`)" % ", ".join(quest_arts)
        doc.p("- **Drops**: %s" % md(drops))
        if "QUESTOR" in f:
            doc.p("- **Quest monster**: must be killed to win "
                  "(`lib/gamedata/quest.txt`).")
        if m["desc"]:
            doc.p("", "> %s" % md(m["desc"]))
        doc.p("")
    return doc.write()


def biggest_hit(m, mspells, projs):
    best, which = 0, ""
    for s in m["spells"]:
        sp = mspells.get(s)
        if sp:
            d = mon_spell_max_damage(sp, m, projs)
            if d and d > best:
                best, which = d, s
    mel = 0
    for b in m["blows"]:
        if b[2]:
            mel += dice_max(b[2], {}, m["depth"]) or 0
    if mel > best:
        return "melee round %d" % mel
    if best == 0:
        return "-"
    return "%s %d" % (which, best)


# Side effects of each element on the player, transcribed from the
# project_player_handler_*() functions of src/project-player.c.
SIDE_EFFECTS = OrderedDict([
    ("ACID", "Damages inventory (unless immune). Hitting armour halves the damage "
             "(`adjust_dam()` / `minus_ac()`)."),
    ("ELEC", "Damages inventory (unless immune)."),
    ("FIRE", "Damages inventory (unless immune). Caster power >= 80: chance "
             "(`randint0(dam) > 500`) each of STR drain, blindness, poison."),
    ("COLD", "Damages inventory (unless immune). Caster power >= 80: chance "
             "(`randint0(dam) > 500`) each of DEX drain and experience drain "
             "(blocked by hold life)."),
    ("POIS", "Poisons for 10+1d(dam) turns unless resisted. Power >= 60: chance "
             "(`randint0(dam) > 200`) of extra acid damage and of CON drain."),
    ("LIGHT", "Unresisted: blind 2+1d5; if dam > 300 also confused."),
    ("DARK", "Unresisted: blind 2+1d5. Power >= 70: chances of experience drain, "
             "slowness and amnesia."),
    ("SOUND", "Unresisted: stun 5+1d(dam/3) (max 35) unless protected from stun; "
              "if dam > 300 also confused."),
    ("SHARD", "Unresisted: cuts of 1d(dam)."),
    ("NEXUS", "Unresisted: stat scramble (saving throw) and one of teleport-to "
              "(1/3), teleport level (1/4 of the rest, saving throw) or teleport 200."),
    ("NETHER", "Unless resisted or hold life: drains 200 + exp/100*life-drain% "
               "experience. Power >= 80: chances of mana loss and losing a turn."),
    ("CHAOS", "Unresisted: hallucination, confusion 10+1d20, and experience drain "
              "(blocked by hold life)."),
    ("DISEN", "Unresisted: disenchants an equipped item."),
    ("WATER", "Confusion 5+1d5 and stun 1d40 (no resistance reduces water)."),
    ("ICE", "Cold inventory damage (unless immune to cold), cuts 5d8 unless shard "
            "resistant, stun 1d15. Damage is reduced by cold resistance."),
    ("GRAVITY", "Blink unless 1d127 <= player level, slow 4+1d4, stun (unless "
                "protected). Cannot be resisted."),
    ("INERTIA", "Slow 4+1d4. Cannot be resisted."),
    ("FORCE", "Stun 1d20 and knock-back. Cannot be resisted."),
    ("TIME", "Experience drain (1/2), else drain two stats (4/5) or all stats. "
             "Cannot be resisted."),
    ("PLASMA", "Stun 5+1d(3*dam/4) (max 35) unless protected from stun. Cannot "
               "be resisted."),
    ("METEOR", "No side effect."), ("MISSILE", "No side effect."),
    ("MANA", "No side effect. Cannot be resisted."),
    ("HOLY_ORB", "No side effect."), ("ARROW", "No side effect."),
])


def gen_a3(mons, mspells, projs, timed):
    doc = Doc("A3-monster-spells.md", "Appendix A3. Monster spells, breaths and threats",
              MON_FILES + ["lib/gamedata/player_timed.txt", "src/project-player.c"], [
        "Every monster spell with its effects, damage formula, lore text and "
        "saving throw; the breath rules per element and the deadliest breather "
        "of each element; and the shallowest native depth at which each major "
        "threat appears. Per-monster numbers are in [A1](A1-monsters-by-depth.md) "
        "and [A2](A2-uniques.md); how to resist is in "
        "[Chapter 5](../05-elements-and-status.md).",
    ])
    doc.h(2, "How monster spell damage works")
    doc.p(
        "- **Spell power** is the monster's native depth unless `spell-power:` "
        "overrides it (`parse_monster_depth()` / `parse_monster_spell_power()` in "
        "`src/mon-init.c`). Dice expressions use `power`; `sight` is "
        "`player:max-sight` = 20 (`lib/gamedata/constants.txt`).",
        "- **Breaths** ignore spell power: damage = `min(current HP / divisor, "
        "cap)` (`breath_dam()` in `src/mon-spell.c`). A wounded breather is weaker.",
        "- **Lore** text changes at `power-cutoff` values: the text of the last "
        "cutoff that is `<=` the monster's power is used "
        "(`mon_spell_lore_description()`).",
        "- **Save**: spells with a `message-save` line allow a saving throw: if "
        "`randint0(100) < skill_save` the spell does nothing (`do_mon_spell()`).",
        "- **Hit**: base chance for the spell to hit (100 = always; archery 50-60).",
        "- **Frequencies**: innate spells (breaths, arrows, boulders, spit, whip, "
        "shriek, weave) use `innate-freq`, all others `spell-freq`.",
        "",
    )
    doc.h(2, "Breath table")
    doc.p("From `projection.txt`. *Divisor* divides the breather's current HP; "
          "*Cap* is the maximum before resistance; *Resist* is the fraction "
          "let through per level of resistance (`numerator / denominator`, "
          "applied once per resistance level by `adjust_dam()` in "
          "`src/project-player.c`; immunity = 0, vulnerability = x4/3). Base "
          "elements: one permanent resist = 1/3, plus a temporary resist = 1/9. "
          "High elements use a random denominator, so one resist leaves "
          "6/12 to 6/9 of the damage. *HP for cap* is the HP at which the cap is "
          "reached. Only elements with a monster breath spell are listed.\n")
    breath_elems = []
    for s, sp in mspells.items():
        if spell_is(sp, "BREATH"):
            breath_elems.append((sp["effects"][0].sub, s))
    trows = []
    for el, s in breath_elems:
        p = projs[el]
        if not p["num"]:
            frac = "cannot be resisted"
        elif p["den"].isdigit():
            frac = "%s/%s" % (p["num"], p["den"])
        else:
            frac = "%s/(%s)" % (p["num"], p["den"])
        trows.append([md(p["name"]), code(s), str(p["divisor"]), str(p["cap"]),
                      md(frac), str(p["divisor"] * p["cap"]), md(SIDE_EFFECTS.get(el, ""))])
    doc.table(["Element", "Spell", "Divisor", "Cap", "Resist", "HP for cap", "Side effects (unresisted)"],
              ["---", "---", "---:", "---:", "---", "---:", "---"], trows)
    doc.p("Other projections (no breath): " + "; ".join(
        "**%s**: %s" % (md(projs[c]["name"]), md(t)) for c, t in SIDE_EFFECTS.items()
        if c not in [e for e, _ in breath_elems] and c in projs) + "\n")

    doc.h(2, "Every monster spell")
    doc.p("*Type* is from `src/list-mon-spells.h`. *Damage* gives the formula "
          "(`power` = spell power); for breaths the HP rule. *Lore by power* lists "
          "each lore text with the spell-power range it applies to. *Protection* "
          "lists what blocks the timed effects (from `fail:` lines in "
          "`player_timed.txt`).\n")
    trows = []
    for name, sp in mspells.items():
        types = ", ".join(t.replace("RST_", "").lower() for t in sp["types"])
        effs = effects_text(sp["effects"]) or "-"
        if spell_is(sp, "BREATH"):
            p = projs[sp["effects"][0].sub]
            dmg = "min(HP/%d, %d)" % (p["divisor"], p["cap"])
        elif spell_has_damage(sp):
            fs = []
            for e in sp["effects"]:
                if e.name == "LASH":
                    fs.append("first blow + half of each other blow (max dice)")
                elif e.dice and e.name != "TIMED_INC":
                    fs.append(dice_text(e.dice, e.exprs))
            dmg = " + ".join(fs) or "-"
        else:
            dmg = "-"
        lores = []
        lv = sp["levels"]
        for i, l in enumerate(lv):
            lo = l["power"]
            hi = lv[i + 1]["power"] - 1 if i + 1 < len(lv) else None
            txt = l["lore"].strip()
            if len(lv) == 1:
                lores.append(txt)
            else:
                rng = ("power %d+" % lo) if hi is None else ("power %d-%d" % (lo, hi))
                lores.append("%s: %s" % (rng, txt))
        prot = []
        for e in sp["effects"]:
            if e.name == "TIMED_INC" and e.sub in timed:
                prot += [timed_fail_text(fl) for fl in timed[e.sub]["fail"]]
        save = "yes" if any(l["save"] for l in lv) else "-"
        trows.append([code(name), md(types), str(sp["hit"]), save, md(effs), md(dmg),
                      md("; ".join(lores)), cell(", ".join(dict.fromkeys(prot)))])
    doc.table(["Spell", "Type", "Hit", "Save", "Effects", "Damage", "Lore by power", "Protection"],
              ["---", "---", "---:", ":-:", "---", "---", "---", "---"], trows)

    # Deadliest breathers
    doc.h(2, "Deadliest breathers by element")
    doc.p("For each breath, the monsters with the largest breath at average HP "
          "(top 10, uniques marked U), the shallowest breather, and the number of "
          "breathers (shape-change forms without their own HP are not counted). *Max-HP breath* uses the largest possible HP roll "
          "(`avg + 4*sd`, `mon_hp(MAXIMISE)`) for non-uniques; uniques have fixed HP.\n")
    by_spell = defaultdict(list)
    for m in mons:
        if m["hp"] is None:
            continue
        for s in m["spells"]:
            sp = mspells.get(s)
            if sp and spell_is(sp, "BREATH"):
                d = mon_spell_max_damage(sp, m, projs)
                uniq = "UNIQUE" in m["flags"]
                dmax = d if uniq else mon_spell_max_damage(sp, m, projs, hp=mon_hp_max(m["hp"]))
                by_spell[s].append((d, dmax, m))
    for el, s in breath_elems:
        lst = by_spell.get(s, [])
        p = projs[el]
        doc.h(3, "%s (%s)" % (p["name"].capitalize(), s))
        if not lst:
            doc.p("No monster has this breath.\n")
            continue
        shallow = min(lst, key=lambda x: (x[2]["depth"], x[2]["name"]))
        doc.p("%d breathers. Shallowest: **%s** (level %d, %d damage). Cap %d reached "
              "at %d HP.\n" % (len(lst), md(shallow[2]["name"]), shallow[2]["depth"],
                               shallow[0], p["cap"], p["divisor"] * p["cap"]))
        top = sorted(lst, key=lambda x: (-x[0], x[2]["depth"], x[2]["name"]))[:10]
        trows = [[md(m["name"]) + (" (U)" if "UNIQUE" in m["flags"] else ""),
                  str(m["depth"]), fmt_speed(m["speed"]), str(m["hp"]), str(d), str(dm)]
                 for d, dm, m in top]
        doc.table(["Monster", "DL", "Spd", "Avg HP", "Breath", "Max-HP breath"],
                  ["---", "---:", "---:", "---:", "---:", "---:"], trows)

    # First appearance of threats
    doc.h(2, "Shallowest appearance of major threats")
    doc.p("The shallowest native depth of any monster (non-unique and unique "
          "separately) with each ability. Monsters can appear out of depth, so "
          "treat these as the point where the threat becomes *routine*, not the "
          "first possible encounter.\n")
    checks = OrderedDict()

    def has_spell(s):
        return lambda m: s in m["spells"]

    def has_blow(e):
        return lambda m: any(b[1] == e for b in m["blows"])

    def has_flag(f):
        return lambda m: f in m["flags"]

    for s in mspells:
        if spell_is(mspells[s], "BREATH"):
            checks["breathes %s (%s)" % (projs[mspells[s]["effects"][0].sub]["name"], s)] = has_spell(s)
    for s in ["HOLD", "BLIND", "CONF", "SCARE", "SLOW", "FORGET", "DRAIN_MANA",
              "MIND_BLAST", "BRAIN_SMASH", "WOUND", "TELE_TO", "TELE_SELF_TO",
              "TELE_AWAY", "TELE_LEVEL", "DARKNESS", "TRAPS", "HEAL", "HASTE",
              "BLINK", "TPORT", "SHAPECHANGE", "BO_MANA", "BA_MANA", "BA_NETH",
              "BA_DARK", "BA_WATE", "BE_NETH", "STORM", "BO_PLAS", "BO_ICE"]:
        if s in mspells:
            checks["casts %s" % s] = has_spell(s)
    for s in mspells:
        if s.startswith("S_"):
            checks["summons (%s)" % s] = has_spell(s)
    blow_effects = sorted({b[1] for m in mons for b in m["blows"] if b[1] and b[1] != "HURT"})
    for be in blow_effects:
        checks["melee %s" % be] = has_blow(be)
    for fl in ["INVISIBLE", "PASS_WALL", "KILL_WALL", "MULTIPLY", "EMPTY_MIND",
               "WEIRD_MIND", "COLD_BLOOD", "TAKE_ITEM", "KILL_ITEM", "IM_NETHER",
               "IM_DISEN", "IM_NEXUS"]:
        checks["flag %s" % fl] = has_flag(fl)
    checks["speed +10 or more"] = lambda m: m["speed"] >= 120
    checks["speed +20 or more"] = lambda m: m["speed"] >= 130
    checks["speed +30 or more"] = lambda m: m["speed"] >= 140
    trows = []
    for label, pred in checks.items():
        nu = [m for m in mons if "UNIQUE" not in m["flags"] and m["name"] != "<player>"
              and m["rarity"] > 0 and pred(m)]
        un = [m for m in mons if "UNIQUE" in m["flags"] and pred(m)]
        a = min(nu, key=lambda m: (m["depth"], m["name"])) if nu else None
        b = min(un, key=lambda m: (m["depth"], m["name"])) if un else None
        trows.append([md(label),
                      ("%d" % a["depth"]) if a else "-", md(a["name"]) if a else "-",
                      ("%d" % b["depth"]) if b else "-", md(b["name"]) if b else "-",
                      str(len(nu) + len(un))])
    doc.table(["Threat", "DL", "Shallowest non-unique", "DL", "Shallowest unique", "# monsters"],
              ["---", "---:", "---", "---:", "---", "---:"], trows)
    return doc.write()


def timed_fail_text(fl):
    kind = fl[0].strip() if fl else ""
    what = fl[1].strip() if len(fl) > 1 else "?"
    return {"1": "%s" % what, "2": "resist %s" % pname(what),
            "3": "vulnerable %s" % pname(what), "4": "player flag %s" % what,
            "5": "timed %s" % what}.get(kind, what)


# ---------------------------------------------------------------------------
# A4 artifacts, A5 egos, A6 object kinds
# ---------------------------------------------------------------------------

TVAL_ORDER = ["sword", "polearm", "hafted", "digger", "bow", "shot", "arrow", "bolt",
              "soft armor", "hard armor", "dragon armor", "shield", "cloak", "gloves",
              "boots", "helm", "crown", "light", "ring", "amulet", "potion", "scroll",
              "flask", "food", "mushroom", "wand", "staff", "rod", "magic book",
              "prayer book", "nature book", "shadow book", "chest", "gold"]


def tval_title(t):
    return t.title()


def act_text(name, acts):
    a = acts.get(name)
    if not a:
        warn("unresolved activation %s" % name)
        return name
    eff = effects_text(a["effects"])
    s = a["desc"] or eff
    if a["desc"] and eff and eff.lower() != a["desc"].lower():
        s = "%s [%s]" % (a["desc"], eff)
    return "%s: %s (level %s%s)" % (name, s, a["level"], ", aimed" if a["aim"] == "1" else "")


def gen_a4(acts, slays, brands, kinds):
    recs = parse_records("artifact.txt")
    doc = Doc("A4-artifacts.md", "Appendix A4. Standard artifacts",
              ["lib/gamedata/artifact.txt", "lib/gamedata/activation.txt",
               "lib/gamedata/object.txt", "lib/gamedata/slay.txt", "lib/gamedata/brand.txt"], [
        "All %d standard artifacts (not used with the `birth_randarts` option), "
        "grouped by item type and sorted by level: base item, level and "
        "allocation, combat values, modifiers, slays/brands, resistances and "
        "flags, curses, and activation resolved through `activation.txt`. "
        "How to judge and use them: [Chapter 6](../06-equipment.md)." % len(recs),
    ])
    kind_by = {(k["tval"], k["name"].lower()): k for k in kinds}
    doc.h(2, "Legend")
    doc.p(
        "- **Level** is the artifact level (also the activation difficulty). "
        "**Alloc** is `min-max depth (xcommonness)`. When an object of the "
        "artifact's exact base kind is allowed to become an artifact "
        "(`make_artifact()`; special one-of-a-kind items such as the Phial use "
        "`make_artifact_special()`, both in `src/obj-make.c`), the artifact is "
        "picked if `randint1(100) <= commonness`; shallower than its minimum depth "
        "it also needs a 1 in `2 * (min - depth)` roll; deeper than its maximum "
        "depth it is never generated. Each artifact exists once per game.",
        "- **Combat**: weapons `dice (+hit, +dam)`, armour `[base AC, +AC]`, launchers "
        "`xmultiplier` (base item).",
        "- **Mods**: stat and other modifiers; `resist`/`immune`/`vulnerable` are "
        "elemental resistance levels from `values:` (`RES_*`).",
        "- **Activation**: activation name, description from `activation.txt` (or "
        "the effect list if no description), activation level; **recharge** in "
        "player turns (`time:`).",
        MODS_NOTE,
        "",
    )
    groups = defaultdict(list)
    for r in recs:
        tv = tv_norm(first(r, "base-object", "?:?").split(":")[0])
        groups[tv].append(r)
    order = [t for t in TVAL_ORDER if t in groups] + sorted(t for t in groups if t not in TVAL_ORDER)
    doc.h(2, "Index")
    trows = []
    for t in order:
        for r in sorted(groups[t], key=lambda r: (int(first(r, "level", "0")), first(r, "name"))):
            nm = art_name(r)
            trows.append(["[%s](#%s)" % (md(nm), gh_anchor(nm)), md(tval_title(t)),
                          first(r, "level", "").strip(), md(fmt_alloc(first(r, "alloc", "").strip()))])
    doc.table(["Artifact", "Type", "Level", "Alloc"], ["---", "---", "---:", "---"], trows)
    for t in order:
        doc.h(2, "%s (%d)" % (tval_title(t), len(groups[t])))
        for r in sorted(groups[t], key=lambda r: (int(first(r, "level", "0")), first(r, "name"))):
            nm = art_name(r)
            a = doc.h(3, nm, toc=False)
            if a != gh_anchor(nm):
                warn("artifact anchor collision %s" % nm)
            btv, _, bname = first(r, "base-object", "?:?").partition(":")
            k = kind_by.get((tv_norm(btv), clean_name(bname).lower()))
            special = first(r, "graphics") is not None
            if k is None and not special:
                warn("artifact %s: base object %s not found" % (nm, bname))
            att = (first(r, "attack") or "").strip()
            arm = (first(r, "armor") or "").strip()
            combat = []
            if att:
                p = att.split(":")
                if t == "bow" and k:
                    mult = [v for v in k["values"] if v.startswith("MIGHT")]
                    combat.append("x%s (+%s, +%s)" % (k["pval"] or "?", p[1], p[2]))
                elif t in ("sword", "polearm", "hafted", "digger", "shot", "arrow", "bolt") or p[0] not in ("1d1", "0d0"):
                    combat.append("%s (%+d, %+d)" % (p[0], int(p[1]), int(p[2])))
                elif int(p[1]) or int(p[2]):
                    combat.append("(%+d, %+d)" % (int(p[1]), int(p[2])))
            if arm:
                p = arm.split(":")
                if int(p[0]) or int(p[1]):
                    combat.append("[%s, %+d]" % (p[0], int(p[1])))
            doc.p("- **Base**: %s; **level** %s; **alloc** %s; **weight** %s; **cost** %s" % (
                md(clean_name(bname)) + (" (unique item kind made for this artifact)"
                                         if special else ""),
                first(r, "level", "?").strip(),
                md(fmt_alloc((first(r, "alloc") or "").strip())),
                fmt_weight(int(first(r, "weight", "0"))), first(r, "cost", "?").strip()))
            if combat:
                doc.p("- **Combat**: %s" % md(" ".join(combat)))
            vals = split_flags(every(r, "values"))
            mods = [v for v in fmt_values(vals) if not re.match(r"(resist|immune|vulnerable|res level)", v)]
            res = [v for v in fmt_values(vals) if re.match(r"(resist|immune|vulnerable|res level)", v)]
            if mods:
                doc.p("- **Mods**: %s" % md(", ".join(mods)))
            sb = slay_text([x.strip() for x in every(r, "slay")], slays) + \
                brand_text([x.strip() for x in every(r, "brand")], brands)
            if sb:
                doc.p("- **Slays/brands**: %s" % md(", ".join(sb)))
            if res:
                doc.p("- **Resists**: %s" % md(", ".join(res)))
            fl = split_flags(every(r, "flags"))
            if fl:
                doc.p("- **Flags**: %s" % md(", ".join(fl)))
            cur = every(r, "curse")
            if cur:
                doc.p("- **Curses**: %s" % md(", ".join(
                    "%s (power %s)" % tuple((c.split(":") + ["?"])[:2]) for c in cur)))
            act = (first(r, "act") or "").strip()
            if act:
                doc.p("- **Activation**: %s; recharge %s turns" % (
                    md(act_text(act, acts)), md((first(r, "time") or "?").strip())))
            desc = joined_text(every(r, "desc"))
            if desc:
                doc.p("", "> %s" % md(desc))
            doc.p("")
    return doc.write()


def art_name(r):
    nm = first(r, "name").strip()
    base = clean_name(first(r, "base-object", "?:?").partition(":")[2])
    if nm.startswith("'"):
        return "%s %s" % (base, nm)
    return "The %s %s" % (base, nm)


EGO_GROUPS = OrderedDict([
    ("Melee weapons", ["sword", "polearm", "hafted"]),
    ("Diggers", ["digger"]),
    ("Launchers", ["bow"]),
    ("Ammunition", ["shot", "arrow", "bolt"]),
    ("Body armour", ["soft armor", "hard armor", "dragon armor"]),
    ("Shields", ["shield"]),
    ("Cloaks", ["cloak"]),
    ("Gloves", ["gloves"]),
    ("Boots", ["boots"]),
    ("Helms and crowns", ["helm", "crown"]),
    ("Light sources", ["light"]),
])


def gen_a5(slays, brands, props):
    recs = parse_records("ego_item.txt")
    doc = Doc("A5-ego-items.md", "Appendix A5. Ego items",
              ["lib/gamedata/ego_item.txt", "lib/gamedata/object_property.txt",
               "lib/gamedata/slay.txt", "lib/gamedata/brand.txt", "src/obj-make.c"], [
        "All %d ego-item types, grouped by the kind of item they apply to: "
        "applicable item types, allocation, value, combat bonuses, modifiers, "
        "flags, slays/brands and random extra powers. Use with "
        "[Chapter 6](../06-equipment.md)." % len(recs),
    ])
    sust = [p["code"] for p in props if p["type"] == "flag" and p["subtype"] == "sustain"]
    pw = [p["code"] for p in props if p["type"] == "flag" and p["subtype"] in ("protection", "misc ability")]
    doc.h(2, "Legend")
    doc.p(
        "- **Alloc**: `min-max depth (xcommonness)`. An ego can still be rolled "
        "out of depth: below its minimum depth with chance 1 in "
        "`max(2, (min - level) / 3)` (`ego_find_random()` in `src/obj-make.c`).",
        "- **Cost/rating**: `info:` value added to the item and level-feeling rating.",
        "- **Combat**: extra `+hit : +dam : +AC`, each a random value; `d6` = 1d6, "
        "`M8` = level-scaled bonus 0..8 (see the `object.txt` header). **Min** gives "
        "`min-combat`/`min-values` floors (255 = no minimum).",
        MODS_NOTE,
        "- **Random power** (`ego_apply_magic()` in `src/obj-make.c`): "
        "`RAND_SUSTAIN` one random sustain (%s); `RAND_POWER` one random "
        "protection or ability (%s); `RAND_BASE_RES` one random base resist "
        "(acid, lightning, fire, cold); `RAND_HI_RES` one random high resist "
        "(poison, light, dark, sound, shards, nexus, nether, chaos, disenchantment); "
        "`RAND_RES_POWER` one of: random power (1/3) or random base resist (2/3). "
        "The pick avoids properties the item already has." % (
            ", ".join(sust), ", ".join(pw)),
        "",
    )
    rows_by = OrderedDict((g, []) for g in EGO_GROUPS)
    rows_by["Other"] = []
    for r in recs:
        types = [tv_norm(x) for x in every(r, "type")]
        items = every(r, "item")
        item_tvs = [tv_norm(x.split(":")[0]) for x in items]
        alltv = types + [x for x in item_tvs if x not in types]
        grp = "Other"
        for g, tvs in EGO_GROUPS.items():
            if alltv and alltv[0] in tvs:
                grp = g
                break
        applies = []
        if types:
            applies.append("any " + ", ".join(types))
        if items:
            by_tv = OrderedDict()
            for x in items:
                tv, _, nm = x.partition(":")
                by_tv.setdefault(tv_norm(tv), []).append(clean_name(nm))
            for tv, nms in by_tv.items():
                applies.append("%s: %s" % (tv, ", ".join(nms)))
        if not applies:
            applies.append("nothing: no `type:`/`item:` lines, so it is never generated")
        info = (first(r, "info") or ":").split(":")
        flags = split_flags(every(r, "flags"))
        rnd = [f for f in flags if f.startswith("RAND_")]
        flags = [f for f in flags if not f.startswith("RAND_")]
        off = split_flags(every(r, "flags-off"))
        if off:
            flags.append("removes " + ", ".join(off))
        vals = fmt_values(split_flags(every(r, "values")))
        mins = []
        if first(r, "min-combat"):
            mins.append("combat " + first(r, "min-combat").strip())
        mv = split_flags(every(r, "min-values"))
        if mv:
            mins.append(" ".join(mv))
        sb = slay_text([x.strip() for x in every(r, "slay")], slays) + \
            brand_text([x.strip() for x in every(r, "brand")], brands) + \
            ["curse " + x.strip() for x in every(r, "curse")]
        act = (first(r, "act") or "").strip()
        rows_by[grp].append([
            md(first(r, "name").strip()), cell("; ".join(applies)),
            md(fmt_alloc((first(r, "alloc") or "").strip())),
            "%s / %s" % (info[0].strip(), info[1].strip() if len(info) > 1 else "?"),
            cell((first(r, "combat") or "").strip()), cell(", ".join(vals)),
            cell(", ".join(flags)), cell(", ".join(sb)), cell(", ".join(rnd)),
            cell("; ".join(mins)) + ((" act " + md(act)) if act else ""),
        ])
    for g, rows in rows_by.items():
        if not rows:
            continue
        doc.h(2, "%s (%d)" % (g, len(rows)))
        doc.table(["Ego", "Applies to", "Alloc", "Cost / rating", "Combat", "Mods",
                   "Flags", "Slays/brands", "Random power", "Min"],
                  ["---", "---", "---", "---", "---", "---", "---", "---", "---", "---"], rows)
    return doc.write()


WEAPON_TV = ["sword", "polearm", "hafted", "digger", "shot", "arrow", "bolt"]
ARMOR_TV = ["soft armor", "hard armor", "dragon armor", "shield", "cloak", "gloves",
            "boots", "helm", "crown"]
DEVICE_TV = ["wand", "staff", "rod"]
CONSUME_TV = ["potion", "scroll", "food", "mushroom", "flask"]


def obj_notes(o, slays, brands, curses=True):
    notes = list(o["flags"]) + fmt_values(o["values"])
    notes += slay_text(o["slays"], slays) + brand_text(o["brands"], brands)
    if curses:
        notes += ["curse " + c.replace(":", " power ") for c in o["curses"]]
    return notes


def gen_a6(kinds, bases, slays, brands, books):
    doc = Doc("A6-object-kinds.md", "Appendix A6. Object kinds",
              ["lib/gamedata/object.txt", "lib/gamedata/object_base.txt",
               "lib/gamedata/class.txt", "lib/gamedata/projection.txt",
               "lib/gamedata/player_timed.txt", "src/list-effects.h"], [
        "Every base object kind (%d in `object.txt`, not counting the %d "
        "`type:none` display placeholders, plus the %d spell books "
        "defined in `class.txt`), grouped by item type: allocation, cost and base "
        "statistics; for consumables and devices the effect in words, charges "
        "and recharge time. See [Chapter 7](../07-consumables-and-devices.md) and "
        "[Chapter 6](../06-equipment.md)." % (
            sum(1 for k in kinds if k["tval"] != "none"),
            sum(1 for k in kinds if k["tval"] == "none"), len(books)),
    ])
    doc.h(2, "Legend")
    doc.p(
        "- **Lvl** is the object level (device skill difficulty and activation "
        "level). **Alloc** is `min-max depth (xcommonness)`; `not random` items "
        "are only created by stores, birth kits or special code.",
        "- **Cost** is the base value in gold (store sell price before the black "
        "market multiplier, see [A9](A9-stores.md)).",
        "- Weapons: **Dice** base damage; ammunition is multiplied by the launcher. "
        "Armour: **AC** base armour class, **Hit** to-hit modifier. Launchers: "
        "**Mult** multiplier.",
        "- **Effect**: what using the item does, generated from the `effect:` "
        "lines. Dice notation: `20+1d20`; `plev` = player level; `m35` in a "
        "healing roll = at least 35% of HP lost; `M60` in a teleport = 60% of "
        "the level's larger dimension; `x food-value` = times "
        "`player:food-value` (100).",
        "- **Charges** for wands/staffs (random value); **Recharge** for rods "
        "and activations, in player turns.",
        "- **Pile**: `chance% : number` for being generated in a stack.",
        MODS_NOTE,
        "",
    )
    by = defaultdict(list)
    for o in kinds:
        if o["tval"] == "none":
            continue  # <pile>, <unknown item> etc.: display placeholders, not objects
        by[o["tval"]].append(o)
    order = [t for t in TVAL_ORDER if t in by] + sorted(t for t in by if t not in TVAL_ORDER)
    for t in order:
        rows = sorted(by[t], key=lambda o: (o["level"], o["name"].lower()))
        b = bases.get(t, {})
        doc.h(2, "%s (%d)" % (tval_title(t), len(rows)))
        bf = b.get("flags", [])
        extra = []
        if bf:
            extra.append("base flags: %s" % ", ".join(bf))
        if b.get("break"):
            extra.append("break chance %s%%" % b["break"])
        if extra:
            doc.p("*%s* (`object_base.txt`).\n" % md("; ".join(extra)))
        if t == "gold":
            doc.p("Gold kinds only name the drop: its value is "
                  "`rand_spread(16 * level / 10 + 16, level + 10)`, multiplied by 10 "
                  "with chance 1/100 (repeatable), x5 with `birth_no_selling` in the "
                  "dungeon, capped near 32767; the kind is picked from the value by "
                  "`money_kind()` (`make_gold()` in `src/obj-make.c`).\n")
        if t == "chest":
            doc.p("Chest traps are listed in "
                  "[A10](A10-curses-traps-timed-effects.md#chest-traps).\n")
        trows = []
        if t in WEAPON_TV:
            for o in rows:
                atk = o["attack"].split(":") if o["attack"] else ["", "0", "0"]
                plus = ""
                if len(atk) > 2 and (atk[1].strip() not in ("0", "") or atk[2].strip() not in ("0", "")):
                    plus = " (%s, %s)" % (atk[1], atk[2])
                trows.append([md(o["name"]), str(o["level"]), md(fmt_alloc(o["alloc"])),
                              md(atk[0] + plus), fmt_weight(o["weight"]), str(o["cost"]),
                              cell(", ".join(obj_notes(o, slays, brands)))])
            doc.table(["Name", "Lvl", "Alloc", "Dice", "Weight", "Cost", "Notes"],
                      ["---", "---:", "---", "---", "---:", "---:", "---"], trows)
        elif t == "bow":
            for o in rows:
                trows.append([md(o["name"]), str(o["level"]), md(fmt_alloc(o["alloc"])),
                              "x" + (o["pval"] or "?"), fmt_weight(o["weight"]), str(o["cost"]),
                              cell(", ".join(obj_notes(o, slays, brands)))])
            doc.table(["Name", "Lvl", "Alloc", "Mult", "Weight", "Cost", "Notes"],
                      ["---", "---:", "---", "---:", "---:", "---:", "---"], trows)
        elif t in ARMOR_TV:
            for o in rows:
                arm = o["armor"].split(":") if o["armor"] else ["0", "0"]
                atk = o["attack"].split(":") if o["attack"] else ["", "0", "0"]
                ac = arm[0] + ((" +" + arm[1]) if len(arm) > 1 and arm[1].strip() not in ("0", "") else "")
                notes = obj_notes(o, slays, brands)
                if o["effects"]:
                    notes.append("activation: " + effects_text(o["effects"]))
                    if o["time"]:
                        notes.append("recharge " + o["time"])
                trows.append([md(o["name"]), str(o["level"]), md(fmt_alloc(o["alloc"])),
                              md(ac), atk[1] if len(atk) > 1 else "0",
                              fmt_weight(o["weight"]), str(o["cost"]), cell(", ".join(notes))])
            doc.table(["Name", "Lvl", "Alloc", "AC", "Hit", "Weight", "Cost", "Notes"],
                      ["---", "---:", "---", "---", "---:", "---:", "---:", "---"], trows)
        elif t in DEVICE_TV:
            for o in rows:
                ch = o["charges"] if t != "rod" else ""
                tm = o["time"]
                trows.append([md(o["name"]), str(o["level"]), md(fmt_alloc(o["alloc"])),
                              str(o["cost"]), cell(effects_text(o["effects"])),
                              cell(ch), cell(tm), cell(", ".join(obj_notes(o, slays, brands)))])
            doc.table(["Name", "Lvl", "Alloc", "Cost", "Effect", "Charges", "Recharge", "Notes"],
                      ["---", "---:", "---", "---:", "---", "---", "---", "---"], trows)
        elif t in CONSUME_TV:
            for o in rows:
                trows.append([md(o["name"]), str(o["level"]), md(fmt_alloc(o["alloc"])),
                              str(o["cost"]), cell(effects_text(o["effects"])), cell(o["pile"]),
                              cell(", ".join(obj_notes(o, slays, brands)))])
            doc.table(["Name", "Lvl", "Alloc", "Cost", "Effect", "Pile", "Notes"],
                      ["---", "---:", "---", "---:", "---", "---", "---"], trows)
        else:
            for o in rows:
                notes = obj_notes(o, slays, brands)
                eff = effects_text(o["effects"])
                if eff:
                    eff = "activation: " + eff + ((" (recharge %s)" % o["time"]) if o["time"] else "")
                if o["pval"] and t != "chest":
                    notes.append("pval " + o["pval"])
                trows.append([md(o["name"]), str(o["level"]), md(fmt_alloc(o["alloc"])),
                              str(o["cost"]), fmt_weight(o["weight"]),
                              cell(", ".join(notes)), cell(eff)])
            doc.table(["Name", "Lvl", "Alloc", "Cost", "Weight", "Mods / flags", "Effect"],
                      ["---", "---:", "---", "---:", "---:", "---", "---"], trows)
    # Books
    doc.h(2, "Spell books (%d)" % len(books))
    doc.p("Books are defined in `class.txt` (`book:` and `book-properties:` "
          "lines), not in `object.txt`. *Town* books are stocked by the "
          "Bookseller ([A9](A9-stores.md)); *dungeon* books are only found in the "
          "dungeon or the black market. Spells are listed in "
          "[A7](A7-class-spells.md).\n")
    trows = []
    for bk in books.values():
        trows.append([md(bk["name"]), md(bk["tval"]), bk["where"], md(bk["realm"]),
                      str(bk["cost"]), md(bk["alloc"]), md(", ".join(bk["classes"]))])
    doc.table(["Book", "Type", "Source", "Realm", "Cost", "Alloc", "Used by"],
              ["---", "---", "---", "---", "---:", "---", "---"], trows)
    return doc.write()


# ---------------------------------------------------------------------------
# Classes and races (A7, A8)
# ---------------------------------------------------------------------------

SKILLS = [("disarm-phys", "Disarm (phys)"), ("disarm-magic", "Disarm (magic)"),
          ("device", "Device"), ("save", "Save"), ("stealth", "Stealth"),
          ("search", "Search"), ("melee", "Melee"), ("shoot", "Shoot"),
          ("throw", "Throw"), ("dig", "Dig")]


def load_classes():
    classes = []
    cur = None
    book = None
    spell = None
    for k, v in read_lines("class.txt"):
        if k == "name":
            cur = {"name": v.strip(), "stats": [], "skills": {}, "equip": [], "titles": [],
                   "flags": [], "pflags": [], "magic": None, "books": [], "raw": {}}
            classes.append(cur)
            book = spell = None
            continue
        if cur is None:
            continue
        if k == "stats":
            cur["stats"] = [int(x) for x in v.split(":")]
        elif k.startswith("skill-"):
            p = v.split(":")
            cur["skills"][k[6:]] = (int(p[0]), int(p[1]) if len(p) > 1 else 0)
        elif k in ("hitdie", "exp", "max-attacks", "min-weight", "strength-multiplier"):
            cur["raw"][k] = v.strip()
        elif k == "title":
            cur["titles"].append(v.strip())
        elif k == "equip":
            cur["equip"].append(v.strip())
        elif k == "obj-flags":
            cur["flags"] += split_flags([v])
        elif k == "player-flags":
            cur["pflags"] += split_flags([v])
        elif k == "magic":
            p = v.split(":")
            cur["magic"] = {"first": int(p[0]), "weight": int(p[1]), "nbooks": int(p[2])}
        elif k == "book":
            p = v.split(":")
            book = {"tval": tv_norm(p[0]), "where": p[1].strip(), "name": p[2].strip(),
                    "nspells": p[3].strip() if len(p) > 3 else "",
                    "realm": p[4].strip() if len(p) > 4 else "", "spells": [],
                    "props": None, "graphics": None}
            cur["books"].append(book)
            spell = None
        elif k == "book-properties" and book is not None:
            p = v.split(":")
            book["props"] = {"cost": int(p[0]), "common": p[1].strip(), "alloc": p[2].strip()}
        elif k == "book-graphics" and book is not None:
            book["graphics"] = v.strip()
        elif k == "spell" and book is not None:
            p = v.split(":")
            spell = {"name": p[0].strip(), "level": int(p[1]), "mana": int(p[2]),
                     "fail": int(p[3]), "exp": int(p[4]), "rec": [], "desc": ""}
            book["spells"].append(spell)
        elif spell is not None and k in ("effect", "dice", "expr", "effect-yx", "effect-msg"):
            spell["rec"].append((k, v))
        elif spell is not None and k == "desc":
            spell["desc"] += v
    for c in classes:
        for b in c["books"]:
            for s in b["spells"]:
                s["effects"] = parse_effects(s["rec"])
                s["desc"] = re.sub(r"\s+", " ", s["desc"]).strip()
    return classes


def collect_books(classes):
    books = OrderedDict()
    for c in classes:
        for b in c["books"]:
            key = b["name"]
            if key not in books:
                books[key] = {"name": key, "tval": b["tval"], "where": b["where"],
                              "realm": b["realm"], "cost": 0, "alloc": "", "classes": []}
            bk = books[key]
            if b["props"] and not bk["alloc"]:
                bk["cost"] = b["props"]["cost"]
                bk["alloc"] = "%s (x%s)" % (b["props"]["alloc"].replace(" to ", "-"),
                                            b["props"]["common"])
            if c["name"] not in bk["classes"]:
                bk["classes"].append(c["name"])
    return books


def load_realms():
    out = OrderedDict()
    for r in parse_records("realm.txt"):
        n = first(r, "name").strip()
        out[n] = {k: (first(r, k) or "").strip() for k in
                  ("stat", "verb", "spell-noun", "book-noun")}
    return out


def gen_a7(classes, realms, books, stores):
    doc = Doc("A7-class-spells.md", "Appendix A7. Class spells",
              ["lib/gamedata/class.txt", "lib/gamedata/realm.txt", "lib/gamedata/store.txt",
               "lib/gamedata/projection.txt", "lib/gamedata/player_timed.txt",
               "src/list-effects.h"], [
        "Every spell of every class, book by book: level, mana, base failure "
        "rate, first-cast experience, the effect in words and the in-game "
        "description. Which spells to learn and when: "
        "[Chapter 3](../03-magic-and-spells.md) and the class playbooks in "
        "[Chapter 14](../14-class-playbooks.md).",
    ])
    town_books = set()
    for s in stores:
        for item in s["always"] + s["normal"]:
            tv, nm = item
            if nm is None:
                town_books.add(tv)
    doc.h(2, "Legend")
    doc.p(
        "- **Lvl**: character level needed to learn. **Mana**: SP cost. "
        "**Fail**: base failure rate before level and stat adjustments "
        "(see [Chapter 1](../01-core-mechanics.md) and the stat tables in "
        "[A8](A8-races-and-classes.md)). **Exp**: the first successful cast gives "
        "`Exp * Lvl` experience (`spell_cast()` in `src/player-spell.c`).",
        "- **Effect** is generated from the `effect:` lines: `plev` = character "
        "level, dice as in the data (`$` variables substituted with their "
        "expressions); `+1 per N plev` = radius/length grows every N levels.",
        "- **Source** of a book: *town* books are always stocked by the Bookseller "
        "(`always:` lines for every book type in `store.txt`); *dungeon* books are "
        "found at the listed depths (`book-properties:` `cost:commonness:min to "
        "max`).",
        "",
    )
    doc.h(2, "Realms")
    trows = [[md(n), md(r["stat"]), md(r["verb"]), md(r["spell-noun"]), md(r["book-noun"]),
              md(", ".join(c["name"] for c in classes
                           if any(b["realm"] == n for b in c["books"])))]
             for n, r in realms.items()]
    doc.table(["Realm", "Stat", "Verb", "Spell noun", "Book noun", "Classes"],
              ["---", "---", "---", "---", "---", "---"], trows)
    for c in classes:
        doc.h(2, c["name"])
        if not c["magic"]:
            doc.p("No spells (%s).\n" % md(", ".join(c["pflags"]) or "no magic"))
            continue
        realm = c["books"][0]["realm"] if c["books"] else "?"
        st = realms.get(realm, {}).get("stat", "?")
        doc.p("Realm **%s** (spell stat %s). First spell at level %d; total armour "
              "weight above %d (%s lb) costs 1 SP per pound over (`calc_mana()` in "
              "`src/player-calcs.c`). %d books. Class flags: %s.\n" % (
                  md(realm), st, c["magic"]["first"], c["magic"]["weight"],
                  "%d.%d" % (c["magic"]["weight"] // 10, c["magic"]["weight"] % 10),
                  c["magic"]["nbooks"], md(", ".join(c["pflags"]) or "-")))
        for b in c["books"]:
            bk = books.get(b["name"], {})
            if b["where"] == "town":
                src = "town book: Bookseller"
            else:
                src = "dungeon book: depth %s" % bk.get("alloc", "?")
            doc.h(3, "%s: %s" % (c["name"], b["name"].strip("[]")))
            doc.p("*%s, cost %s, %s spells for this class.*\n" % (
                md(src), bk.get("cost", "?"), len(b["spells"])))
            trows = []
            for s in b["spells"]:
                trows.append([md(s["name"]), str(s["level"]), str(s["mana"]), str(s["fail"]),
                              str(s["exp"]), cell(effects_text(s["effects"])), cell(s["desc"])])
            doc.table(["Spell", "Lvl", "Mana", "Fail", "Exp", "Effect", "Description"],
                      ["---", "---:", "---:", "---:", "---:", "---", "---"], trows)
    return doc.write()


def load_races():
    out = []
    for r in parse_records("p_race.txt"):
        d = {"name": first(r, "name").strip(),
             "stats": [int(x) for x in first(r, "stats").split(":")],
             "skills": {k[6:]: int(v) for k, v in r if k.startswith("skill-")},
             "flags": split_flags(every(r, "obj-flags")),
             "pflags": split_flags(every(r, "player-flags")),
             "values": split_flags(every(r, "values"))}
        for k in ("hitdie", "exp", "infravision", "age", "height", "weight", "history"):
            d[k] = (first(r, k) or "").strip()
        out.append(d)
    return out


def parse_c_table(path, name):
    """Parse 'const int name[...] = { v /* label */, ... };' from a C file."""
    txt = open(path, encoding="utf-8").read()
    m = re.search(r"(/\*\*(?:(?!\*/).)*\*/\s*)?(?:static\s+)?const\s+int(?:32_t)?\s+%s\s*\[[^\]]*\]\s*=\s*\{(.*?)\};"
                  % re.escape(name), txt, re.S)
    if not m:
        warn("C table %s not found in %s" % (name, path))
        return None, []
    doc_ = ""
    if m.group(1):
        doc_ = " ".join(l.strip(" */") for l in m.group(1).splitlines() if l.strip(" */"))
    body = m.group(2)
    entries = []
    for mm in re.finditer(r"(-?\d+)L?\s*(?:/\*\s*(.*?)\s*\*/)?\s*,?", body):
        entries.append((int(mm.group(1)), (mm.group(2) or "").strip()))
    return doc_, entries


def parse_blows_table(path):
    txt = open(path, encoding="utf-8").read()
    m = re.search(r"blows_table\[12\]\[12\]\s*=\s*\{(.*?)\};", txt, re.S)
    if not m:
        warn("blows_table not found")
        return []
    rows = re.findall(r"\{([^{}]*)\}", m.group(1))
    return [[int(x) for x in re.findall(r"-?\d+", r)] for r in rows]


def gen_a8(classes, races, kinds):
    pc = os.path.join(SRC, "src", "player-calcs.c")
    ps = os.path.join(SRC, "src", "player-spell.c")
    pl = os.path.join(SRC, "src", "player.c")
    doc = Doc("A8-races-and-classes.md", "Appendix A8. Races, classes and player tables",
              ["lib/gamedata/p_race.txt", "lib/gamedata/class.txt",
               "lib/gamedata/player_property.txt", "src/player-calcs.c",
               "src/player-spell.c", "src/player.c"], [
        "Race and class statistics (stat modifiers, skills, hit dice, experience "
        "factor, infravision, intrinsic flags), class melee-blow parameters, "
        "class skill progressions, starting kits and titles, the player "
        "experience table and the stat-to-bonus tables from the C source. "
        "Choosing a combination: [Chapter 2](../02-character-creation.md); the "
        "formulas behind the tables: [Chapter 1](../01-core-mechanics.md).",
    ])
    pprops = {}
    for r in parse_records("player_property.txt", ("type",)):
        c = (first(r, "code") or "").strip()
        if c:
            pprops[c] = ((first(r, "name") or "").strip(), joined_text(every(r, "desc")))
    oprops = {}
    for r in parse_records("object_property.txt"):
        c = (first(r, "code") or "").strip()
        if c:
            oprops[c] = (first(r, "name") or "").strip()

    doc.h(2, "How the numbers combine")
    doc.p(
        "- **Stats**: race + class modifier applied to the rolled/bought stat "
        "(`src/player-birth.c`, `src/player-calcs.c`).",
        "- **Skills**: race skill + class base + `class increment * level / 10` "
        "(`calc_bonuses()` in `src/player-calcs.c`), plus stat bonuses "
        "(`adj_dex_dis`, `adj_int_dis`, `adj_int_dev`, `adj_wis_sav`, "
        "`adj_str_dig` below) and equipment. Stealth is clamped to 0..30, digging "
        "to at least 1.",
        "- **Hit dice**: player hit die = race `hitdie` + class `hitdie`; HP per "
        "level is rolled from it, plus `adj_con_mhp / 100` per level.",
        "- **Experience factor**: race `exp` + class `exp` percent; the "
        "experience needed for level L is `player_exp[L-2] * expfact / 100`.",
        "- **Blows**: `calc_blows()` in `src/player-calcs.c`: "
        "`P = min(11, adj_str_blow[STR] * strength-multiplier / max(weapon weight, "
        "min-weight))`, `D = min(11, adj_dex_blow[DEX])`, energy per blow = "
        "`blows_table[P][D]`, blows = `min(100 / energy, max-attacks)` (as "
        "hundredths: `10000 / energy`), plus extra blows from equipment.",
        "",
    )
    doc.h(2, "Races")
    stat_h = ["STR", "INT", "WIS", "DEX", "CON"]
    trows = []
    for r in races:
        trows.append([md(r["name"])] + ["%+d" % x for x in r["stats"]] +
                     [r["hitdie"], r["exp"] + "%", "%d ft" % (int(r["infravision"] or 0) * 10)])
    doc.table(["Race"] + stat_h + ["Hit die", "Exp", "Infra"],
              ["---"] + ["---:"] * 5 + ["---:", "---:", "---:"], trows)
    trows = []
    for r in races:
        trows.append([md(r["name"])] + ["%+d" % r["skills"].get(k, 0) for k, _ in SKILLS])
    doc.table(["Race"] + [lab for _, lab in SKILLS], ["---"] + ["---:"] * len(SKILLS), trows)
    trows = []
    for r in races:
        ab = ["%s (%s)" % (oprops.get(f, f), f) for f in r["flags"]]
        ab += ["%s (%s): %s" % (pprops.get(f, (f, ""))[0], f, pprops.get(f, ("", ""))[1])
               for f in r["pflags"]]
        ab += fmt_values(r["values"])
        trows.append([md(r["name"]), cell("; ".join(ab)), md(r["age"]), md(r["height"]),
                      md(r["weight"])])
    doc.table(["Race", "Intrinsic abilities", "Age (base:rand)", "Height", "Weight"],
              ["---", "---", "---", "---", "---"], trows)

    doc.h(2, "Classes")
    trows = []
    for c in classes:
        raw = c["raw"]
        trows.append([md(c["name"])] + ["%+d" % x for x in c["stats"]] +
                     [raw.get("hitdie", "0"), (raw.get("exp", "0") or "0") + "%",
                      raw.get("max-attacks", "?"),
                      "%s (%s lb)" % (raw.get("min-weight", "?"),
                                      "%.1f" % (int(raw.get("min-weight", "0")) / 10.0)),
                      raw.get("strength-multiplier", "?")])
    doc.table(["Class"] + stat_h + ["Hit die", "Exp", "Max blows", "Min weight", "STR mult"],
              ["---"] + ["---:"] * 5 + ["---:", "---:", "---:", "---:", "---:"], trows)
    doc.p("Class skills: `base / +increment per 10 levels`.\n")
    trows = []
    for c in classes:
        trows.append([md(c["name"])] + ["%d / %+d" % c["skills"].get(k, (0, 0)) for k, _ in SKILLS])
    doc.table(["Class"] + [lab for _, lab in SKILLS], ["---"] + ["---:"] * len(SKILLS), trows)
    trows = []
    for c in classes:
        ab = ["%s (%s): %s" % (pprops.get(f, (f, ""))[0], f, pprops.get(f, ("", ""))[1])
              for f in c["pflags"]]
        ab += ["%s (%s)" % (oprops.get(f, f), f) for f in c["flags"]]
        mg = "-"
        if c["magic"]:
            mg = "%s, first spell level %d, armour weight limit %d.%d lb, %d books" % (
                c["books"][0]["realm"] if c["books"] else "?", c["magic"]["first"],
                c["magic"]["weight"] // 10, c["magic"]["weight"] % 10, c["magic"]["nbooks"])
        trows.append([md(c["name"]), cell("; ".join(ab)), md(mg)])
    doc.table(["Class", "Class abilities", "Magic"], ["---", "---", "---"], trows)

    doc.h(2, "Class skill progression")
    doc.p("Class contribution only (`base + increment * level / 10`, integer "
          "division); add the race skill, stat bonus and equipment. Stealth "
          "is shown before the 0..30 clamp.\n")
    levels = [1, 5, 10, 15, 20, 25, 30, 35, 40, 45, 50]
    for k, lab in SKILLS:
        trows = []
        for c in classes:
            b, inc = c["skills"].get(k, (0, 0))
            trows.append([md(c["name"])] + [str(b + c_div(inc * L, 10)) for L in levels])
        doc.h(3, "%s by level" % lab)
        doc.table(["Class"] + ["L%d" % L for L in levels], ["---"] + ["---:"] * len(levels), trows)

    doc.h(2, "Starting equipment")
    doc.p("`equip:` lines: item, quantity range, and birth options that remove "
          "it (e.g. `birth_no_recall` removes the Word of Recall scroll). Starting "
          "gold is `player:start-gold` (600) plus 50 per unspent point-buy point; "
          "the value of each kit item given is deducted from it, and with "
          "`birth_start_kit` off only one food item and one light are given "
          "(`player_outfit()` and `recalculate_stats()` in "
          "`src/player-birth.c`). See [Chapter 2](../02-character-creation.md).\n")
    trows = []
    for c in classes:
        items = []
        for e in c["equip"]:
            p = e.split(":")
            tv, nm = tv_norm(p[0]), p[1].strip()
            qty = p[2] if p[2] == p[3] else "%s-%s" % (p[2], p[3])
            opt = p[4].strip() if len(p) > 4 else "none"
            s = "%s x%s" % (nm, qty)
            if opt and opt != "none":
                s += " (not with %s)" % opt
            items.append(s)
        trows.append([md(c["name"]), md("; ".join(items))])
    doc.table(["Class", "Starting kit"], ["---", "---"], trows)

    doc.h(2, "Titles")
    doc.p("Title index = `(level - 1) / 5` (`src/ui-death.c`, character sheet).\n")
    hdr = ["Class"] + ["L%d-%d" % (i * 5 + 1, i * 5 + 5) for i in range(10)]
    trows = [[md(c["name"])] + [md(t) for t in c["titles"]] + [""] * (10 - len(c["titles"]))
             for c in classes]
    doc.table(hdr, ["---"] * len(hdr), trows)

    doc.h(2, "Experience table")
    _, exp = parse_c_table(pl, "player_exp")
    doc.p("`player_exp[]` in `src/player.c`: base experience to reach each level; "
          "multiply by the race+class experience factor / 100. The maximum level "
          "is 50 (`PY_MAX_LEVEL`), so the array's last entry is never used. "
          "Values below are for factor 100% and for the other factors that "
          "occur.\n")
    facs = sorted({int(r["exp"]) + int(c["raw"].get("exp", "0") or 0)
                   for r in races for c in classes})
    show = [100] + [f for f in (120, 150, 200, 250) if f in facs and f != 100]
    trows = []
    for i, (v, _) in enumerate(exp[:49]):  # levels 2..50 (PY_MAX_LEVEL = 50)
        trows.append(["%d" % (i + 2)] + ["%d" % (v * f // 100) for f in show])
    doc.table(["Level"] + ["exp at %d%%" % f for f in show], ["---:"] * (1 + len(show)), trows)
    doc.p("Experience factors of all race/class combinations range from %d%% to %d%%.\n"
          % (min(facs), max(facs)))

    doc.h(2, "Stat tables")
    doc.p("Transcribed by the generator from the C arrays named below. Index "
          "label is the stat value (`18/xx` = 18 plus xx percentile points). The "
          "comment above each array in the source is quoted as the meaning.\n")
    tables = [(pc, n) for n in ("adj_str_td", "adj_str_th", "adj_dex_th", "adj_dex_ta",
                                "adj_str_wgt", "adj_str_hold", "adj_str_dig", "adj_str_blow",
                                "adj_dex_blow", "adj_dex_safe", "adj_con_fix", "adj_con_mhp",
                                "adj_dex_dis", "adj_int_dis", "adj_int_dev", "adj_wis_sav",
                                "adj_mag_study", "adj_mag_mana")] + \
             [(ps, n) for n in ("adj_mag_fail", "adj_mag_stat")]
    parsed = []
    labels = None
    for path, n in tables:
        d, ents = parse_c_table(path, n)
        if not ents:
            continue
        if labels is None:
            labels = [l for _, l in ents]
        parsed.append((n, os.path.basename(path), d, [v for v, _ in ents]))
    doc.p("| Array | File | Meaning |", "|---|---|---|")
    for n, f, d, _ in parsed:
        doc.p("| `%s` | `src/%s` | %s |" % (n, f, md(d or "(no comment in source)")))
    doc.p("")
    # Split into two tables for width
    half = (len(parsed) + 1) // 2
    for chunk in (parsed[:half], parsed[half:]):
        hdr = ["Stat"] + [code(n.replace("adj_", "")) for n, _, _, _ in chunk]
        trows = []
        for i, lab in enumerate(labels or []):
            trows.append([md(lab)] + [str(v[i]) if i < len(v) else "" for _, _, _, v in chunk])
        doc.table(hdr, ["---"] + ["---:"] * len(chunk), trows)
    doc.h(3, "Blows table")
    bt = parse_blows_table(pc)
    doc.p("`blows_table[P][D]` in `src/player-calcs.c`: energy per blow (100 = one "
          "blow per turn). Rows P = STR index (`adj_str_blow * mult / weight`), "
          "columns D = DEX index (`adj_dex_blow`). Blows per round = "
          "`min(100 / energy, max blows)`.\n")
    trows = [["P=%d" % i] + [str(x) for x in row] for i, row in enumerate(bt)]
    doc.table(["P \\ D"] + ["D=%d" % i for i in range(12)], ["---"] + ["---:"] * 12, trows)
    return doc.write()


# ---------------------------------------------------------------------------
# Stores (A9)
# ---------------------------------------------------------------------------

def load_stores():
    names = {}
    for r in parse_records("terrain.txt", ("code",)):
        names[first(r, "code").strip()] = (first(r, "name") or "").strip()
    stores = []
    for r in parse_records("store.txt", ("store",)):
        code_ = first(r, "store").strip()
        def items(key):
            out = []
            for v in every(r, key):
                tv, _, nm = v.partition(":")
                out.append((tv_norm(tv), clean_name(nm) if nm.strip() else None))
            return out
        stores.append({
            "code": code_, "name": names.get(code_, code_),
            "owners": [o.split(":", 1) for o in every(r, "owner")],
            "slots": (first(r, "slots") or "").strip(),
            "turnover": (first(r, "turnover") or "").strip(),
            "always": items("always"), "normal": items("normal"),
            "buy": [tv_norm(x) for x in every(r, "buy")],
            "buyflag": every(r, "buy-flag"),
        })
    return stores


def gen_a9(stores, kinds, consts, books):
    doc = Doc("A9-stores.md", "Appendix A9. Stores",
              ["lib/gamedata/store.txt", "lib/gamedata/terrain.txt", "lib/gamedata/object.txt",
               "lib/gamedata/constants.txt", "src/store.c"], [
        "Each town store: owners and purse limits, stock slots and turnover, "
        "items always stocked, the list normal stock is drawn from (with base "
        "cost and effect), what the store buys; then the pricing and restocking "
        "rules from `src/store.c`. Shopping strategy: "
        "[Chapter 11](../11-town-and-shopping.md).",
    ])
    kind_by = {(k["tval"], k["name"].lower()): k for k in kinds}
    doc.h(2, "Rules from the source")
    sm = consts.get("store:magic-level", "?")
    doc.p(
        "FACT, from `src/store.c` unless noted:",
        "",
        "- **Buying from a store** (`price_item()`, store selling): price = the "
        "object's full value (`object_value_real()`), times the quantity; the "
        "**black market** doubles it and then applies an adjust factor of 150%, "
        "i.e. **3x value**.",
        "- **Selling to a store** (store buying): price = min(real value, known "
        "value) `* 2 / 3`; the black market pays half of that and applies a 50% "
        "factor, i.e. **1/6 of value**. The result is capped at the owner's purse "
        "(`owner:` max cost) per item. With `birth_no_selling` stores pay 0 (gold "
        "drops in the dungeon are increased instead, see "
        "[Chapter 11](../11-town-and-shopping.md)).",
        "- Stores refuse worthless items and anything not on their `buy:` list "
        "(`store_will_buy()`); the home accepts anything.",
        "- **Normal stock** is generated at object level `rand_range(1, %s + "
        "max(max_depth - 20, 0))` (capped at 70; `store:magic-level` = %s in "
        "`constants.txt`) with `apply_magic()` but no artifacts; items with "
        "negative to-hit/to-dam/AC or curses are rejected "
        "(`store_create_random()`). Each pick is uniform over the `normal:` list "
        "(duplicates count twice)." % (sm, sm),
        "- **Black market** stock is any object generated at level "
        "`max_depth + 5 .. max_depth + 20` (min capped at 55, max at 70); it only "
        "keeps egos, items with to-hit > 1, to-dam > 2 or AC bonus > 2, or items "
        "worth at least 10 gold that no other store stocks (`black_market_ok()`).",
        "- **Always** items are kept at a full stack (`max-stack`, 40 by default) "
        "and buying them does not deplete the stock (`store_maint()`).",
        "- **Turnover**: when the stores are updated, each store sells off 1d(turnover) "
        "items and then buys 1d(turnover) new ones, keeping the stock "
        "between `slots` min and max (+ always items) (`store_maint()`).",
        "- **Restock timing**: while you are in the dungeon a day counter "
        "increases every `10 * store:turns` = %d game turns (`process_world()` in "
        "`src/game-world.c`); on return to town each store is maintained once per "
        "elapsed day (`store_update()`), and owners are replaced with chance "
        "1 in `store:shuffle` (%s) per update. Buying out a store's last item "
        "triggers 10 maintenance passes." % (
            10 * int(consts.get("store:turns", "0") or 0), consts.get("store:shuffle", "?")),
        "- **Pile sizes** of generated stock (`mass_produce()`): food, flasks, "
        "lights: +3d5-3 if value <= 5 and again if <= 20; potions/scrolls: +3d5-3 "
        "if <= 60, +1d5-1 if <= 240; books: +2d3-2 if <= 50, +1d3-1 if <= 500; "
        "non-ego armour/weapons: +3d5-3 if <= 10 and again if <= 100; ammo: 20-40 "
        "if value <= 5, 10-40 if <= 50, 5-20 if <= 500, else 1.",
        "- Store inventory is capped at `store:inven-max` = %s slots." % consts.get("store:inven-max", "?"),
        "",
    )
    for s in stores:
        doc.h(2, s["name"])
        owners = "; ".join("%s (purse %s)" % (o[1].strip(), o[0].strip()) for o in s["owners"])
        doc.p("- **Code**: `%s`" % s["code"],
              "- **Owners**: %s" % md(owners))
        if s["slots"]:
            lo, _, hi = s["slots"].partition(":")
            doc.p("- **Normal slots**: %s-%s; **turnover**: %s" % (lo, hi, s["turnover"] or "0"))
        if s["buy"]:
            doc.p("- **Buys**: %s" % md(", ".join(s["buy"])))
        elif s["code"] == "STORE_BLACK":
            doc.p("- **Buys**: anything of value (no `buy:` list)")
        if s["buyflag"]:
            doc.p("- **Also buys**: %s" % md(", ".join(s["buyflag"])))
        doc.p("")

        def row(tv, nm, cnt=None):
            if nm is None:
                bks = [b for b in books.values() if b["tval"] == tv and b["where"] == "town"]
                return [md("all town %ss" % tv), md(tv), "-",
                        md(", ".join("%s (%d)" % (b["name"], b["cost"]) for b in bks)), "-"]
            k = kind_by.get((tv, nm.lower()))
            if k is None:
                warn("store %s: item %s:%s not found" % (s["name"], tv, nm))
                return [md(nm), md(tv), "?", "?", "?"]
            eff = effects_text(k["effects"])
            if not eff:
                bits = []
                if k["attack"] and tv in WEAPON_TV:
                    bits.append(k["attack"].split(":")[0])
                if k["armor"] and tv in ARMOR_TV:
                    bits.append("AC " + k["armor"].split(":")[0])
                if tv == "bow":
                    bits.append("x" + (k["pval"] or "?"))
                bits += obj_notes(k, {}, {})
                eff = ", ".join(bits)
            r = [md(nm), md(tv), str(k["level"]), str(k["cost"]), cell(eff)]
            if cnt is not None:
                r.insert(2, str(cnt))
            return r
        if s["always"]:
            doc.h(3, "%s: always in stock" % s["name"])
            doc.table(["Item", "Type", "Lvl", "Cost", "Effect / stats"],
                      ["---", "---", "---:", "---:", "---"],
                      [row(tv, nm) for tv, nm in s["always"]])
        if s["normal"]:
            cnt = OrderedDict()
            for it in s["normal"]:
                cnt[it] = cnt.get(it, 0) + 1
            doc.h(3, "%s: normal stock list" % s["name"])
            doc.p("*Entries* = how many times the item appears in the `normal:` "
                  "list (each restock pick is uniform over the entries).\n")
            doc.table(["Item", "Type", "Entries", "Lvl", "Cost", "Effect / stats"],
                      ["---", "---", "---:", "---:", "---:", "---"],
                      [row(tv, nm, n) for (tv, nm), n in cnt.items()])
    return doc.write()


# ---------------------------------------------------------------------------
# A10 curses, traps, timed effects, properties, runes
# ---------------------------------------------------------------------------

def gen_a10(curses, timed, kinds, acts, classes, props, slays, brands, shapes_src):
    doc = Doc("A10-curses-traps-timed-effects.md",
              "Appendix A10. Curses, traps, timed effects and properties",
              ["lib/gamedata/curse.txt", "lib/gamedata/trap.txt", "lib/gamedata/chest_trap.txt",
               "lib/gamedata/player_timed.txt", "lib/gamedata/player_property.txt",
               "lib/gamedata/object_property.txt", "lib/gamedata/object.txt",
               "lib/gamedata/activation.txt", "lib/gamedata/class.txt",
               "lib/gamedata/slay.txt", "lib/gamedata/brand.txt", "src/trap.c"], [
        "Every object curse and the item types it can appear on; every trap and "
        "rune with depth, effect and save; chest traps; every timed effect with "
        "its grades, what protects against it and every item, activation or "
        "spell that cures or reduces it; player and object properties; and the "
        "rune list used by identification. Advice: "
        "[Chapter 5](../05-elements-and-status.md), "
        "[Chapter 6](../06-equipment.md), [Chapter 9](../09-dungeon.md).",
    ])
    # Curses
    doc.h(2, "Curses")
    doc.p("From `curse.txt`. A curse can only be put on the listed object bases. "
          "*Combat* is `to-hit : to-dam : AC` adjustment; *Random effect* fires "
          "on average once every *time* turns. *Power* (on the item) decides how "
          "hard the curse is to remove. Items that carry fixed curses are listed "
          "after the table.\n")
    trows = []
    for c in curses.values():
        eff = effects_text(c["effects"])
        if eff and c["time"]:
            eff += " (every %s turns)" % c["time"]
        trows.append([md(c["name"]), md(c["desc"]), md(", ".join(c["types"])),
                      cell(c["combat"]), cell(", ".join(fmt_values(c["values"]))),
                      cell(", ".join(c["flags"])), cell(eff),
                      cell(", ".join(c["conflict"] + c["conflict_flags"]))])
    doc.table(["Curse", "Description", "Item types", "Combat", "Mods", "Flags",
               "Random effect", "Conflicts with"],
              ["---", "---", "---", "---", "---", "---", "---", "---"], trows)
    fixed = []
    for k in kinds:
        for cu in k["curses"]:
            fixed.append("%s %s: %s" % (k["tval"], k["name"], cu.replace(":", " power ")))
    for r in parse_records("artifact.txt"):
        for cu in every(r, "curse"):
            fixed.append("artifact %s: %s" % (art_name(r), cu.replace(":", " power ")))
    if fixed:
        doc.p("Fixed curses: " + md("; ".join(fixed)) + ".\n")

    # Traps
    doc.h(2, "Traps and runes")
    doc.p("From `trap.txt`; trap mechanics in `hit_trap()` (`src/trap.c`). "
          "*Appear* is `rarity : minimum depth : max number per level`. *Save*: "
          "an object flag listed on `save:` avoids the trap; `SAVE_ARMOR` = "
          "avoided if a monster-style hit roll of power 125 against your AC "
          "misses (`check_hit()`); `SAVE_THROW` = avoided if "
          "`randint0(100) < skill_save`. *Extra* effects happen 50% of the time "
          "when the trap fires. `trap immunity` (TRAP_IMMUNE) avoids all traps. "
          "*Disarm*: `MAGICAL` traps use the magical disarm skill, others the "
          "physical one. *Visibility* is the trap's hidden-ness value.\n")
    trows = []
    for r in parse_records("trap.txt"):
        nm = first(r, "name").split(":")
        name = nm[1].strip() if len(nm) > 1 else nm[0].strip()
        flags = split_flags(every(r, "flags"))
        effs = parse_effects(r, eff="effect", dice="dice", expr="expr")
        xtra = parse_effects(r, eff="effect-xtra", dice="dice-xtra", expr="expr-xtra")
        save = split_flags(every(r, "save"))
        save += [f for f in flags if f.startswith("SAVE_")]
        trows.append([md(name), md(nm[0].strip()), cell((first(r, "appear") or "").strip()),
                      cell((first(r, "visibility") or "").strip()),
                      cell(", ".join(f for f in flags if not f.startswith("SAVE_"))),
                      cell(effects_text(effs)), cell(effects_text(xtra)), cell(", ".join(save)),
                      cell(joined_text(every(r, "desc")))])
    doc.table(["Trap", "Shown as", "Appear", "Visibility", "Flags", "Effect", "Extra (50%)",
               "Save", "Description"],
              ["---", "---", "---", "---", "---", "---", "---", "---", "---"], trows)

    doc.h(2, "Chest traps")
    doc.p("From `chest_trap.txt`. A chest's trap is chosen from those with "
          "*level* <= the chest's level (`src/obj-chest.c`).\n")
    trows = []
    for r in parse_records("chest_trap.txt"):
        trows.append([md(first(r, "name").strip()), code(first(r, "code").strip()),
                      (first(r, "level") or "").strip(), cell(effects_text(parse_effects(r))),
                      "yes" if (first(r, "destroy") or "").strip() == "1" else "-",
                      "magical" if (first(r, "magic") or "").strip() == "1" else "physical",
                      cell((first(r, "msg") or "").strip())])
    doc.table(["Trap", "Code", "Level", "Effect", "Destroys contents", "Disarm skill", "Message"],
              ["---", "---", "---:", "---", ":-:", "---", "---"], trows)

    # Timed effects and cures
    doc.h(2, "Timed effects")
    cures = defaultdict(list)

    def scan(cat, label, effects):
        for e in effects:
            if e.name == "CURE":
                cures[e.sub].append((cat, label))
            elif e.name == "TIMED_DEC":
                cures[e.sub].append((cat, "%s (-%s)" % (label, dice_text(e.dice, e.exprs))))
            elif e.name == "TIMED_SET" and (e.dice or "").strip() == "0":
                cures[e.sub].append((cat, "%s (set 0)" % label))
    for k in kinds:
        scan(k["tval"], k["name"], k["effects"])
    for a in acts.values():
        scan("activation", a["name"], a["effects"])
    for c in classes:
        for b in c["books"]:
            for s in b["spells"]:
                scan("%s spell" % c["name"], s["name"], s["effects"])
    for nm, effs in shapes_src:
        scan("shape", nm, effs)

    def cure_text(name):
        groups = OrderedDict()
        for cat, label in sorted(set(cures.get(name, []))):
            groups.setdefault(cat, []).append(label)
        return "; ".join("**%s**: %s" % (md(cat), md(", ".join(v))) for cat, v in groups.items())
    doc.p("From `player_timed.txt`. *Grades* are `max duration: name` steps "
          "(for cuts and stuns the grade decides the damage/penalty). "
          "*Protection* lists the `fail:` conditions that prevent the effect "
          "(object flag, resistance, player flag or another timed effect). "
          "*Cured / reduced by* is found by scanning every object kind, "
          "activation and class spell for `CURE:` and `TIMED_DEC:` effects "
          "(artifact activations appear under their activation name, see "
          "[A4](A4-artifacts.md)).\n")
    trows = []
    for t in timed.values():
        grades = "; ".join("%s: %s" % (g[1], g[2]) for g in t["grades"] if len(g) > 2)
        prot = ", ".join(timed_fail_text(f) for f in t["fail"])
        extra = []
        if t["resist"]:
            extra.append("grants resist " + ", ".join(pname(x) for x in t["resist"]))
        if t["brand"]:
            extra.append("brand " + ", ".join(t["brand"]))
        if t["slay"]:
            extra.append("slay " + ", ".join(t["slay"]))
        for syn in t["synonym"]:
            fl, _, exact = syn.partition(":")
            if exact.strip() not in ("", "0"):
                extra.append("same as the %s flag" % fl)
            else:
                extra.append("gives the %s flag (plus other effects)" % fl)
        if "NONSTACKING" in t["flags"]:
            extra.append("NONSTACKING: cannot be extended while active")
        others = [f for f in t["flags"] if f != "NONSTACKING"]
        if others:
            extra.append(", ".join(others))
        if t["lower"]:
            extra.append("lower bound " + t["lower"])
        if t["begin"]:
            extra.append("on start: " + effects_text(t["begin"]))
        if t["end"]:
            extra.append("on end: " + effects_text(t["end"]))
        trows.append([code(t["name"]), md(t["desc"]), md(grades), cell(prot),
                      cell("; ".join(extra)), cure_text(t["name"]) or "none"])
    doc.table(["Effect", "Description", "Grades", "Protection", "Notes", "Cured / reduced by"],
              ["---", "---", "---", "---", "---", "---"], trows)

    # Player properties
    doc.h(2, "Player properties")
    doc.p("From `player_property.txt`: race/class abilities (`player`), object "
          "flags as they appear on the character sheet (`object`) and element "
          "templates (`element`).\n")
    trows = []
    for r in parse_records("player_property.txt", ("type",)):
        trows.append([md((first(r, "type") or "").strip()), code((first(r, "code") or "").strip()),
                      md((first(r, "name") or "").strip()), cell(joined_text(every(r, "desc")))])
    doc.table(["Type", "Code", "Name", "Description"], ["---", "---", "---", "---"], trows)

    # Object properties / runes
    doc.h(2, "Object properties and runes")
    doc.p("From `object_property.txt`. In 4.2 every property is a **rune**: once "
          "you learn a rune (by noticing it in use, by an Identify effect, or as "
          "listed under *Learned*), it is recognised on every object. *Learned* "
          "(`id-type`): `on wield` = known as soon as the item is wielded, "
          "`on effect` = when it does something, `timed` = after wearing it for "
          "a while. *Power* is the weight in object power/value calculations.\n")
    trows = []
    for p in props:
        trows.append([md(p["type"]), code(p["code"]), md(p["name"]), cell(p["subtype"]),
                      cell(p["id"]), cell(p["power"]), cell(p["desc"] or p["msg"])])
    doc.table(["Type", "Code", "Name", "Subtype", "Learned", "Power", "Description / message"],
              ["---", "---", "---", "---", "---", "---:", "---"], trows)
    doc.h(3, "Slays")
    trows = [[code(c), md(s["name"]), md(s["flag"]), "x" + s["mult"]] for c, s in slays.items()]
    doc.table(["Code", "Slays", "Monster flag/base", "Multiplier"], ["---", "---", "---", "---:"], trows)
    doc.h(3, "Brands")
    trows = [[code(c), md(b["name"]), md(b["verb"]), "x" + b["mult"], cell(b["resist"]),
              cell(b["vuln"])] for c, b in brands.items()]
    doc.table(["Code", "Brand", "Verb", "Multiplier", "Resisted by", "Extra vs"],
              ["---", "---", "---", "---:", "---", "---"], trows)
    doc.p("Rune categories: combat runes (to-hit, to-dam, AC bonuses), the "
          "modifiers and stats above, each resistance/immunity/vulnerability, "
          "each flag, each slay and brand, and each curse (curses are runes too, "
          "learned when the curse is noticed). See "
          "[Chapter 6](../06-equipment.md) for identification by use.\n")
    return doc.write()


def load_shapes():
    out = []
    for r in parse_records("shape.txt"):
        out.append((first(r, "name").strip(), parse_effects(r)))
    return out


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def git_commit(src):
    """Last commit touching the game source (src/, lib/), ignoring doc-only commits."""
    try:
        r = subprocess.run(["git", "-C", src, "log", "-1", "--format=%h", "--abbrev=9",
                            "--", "src", "lib"],
                           capture_output=True, text=True, timeout=20)
        if r.returncode == 0:
            return r.stdout.strip()
    except (OSError, subprocess.SubprocessError):
        pass
    return ""


def main(argv=None):
    global SRC, OUT, COMMIT
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--src", default=DEFAULT_SRC, help="Angband source root")
    ap.add_argument("--out", default=DEFAULT_OUT, help="output directory")
    ap.add_argument("--no-commit", action="store_true",
                    help="do not record the source git commit in the headers")
    args = ap.parse_args(argv)
    SRC = os.path.abspath(args.src)
    OUT = os.path.abspath(args.out)
    if not os.path.isfile(gd("monster.txt")):
        sys.exit("no lib/gamedata/monster.txt under %s (use --src)" % SRC)
    os.makedirs(OUT, exist_ok=True)
    COMMIT = "" if args.no_commit else git_commit(SRC)

    load_effect_descs()
    consts = load_constants()
    projs = load_projections()
    Ctx.proj = projs
    timed = load_timed()
    Ctx.timed = {k: (v["desc"] or k.lower()) for k, v in timed.items()}
    summons = load_summons()
    Ctx.summon = {k: v["desc"] for k, v in summons.items()}
    spell_types = load_spell_types()
    mspells = load_monster_spells(spell_types)
    for s in mspells:
        if not spell_types.get(s):
            warn("monster spell %s missing from list-mon-spells.h" % s)
    bases = load_bases()
    mons = load_monsters(bases)
    kinds = load_objects()
    obases = load_object_bases()
    acts = load_activations()
    slays = load_slays()
    brands = load_brands()
    props = load_obj_props()
    curses = load_curses()
    classes = load_classes()
    books = collect_books(classes)
    realms = load_realms()
    races = load_races()
    stores = load_stores()
    shapes = load_shapes()

    total = 0
    total += gen_a1(mons, mspells, projs, bases)
    qkinds = {(k["tval"], k["name"].lower()) for k in kinds if "QUEST_ART" in k["flags"]}
    quest_arts = []
    for r in parse_records("artifact.txt"):
        tv, _, nm = first(r, "base-object", "?:?").partition(":")
        if (tv_norm(tv), clean_name(nm).lower()) in qkinds:
            quest_arts.append(art_name(r))
    total += gen_a2(mons, mspells, projs, quest_arts)
    total += gen_a3(mons, mspells, projs, timed)
    total += gen_a4(acts, slays, brands, kinds)
    total += gen_a5(slays, brands, props)
    total += gen_a6(kinds, obases, slays, brands, books)
    total += gen_a7(classes, realms, books, stores)
    total += gen_a8(classes, races, kinds)
    total += gen_a9(stores, kinds, consts, books)
    total += gen_a10(curses, timed, kinds, acts, classes, props, slays, brands, shapes)
    print("total %d lines" % total)
    for w in WARNINGS:
        print("WARNING: " + w, file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
