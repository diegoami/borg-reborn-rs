"""Find changed Rust functions for the Track R carry report (stdlib only)."""
import json
import re
import subprocess
from pathlib import Path


def mask_non_code(text):
    """Blank comments and literal contents, keeping offsets and line breaks.

    Lifetimes are not character literals. Rust block comments can nest;
    raw-string delimiters must match. Strings containing braces or `fn`
    cannot change a function boundary.
    """
    raw_pattern = re.compile(r'(?:br|r)(#*)"')
    char_pattern = re.compile(r"'(?:\\(?:u\{[0-9a-fA-F_]+\}|x[0-9a-fA-F]{2}|.)|[^'\\\n])'")
    out = list(text)
    i = 0
    while i < len(text):
        start = i
        if text.startswith('//', i):
            end = text.find('\n', i)
            i = len(text) if end < 0 else end
        elif text.startswith('/*', i):
            depth = 1
            i += 2
            while i < len(text) and depth:
                if text.startswith('/*', i):
                    depth += 1
                    i += 2
                elif text.startswith('*/', i):
                    depth -= 1
                    i += 2
                else:
                    i += 1
        else:
            raw = raw_pattern.match(text, i)
            char = char_pattern.match(text, i)
            if raw:
                end = text.find('"' + raw[1], raw.end())
                i = len(text) if end < 0 else end + 1 + len(raw[1])
            elif text[i] == '"':
                i += 1
                while i < len(text):
                    if text[i] == '\\':
                        i += 2
                    elif text[i] == '"':
                        i += 1
                        break
                    else:
                        i += 1
            elif char:
                i = char.end()
            else:
                i += 1
                continue
        for j in range(start, min(i, len(text))):
            if text[j] != '\n':
                out[j] = ' '
    return ''.join(out)


def functions(text):
    """Map names to every definition of that name (including impl methods)."""
    code = mask_non_code(text)
    found = {}
    for match in re.finditer(r'\bfn\s+([A-Za-z_][A-Za-z_0-9]*)\s*(?:<|\()', code):
        # The signature can contain const generics; delimiters before the
        # body are balanced instead of treating a generic's brace as a body.
        pos = match.end() - 1
        parens = brackets = angles = 0
        body = None
        while pos < len(code):
            c = code[pos]
            if c == '(': parens += 1
            elif c == ')': parens -= 1
            elif c == '[': brackets += 1
            elif c == ']': brackets -= 1
            elif c == '<': angles += 1
            elif c == '>' and angles: angles -= 1
            elif c == ';' and parens == brackets == angles == 0: break
            elif c == '{' and parens == brackets == angles == 0:
                body = pos
                break
            pos += 1
        if body is None:
            continue
        depth = 1
        pos = body + 1
        while pos < len(code) and depth:
            depth += (code[pos] == '{') - (code[pos] == '}')
            pos += 1
        if depth:
            raise ValueError('unbalanced Rust function: ' + match[1])
        found.setdefault(match[1], []).append(text[text.rfind('\n', 0, match.start()) + 1:pos])
    return found


def read(repo, ref, path):
    result = subprocess.run(['git', '-C', str(repo), 'show', f'{ref}:{path}'], capture_output=True, text=True)
    if result.returncode:
        exists = subprocess.run(['git', '-C', str(repo), 'cat-file', '-e', ref + '^{commit}'], capture_output=True)
        if exists.returncode:
            raise ValueError(f'unknown commit: {ref}')
        listing = subprocess.run(['git', '-C', str(repo), 'ls-tree', ref, '--', path], capture_output=True, text=True, check=True)
        if listing.stdout.strip():
            raise ValueError(f'cannot read {ref}:{path}: {result.stderr.strip()}')
        return ''  # a genuinely added/deleted source file
    return result.stdout


def changed(repo, base, head, manifest):
    for ref in (base, head):
        result = subprocess.run(['git', '-C', str(repo), 'rev-parse', '--verify', ref + '^{commit}'], capture_output=True, text=True)
        if result.returncode:
            raise ValueError(f'unknown commit: {ref}')
    paths = subprocess.run(['git', '-C', str(repo), 'diff', '--name-only', base, head, '--', 'borg-rs/src'], capture_output=True, text=True, check=True).stdout.splitlines()
    paths = {p for p in paths if p.endswith('.rs')}
    ports = {(p['native_path'], p['native_function']): p['point'] for p in manifest['ports']}
    if len(ports) != len(manifest['ports']):
        raise ValueError('duplicate native port in manifest')
    rows = []
    ported = []
    for path in sorted(paths | {p for p, _ in ports}):
        old = functions(read(repo, base, path))
        new = functions(read(repo, head, path))
        for (native_path, name), point in ports.items():
            if native_path == path and name not in old and name not in new:
                raise ValueError(f'{path}:{name}: missing in both revisions')
        for name in sorted(set(old) | set(new)):
            if old.get(name) == new.get(name):
                continue
            row = {'path': path, 'function': name,
                   'change': 'added' if name not in old else 'removed' if name not in new else 'changed'}
            rows.append(row)
            if (path, name) in ports:
                ported.append(dict(row, point=ports[path, name]))
    return {'base': base, 'head': head, 'changed_functions': rows,
            'ported_changes': ported, 'changed_files': sorted(paths)}
