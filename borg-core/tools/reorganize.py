#!/usr/bin/env python3
"""Deduplicate c2rust `-r` output into one shared header module tree.

Input: a directory of c2rust files produced with
`c2rust transpile -r --disable-refactoring`. In each file, every header
declaration sits in a `#[c2rust::header_src = "PATH:LINE"] pub mod X_h { .. }`
block, and every item carries `#[c2rust::src_loc = "L:C"]`.

Output (into OUT_DIR):
  headers/mod.rs, headers/<name>.rs   one module per C header, items merged
                                      across all translation units
  c2rust/<file>.rs                    the per-.c code, importing from
                                      crate::headers
  c2rust/mod.rs

Rules:
* Items are identified by (header path, src_loc, kind). Anonymous
  `C2RustUnnamed*` items get one global name per identity, and every file is
  renamed to use it.
* Extern declarations (fn or static) of symbols that some Borg .c file defines
  (#[no_mangle]) become `pub use crate::c2rust::<module>::<sym>;`.
* Opaque `pub type X;` inside extern blocks become zero-sized #[repr(C)]
  structs (so stable Rust works).
* All `#[c2rust::...]` attributes and nightly feature lines are removed.
"""
import os
import re
import sys
from collections import OrderedDict

# --------------------------------------------------------------------------
# tiny Rust-ish scanner: find the matching brace, skipping literals/comments


def match_brace(s, i):
    """s[i] == '{'; return index of the matching '}'."""
    depth = 0
    n = len(s)
    while i < n:
        c = s[i]
        if c == '"':
            i += 1
            while s[i] != '"':
                if s[i] == '\\':
                    i += 1
                i += 1
        elif c == "b" and s.startswith('b"', i):
            i += 1
            continue
        elif c == "'":
            # char literal or lifetime
            m = re.match(r"'(\\.|[^\\'])'|'\\u\{[0-9a-fA-F]+\}'|'\\x[0-9a-fA-F]{2}'", s[i:])
            if m:
                i += len(m.group(0))
                continue
        elif s.startswith("//", i):
            i = s.index("\n", i)
        elif s.startswith("/*", i):
            i = s.index("*/", i) + 1
        elif c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return i
        i += 1
    raise ValueError("unbalanced")


HEADER_RE = re.compile(
    r'#\[c2rust::header_src = "([^"]+):\d+"\]\npub mod ([A-Za-z0-9_]+) \{')
SRC_LOC_RE = re.compile(r'#\[c2rust::src_loc = "(\d+:\d+)"\]\n')


def split_items(body):
    """Split a module body (text inside braces) into items.

    Returns a list of (src_loc or None, text). `use` lines get src_loc None.
    Items inside `extern "C" { }` are returned with kind prefix 'extern:'.
    """
    items = []
    i = 0
    n = len(body)
    pending_attrs = ""
    loc = None
    while i < n:
        # skip whitespace
        m = re.match(r"\s+", body[i:])
        if m:
            i += len(m.group(0))
            continue
        if i >= n:
            break
        if body.startswith('#[c2rust::src_loc', i):
            m = re.match(r'#\[c2rust::src_loc = "(\d+:\d+)"\]\s*', body[i:])
            loc = m.group(1)
            i += len(m.group(0))
            continue
        if body.startswith("#[", i):
            j = body.index("]\n", i) + 2
            pending_attrs += body[i:j].strip() + "\n"
            i = j
            continue
        if body.startswith('extern "C" {', i):
            j = match_brace(body, body.index("{", i))
            inner = body[body.index("{", i) + 1:j]
            for (l2, k2, t2) in split_items(inner):
                items.append((l2, "extern", t2))
            i = j + 1
            pending_attrs = ""
            continue
        # a single item: ends at ';' at depth 0, or at the closing brace of
        # its first top-level '{' if that comes before any ';'
        j = i
        depth_par = 0
        end = None
        if body.startswith("use ", i) or body.startswith("pub use ", i):
            end = body.index(";", i) + 1
            j = n
        while j < n:
            c = body[j]
            if c == '"':
                j += 1
                while body[j] != '"':
                    if body[j] == "\\":
                        j += 1
                    j += 1
            elif c in "([":
                depth_par += 1
            elif c in ")]":
                depth_par -= 1
            elif c == "{":
                k = match_brace(body, j)
                # struct/union/fn bodies end the item; `= Foo { .. };` not
                if depth_par == 0 and "=" not in body[i:j]:
                    end = k + 1
                    break
                j = k
            elif c == ";" and depth_par == 0:
                end = j + 1
                break
            j += 1
        if end is None:
            raise ValueError("cannot split item at: " + body[i:i + 200])
        text = body[i:end]
        kind = "use" if text.startswith("use ") or text.startswith("pub use ") else "item"
        items.append((loc if kind != "use" else None, kind, pending_attrs + text))
        pending_attrs = ""
        loc = None
        i = end
    return items


def item_name(text):
    t = re.sub(r"^(#\[[^\n]*\]\n)*", "", text)
    m = re.search(
        r"\b(?:type|const|static mut|static|struct|union|fn|enum)\s+([A-Za-z_][A-Za-z0-9_]*)", t)
    return m.group(1) if m else None


def norm_const(text):
    """Same macro constant translated in different contexts may differ only
    in redundant casts: `2 as ::core::ffi::c_int` vs `2`."""
    t = re.sub(r"\s+as\s+(::core::ffi::)?[A-Za-z0-9_:]+", "", text)
    t = re.sub(r"[()\s]", "", t)
    return t


def const_value(text):
    return norm_const(text.split("=", 1)[1])


def strip_c2rust_attrs(text):
    text = re.sub(r'[ \t]*#\[c2rust::[^\]]*\]\n', "", text)
    return text


UNNAMED_RE = re.compile(r"\bC2RustUnnamed(_\d+)?\b")


def main():
    src_dir, out_dir = sys.argv[1], sys.argv[2]
    files = sorted(f for f in os.listdir(src_dir) if f.endswith(".rs"))

    raw = {}
    for f in files:
        with open(os.path.join(src_dir, f)) as fh:
            raw[f] = fh.read()

    # ---- pass 1: collect header modules per file
    # per file: list of (path, modname, start, end, body)
    hdr_blocks = {}
    header_names = OrderedDict()  # path -> modname
    for f in files:
        s = raw[f]
        blocks = []
        for m in HEADER_RE.finditer(s):
            path, name = m.group(1), m.group(2)
            ob = m.end() - 1
            cb = match_brace(s, ob)
            blocks.append((path, name, m.start(), cb + 1, s[ob + 1:cb]))
        hdr_blocks[f] = blocks

    # global module names for header paths (unique)
    used = {}
    for f in files:
        for (path, name, _, _, _) in hdr_blocks[f]:
            if path in header_names:
                continue
            base = name
            k = 1
            while base in used and used[base] != path:
                k += 1
                base = "%s_%d" % (name, k)
            used[base] = path
            header_names[path] = base

    # ---- pass 2: canonical names for anonymous items
    canon = {}  # (path, loc) -> global name
    counter = [0]
    file_renames = {f: {} for f in files}
    for f in files:
        for (path, name, _, _, body) in hdr_blocks[f]:
            for (loc, kind, text) in split_items(body):
                nm = item_name(text)
                if nm and UNNAMED_RE.fullmatch(nm):
                    key = (path, loc)
                    if key not in canon:
                        canon[key] = "C2RustUnnamedH_%d" % counter[0]
                        counter[0] += 1
                    file_renames[f][nm] = canon[key]

    def rename(f, text):
        rn = file_renames[f]
        if not rn:
            return text
        return UNNAMED_RE.sub(lambda m: rn.get(m.group(0), m.group(0)), text)

    # ---- symbols defined by Borg .c files (#[no_mangle])
    defined = {}  # sym -> module
    for f in files:
        mod = f[:-3]
        s = raw[f]
        for m in re.finditer(
                r"#\[no_mangle\]\n(?:#\[[^\n]*\]\n)*pub (?:unsafe extern \"C\" fn|static mut|static) ([A-Za-z_][A-Za-z0-9_]*)", s):
            defined[m.group(1)] = mod

    # ---- pass 3: merge header items
    file_local = {}
    line_cache = {}

    def is_define(path, loc):
        if path not in line_cache:
            try:
                with open(path, errors="replace") as fh:
                    line_cache[path] = fh.read().split("\n")
            except OSError:
                line_cache[path] = []
        ln = int(loc.split(":")[0]) - 1
        lines = line_cache[path]
        return 0 <= ln < len(lines) and lines[ln].lstrip().startswith("#define")
    merged = OrderedDict()  # path -> OrderedDict(key -> text)
    merged_uses = OrderedDict()
    for f in files:
        for (path, name, _, _, body) in hdr_blocks[f]:
            d = merged.setdefault(path, OrderedDict())
            u = merged_uses.setdefault(path, OrderedDict())
            for (loc, kind, text) in split_items(body):
                text = rename(f, text)
                text = strip_c2rust_attrs(text).strip()
                if kind == "use":
                    # use super::x_h::{a, b};  -> split per name
                    m = re.match(r"(pub )?use super::([A-Za-z0-9_]+)::(.*);$", text, re.S)
                    if not m:
                        raise ValueError(text)
                    pub, hm, names = m.groups()
                    # find header path for that local mod name in this file
                    hpath = None
                    for (p2, n2, _, _, _) in hdr_blocks[f]:
                        if n2 == hm:
                            hpath = p2
                    gname = header_names[hpath]
                    names = names.strip()
                    if names.startswith("{"):
                        names = [x.strip() for x in names[1:-1].split(",") if x.strip()]
                    else:
                        names = [names]
                    for nm in names:
                        u[(gname, nm)] = True
                    continue
                nm = item_name(text)
                key = (loc, kind, nm)
                if kind == "item" and text.startswith("pub const ") and \
                        is_define(path, loc):
                    # a #define: c2rust types it per use site, keep per file
                    file_local.setdefault(f, OrderedDict())[(path, nm)] = text
                    continue
                if kind == "extern":
                    if re.match(r"(pub )?type [A-Za-z0-9_]+;$", text):
                        text = "#[repr(C)]\npub struct %s {\n    _opaque: [u8; 0],\n}" % nm
                        key = (loc, "opaque", nm)
                    elif nm in defined:
                        text = "pub use crate::c2rust::%s::%s;" % (defined[nm], nm)
                        key = (loc, "reexport", nm)
                    else:
                        text = 'extern "C" {\n    ' + text.replace("\n", "\n    ") + "\n}"
                if key in d:
                    if d[key] != text and text.startswith("pub const ") and \
                            const_value(d[key]) == const_value(text):
                        # enum constant typed via different (alias) names
                        sys.stderr.write("note: %s %s: keeping %r over %r\n" % (
                            os.path.basename(path), nm, d[key], text))
                        continue
                    if d[key] != text:
                        sys.stderr.write("CONFLICT %s %s\n--- A\n%s\n--- B\n%s\n" % (
                            path, key, d[key], text))
                        sys.exit(1)
                else:
                    d[key] = text

    # opaque (forward-declared) structs: point them at the full definition
    # if any header has one, else keep a single opaque definition
    full_def = {}
    for path, d in merged.items():
        for key, text in d.items():
            if key[1] == "item" and re.search(r"\bpub (struct|union) %s\b" % re.escape(key[2] or "#"), text):
                full_def.setdefault(key[2], header_names[path])
    opaque_home = {}
    opaque_names = set()
    for path, d in merged.items():
        for key in list(d.keys()):
            if key[1] != "opaque":
                continue
            nm = key[2]
            opaque_names.add(nm)
            g = header_names[path]
            home = full_def.get(nm) or opaque_home.setdefault(nm, g)
            if home != g:
                d[key] = "pub use super::%s::%s;" % (home, nm)

    os.makedirs(os.path.join(out_dir, "headers"), exist_ok=True)
    os.makedirs(os.path.join(out_dir, "c2rust"), exist_ok=True)
    with open(os.path.join(out_dir, "headers", "mod.rs"), "w") as fh:
        fh.write("//! Declarations from C headers (engine, libc and Borg headers),\n"
                 "//! deduplicated across the c2rust translation units\n"
                 "//! (generated by tools/reorganize.py).\n")
        for path, gname in header_names.items():
            fh.write("pub mod %s;\n" % gname)
    for path, gname in header_names.items():
        with open(os.path.join(out_dir, "headers", gname + ".rs"), "w") as fh:
            fh.write("//! `%s`\n" % path)
            fh.write("#[allow(unused_imports)]\nuse super::*;\n")
            uses = merged_uses.get(path, {})
            seen = set()
            own = set(k[2] for k, t in merged[path].items()
                      if not t.startswith("pub use "))
            for (hm, nm) in uses:
                if nm in opaque_names:
                    hm = full_def.get(nm) or opaque_home[nm]
                if hm == gname or nm in seen or nm in own:
                    continue
                seen.add(nm)
                fh.write("pub use super::%s::%s;\n" % (hm, nm))
            for key, text in merged[path].items():
                m = re.match(r"pub use super::[A-Za-z0-9_]+::([A-Za-z0-9_]+);$", text)
                if m:
                    if m.group(1) in seen:
                        continue
                    seen.add(m.group(1))
                fh.write(text + "\n")

    # ---- pass 4: rewrite the .c modules
    for f in files:
        s = raw[f]
        # remove header blocks (from the end so offsets stay valid)
        for (path, name, a, b, _) in reversed(hdr_blocks[f]):
            s = s[:a] + s[b:]
        local_to_global = {name: header_names[path] for (path, name, _, _, _) in hdr_blocks[f]}
        local_path = {name: path for (path, name, _, _, _) in hdr_blocks[f]}
        s = rename(f, s)
        locals_ = file_local.get(f, {})

        def fix_use(m):
            pub, hm, names = m.group(1) or "", m.group(2), m.group(3)
            path = local_path.get(hm)
            if names.startswith("{"):
                lst = [x.strip() for x in names[1:-1].split(",") if x.strip()]
            else:
                lst = [names]
            keep = [x for x in lst if (path, x) not in locals_]
            if not keep:
                return ""
            return "%suse self::%s::{%s};" % (pub, hm, ", ".join(keep))
        s = re.sub(r"(pub )?use self::([A-Za-z0-9_]+)::(\{[^}]*\}|[A-Za-z0-9_]+);",
                   fix_use, s)
        if locals_:
            consts = "\n".join(locals_.values()) + "\n"
            s = consts + s
        s = re.sub(r"\bself::([A-Za-z0-9_]+)::",
                   lambda m: ("crate::headers::%s::" % local_to_global[m.group(1)])
                   if m.group(1) in local_to_global else m.group(0), s)
        s = strip_c2rust_attrs(s)
        s = re.sub(r"#!\[allow\([^\]]*\)\]\n", "", s)
        s = re.sub(r"#!\[(register_tool|feature)\([^\]]*\)\]\n", "", s)
        # opaque extern types declared in the .c itself
        def fix_extern(m):
            block = m.group(0)
            opq = re.findall(r"^\s*pub type ([A-Za-z0-9_]+);\s*$", block, re.M)
            block = re.sub(r"^\s*pub type ([A-Za-z0-9_]+);\s*\n", "", block, flags=re.M)
            decl = "".join("#[repr(C)]\npub struct %s {\n    _opaque: [u8; 0],\n}\n" % o for o in opq)
            return decl + block
        s = re.sub(r'(?ms)^extern "C" \{\n.*?^\}\n', fix_extern, s)
        s = re.sub(r'(?m)^extern "C" \{\s*\}\n', "", s)
        s = s.lstrip("\n")
        with open(os.path.join(out_dir, "c2rust", f), "w") as fh:
            fh.write(s)
    with open(os.path.join(out_dir, "c2rust", "mod.rs"), "w") as fh:
        fh.write("//! c2rust translation of angband/src/borg/*.c, one module per file\n"
                 "//! (generated by tools/transpile.sh).\n")
        for f in files:
            fh.write("pub mod %s;\n" % f[:-3])


if __name__ == "__main__":
    main()
