#!/usr/bin/env python3
"""Report and refresh what changed in the sources this repository pins.

Everything in pins.json keeps moving: angband/angband, better-borg-c,
borg-rs, borg-agent, the C Borg specification, the zoo catalogue and,
above all, the death reports. This tool is how a session keeps up
(task T1.0, issue #3).

  scripts/refresh-sources.py                  report only (read only; default)
  scripts/refresh-sources.py --no-fetch       report from what vendor/ already has
  scripts/refresh-sources.py --update [PIN..] move pins to their heads, refresh the
                                              copies in reference/, regenerate the
                                              death index (do it in its own PR)
  scripts/refresh-sources.py --index          regenerate the death index only

The report covers, per pin: commits since the pin on the default branch (or
on the pin's `branch`, when it names one);
files of each copy rule added, changed or removed; new death cases; new tags
(a new better-borg-c milestone tag means a new control arm, GOALS D8); C
Borg functions cited in this repository whose code changed; issues updated
since the pin was read (when `gh` works). Network or `gh` failures are
reported, never fatal. Standard library only.
"""

import argparse
import datetime
import json
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
INDEX = Path('reference/deaths/INDEX.md')
REGISTRIES = [('better-borg-c', Path('reference/deaths/better-borg-c')),
              ('borg-rs', Path('reference/deaths/borg-rs')),
              ('borg-reborn-rs', Path('docs/deaths'))]
# Citations of C Borg code: file:function[:lines], e.g. borg-danger.c:borg_danger:2824-2865
CITATION = re.compile(r'\b(borg[\w-]*\.c):(\w+)(?::\d+(?:-\d+)?)?')
CITED_IN = ['borg-reborn-rs', 'docs']


def git(repo, *args, check=False):
    return subprocess.run(['git', '-C', str(repo), *args], capture_output=True, text=True, check=check)


class Source:
    def __init__(self, root, name, pin):
        self.root, self.name, self.pin = root, name, pin
        # A pin that follows a branch other than the default one has its own clone
        # (`vendor`), so two pins of one repository never fight over a checkout.
        self.repo = root / 'vendor' / pin.get('vendor', pin['repository'].split('/')[1])
        self.ref = f"refs/remotes/origin/{pin['branch']}" if pin.get('branch') else 'refs/remotes/origin/HEAD'
        self.head = None
        self.problems = []

    def resolve_head(self, fetch):
        if not (self.repo / '.git').exists():
            self.problems.append(f'no clone in {self.repo.relative_to(self.root)} (run scripts/setup.sh)')
            return
        if fetch:
            r = git(self.repo, 'fetch', '-q', '--tags', 'origin')
            if r.returncode:
                last = (r.stderr.strip().splitlines() or ['no message'])[-1]
                self.problems.append(f'fetch failed, using what is already fetched (offline?): {last}')
            if not self.pin.get('branch') and \
                    git(self.repo, 'rev-parse', '-q', '--verify', 'refs/remotes/origin/HEAD').returncode:
                git(self.repo, 'remote', 'set-head', 'origin', '-a')
        r = git(self.repo, 'rev-parse', '-q', '--verify', self.ref + '^{commit}')
        if r.returncode:
            self.problems.append(f"cannot resolve {self.ref.removeprefix('refs/remotes/')}")
            return
        self.head = r.stdout.strip()

    def tree(self, commit, path):
        """{relative path: blob id} under path at commit."""
        r = git(self.repo, 'ls-tree', '-r', commit, '--', path)
        out = {}
        for line in r.stdout.splitlines():
            meta, p = line.split('\t', 1)
            out[p[len(path):].lstrip('/') if path else p] = meta.split()[2]
        return out


def selected(rule, tree):
    """The files of a copy rule, from {path: blob}."""
    kind = rule.get('select', 'files')
    if kind == 'files':
        return {p: b for p, b in tree.items() if p in rule['files']}
    if kind == 'all':
        return dict(tree)
    if kind == 'death-registry':
        out = {}
        for p, b in tree.items():
            parts = p.split('/')
            if len(parts) == 1 or (len(parts) == 2 and p.endswith(('.md', '.json'))):
                out[p] = b
        return out
    raise ValueError(f'unknown select {kind}')


def cases(files):
    return sorted({p.split('/')[0] for p in files if '/' in p})


def citations(root):
    found = set()
    for top in CITED_IN:
        for f in (root / top).rglob('*'):
            if f.is_file() and f.suffix in ('.rs', '.md') and 'target' not in f.parts:
                found.update(CITATION.findall(f.read_text(errors='replace')))
    return sorted(found)


def changed_functions(src, cited, borg_dir):
    """Cited (file, function) pairs whose code changed between pin and head."""
    hits = []
    for fname, func in cited:
        path = f'{borg_dir}/{fname}'
        diff = git(src.repo, 'diff', '-U0', src.pin['commit'], src.head, '--', path).stdout
        if not diff:
            continue
        headers = re.findall(r'^@@[^@]*@@(.*)$', diff, re.M)
        if any(func in h for h in headers) or re.search(rf'^[+-].*\b{func}\s*\(', diff, re.M):
            hits.append(f'{fname}:{func}')
    return hits


def report(root, pins, fetch=True, issues=True, out=sys.stdout):
    data = json.loads((root / 'pins.json').read_text())
    cited = citations(root)
    w = out.write
    w(f"# Source refresh report ({datetime.date.today().isoformat()})\n\n")
    stale = False
    for name in pins:
        pin = data['pins'][name]
        src = Source(root, name, pin)
        src.resolve_head(fetch)
        branch = f", branch {pin['branch']}" if pin.get('branch') else ''
        w(f"## {name} ({pin['repository']}{branch})\n\n")
        for p in src.problems:
            w(f'- problem: {p}\n')
        if not src.head:
            w('\n')
            continue
        if src.head == pin['commit']:
            w(f"- at pin {pin['commit'][:12]}, nothing new\n")
        else:
            n = git(src.repo, 'rev-list', '--count', f"{pin['commit']}..{src.head}").stdout.strip()
            w(f"- **{n or '?'} commits** since pin {pin['commit'][:12]} (head {src.head[:12]}, read on {pin.get('read_on', 'unknown')})\n")
            stale = True
        for rule in data.get('copies', []):
            if rule['pin'] != name:
                continue
            old = selected(rule, src.tree(pin['commit'], rule['from']))
            new = selected(rule, src.tree(src.head, rule['from']))
            added = sorted(set(new) - set(old))
            removed = sorted(set(old) - set(new))
            changed = sorted(p for p in set(old) & set(new) if old[p] != new[p])
            if added or removed or changed:
                w(f"- copy `{rule['from']}` → `{rule['to']}`: {len(added)} added, {len(changed)} changed, {len(removed)} removed\n")
                if rule.get('select') == 'death-registry':
                    new_cases = sorted(set(cases(new)) - set(cases(old)))
                    if new_cases:
                        w(f"  - **new death cases:** {', '.join(new_cases)}\n")
                    touched = sorted({p.split('/')[0] for p in changed if '/' in p} - set(new_cases))
                    if touched:
                        w(f"  - updated cases: {', '.join(touched)}\n")
                    new_docs = [p for p in added if '/' not in p]
                    if new_docs:
                        w(f"  - **new registry documents:** {', '.join(new_docs)}\n")
                    changed_docs = [p for p in changed if '/' not in p]
                    if changed_docs:
                        w(f"  - changed registry documents: {', '.join(changed_docs)}\n")
        tag_pattern = pin.get('watch_tags')
        if tag_pattern:
            # Tags after the baseline in use, not after the pin: a milestone
            # tagged before the pin moved past it is still a new control arm.
            baseline = pin.get('baseline') or {}
            since = baseline.get('commit') or pin['commit']
            tags = git(src.repo, 'tag', '--list', tag_pattern, '--contains', since).stdout.split()
            known = set(baseline.get('tag', '').split())
            new_tags = [t for t in tags if t not in known]
            if new_tags:
                w(f"- **new tags** `{tag_pattern}`: {', '.join(new_tags)} (a new milestone means a new control arm, GOALS D8)\n")
        borg_dir = pin.get('borg_dir')
        if borg_dir:
            if not cited:
                w('- cited C Borg functions: none cited in this repository yet\n')
            else:
                hits = changed_functions(src, cited, borg_dir)
                if hits:
                    w(f"- **cited C Borg functions changed:** {', '.join(hits)}\n")
        if issues and pin.get('issues'):
            since = pin.get('read_on')
            if since and shutil.which('gh'):
                r = subprocess.run(['gh', 'issue', 'list', '-R', pin['repository'], '--state', 'all',
                                    '--search', f'updated:>={since}', '--limit', '200',
                                    '--json', 'number,title,state'], capture_output=True, text=True)
                if r.returncode == 0:
                    items = json.loads(r.stdout)
                    w(f"- issues updated since {since}: {len(items)}\n")
                    for i in items[:10]:
                        w(f"  - #{i['number']} [{i['state'].lower()}] {i['title'][:100]}\n")
                else:
                    w('- issues: gh failed (' + (r.stderr.strip().splitlines() or ['?'])[-1] + ')\n')
            elif not since:
                w('- issues: no read_on date on this pin\n')
            else:
                w('- issues: gh not available\n')
        w('\n')
    return stale


def update(root, pins, fetch=True, today=None):
    """Move pins to their heads and refresh their copies, in two phases (M1
    review): first resolve every head and read every blob into memory,
    writing nothing; only when all of that succeeded, write copies, README
    pins, pins.json, vendor checkouts and the index. A failure in either
    phase leaves the repository as it was: the first writes nothing, the
    second restores what it had written."""
    path = root / 'pins.json'
    data = json.loads(path.read_text())
    today = today or datetime.date.today().isoformat()
    readme = root / 'reference/README.md'
    plans = []
    for name in pins:
        pin = data['pins'][name]
        src = Source(root, name, pin)
        src.resolve_head(fetch)
        if not src.head:
            raise SystemExit(f'{name}: cannot update: {"; ".join(src.problems)}')
        copies = [(rule, read_copy(src, rule, src.head)) for rule in data.get('copies', []) if rule['pin'] == name]
        plans.append((name, pin, src, copies))
    # Phase two writes. Everything it can touch is saved first, so a failure
    # part-way (a write, a checkout) restores the copies, the README,
    # pins.json, the index and the vendor checkouts it moved (re-review N10).
    saved = Snapshot(root, [rule for _, _, _, copies in plans for rule, _ in copies],
                     [path, readme, root / INDEX])
    vendor_moved = []
    try:
        moved = []
        for name, pin, src, copies in plans:
            for rule, files in copies:
                write_copy(root, rule, files)
            old = pin['commit']
            if old != src.head:
                moved.append((name, old, src.head))
                if readme.exists():
                    readme.write_text(readme.read_text().replace(old, src.head))
            pin['commit'] = src.head
            pin['read_on'] = today
            # Keep vendor/ at the pin, as scripts/setup.sh expects, unless it has local changes.
            if git(src.repo, 'status', '--porcelain').stdout == '':
                was = git(src.repo, 'rev-parse', 'HEAD').stdout.strip()
                r = git(src.repo, 'checkout', '-q', '--detach', src.head)
                if r.returncode:
                    raise SystemExit(f'{name}: checkout of {src.head[:12]} failed: {r.stderr.strip()}')
                vendor_moved.append((src.repo, was))
        path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + '\n')
        write_index(root)
    except BaseException:
        saved.restore()
        for repo, was in reversed(vendor_moved):
            git(repo, 'checkout', '-q', '--detach', was)
        raise
    saved.discard()
    return moved


class Snapshot:
    """The state of every path an update writes: copy destinations (copied
    whole into a fresh backup directory of its own, or the listed files of a
    'files' rule) and single files. The backup directory is new and owned by
    the snapshot, so nothing that was already there is touched (re-review
    R5); it is on the same filesystem as the copies, so restore can move
    them back."""

    def __init__(self, root, rules, files):
        self.root = root
        self.aside = []   # (dest, backup dir or None if dest did not exist)
        self.files = {}   # path: bytes, or None if it did not exist
        self.dir = Path(tempfile.mkdtemp(prefix='.refresh-backup-', dir=root))
        try:
            for rule in rules:
                dest = root / rule['to']
                if rule.get('select', 'files') == 'files':
                    for f in rule['files']:
                        self.keep(dest / f)
                elif not any(d == dest for d, _ in self.aside):
                    backup = self.dir / str(len(self.aside)) if dest.exists() else None
                    if backup:
                        shutil.copytree(dest, backup, symlinks=True)
                    self.aside.append((dest, backup))
            for f in files:
                self.keep(f)
        except BaseException:
            self.discard()
            raise

    def keep(self, f):
        if f not in self.files:
            self.files[f] = f.read_bytes() if f.exists() else None

    def restore(self):
        for dest, backup in self.aside:
            if dest.exists():
                shutil.rmtree(dest)
            if backup:
                backup.rename(dest)
        for f, content in self.files.items():
            if content is None:
                f.unlink(missing_ok=True)
            else:
                f.parent.mkdir(parents=True, exist_ok=True)
                f.write_bytes(content)
        self.discard()

    def discard(self):
        shutil.rmtree(self.dir, ignore_errors=True)


def read_copy(src, rule, commit):
    """{relative path: bytes} of a copy rule at commit; raises before any write."""
    files = selected(rule, src.tree(commit, rule['from']))
    out = {}
    for rel, blob in sorted(files.items()):
        r = subprocess.run(['git', '-C', str(src.repo), 'cat-file', 'blob', blob], capture_output=True)
        if r.returncode:
            raise SystemExit(f'{src.name}: cannot read {rule["from"]}/{rel} ({blob[:12]}) at {commit[:12]}: '
                             f'{r.stderr.decode().strip()}; nothing was changed')
        out[rel] = r.stdout
    return out


def write_copy(root, rule, files):
    dest = root / rule['to']
    if rule.get('select', 'files') == 'files':
        for f in rule['files']:
            (dest / f).unlink(missing_ok=True)
    elif dest.exists():
        shutil.rmtree(dest)
    for rel, content in sorted(files.items()):
        target = dest / rel
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(content)


def load(p):
    try:
        return json.loads(p.read_text())
    except (OSError, ValueError):
        return {}


def write_index(root):
    rows = []
    families = {}
    for registry, base in REGISTRIES:
        base = root / base
        if not base.exists():
            continue
        for case in sorted(d for d in base.iterdir() if d.is_dir() and re.match(r'D\d+', d.name)):
            rec, ass = load(case / 'record.json'), load(case / 'assessment.json')
            term = rec.get('terminal') or {}
            ident = rec.get('identity') if isinstance(rec.get('identity'), dict) else {}
            title = ''
            report_md = case / 'report.md'
            if report_md.exists():
                first = report_md.read_text(errors='replace').splitlines()[:1]
                title = first[0].lstrip('# ').strip() if first else ''
            # The assessment's family is the qualified grouping (replays of one encounter
            # share it); a record's own family hashes its own start save.
            family = ass.get('encounter_family') or rec.get('encounter_family') or ''
            families.setdefault((registry, family), []).append(case.name)
            rows.append(dict(
                registry=registry, case=case.name, title=title,
                character=str(rec.get('character') or ''),
                cause=str(rec.get('raw_cause') or term.get('cause') or ident.get('cause') or ''),
                turn=term.get('turn') or ident.get('turn') or '',
                depth=term.get('depth', ''), level=term.get('level', ''),
                classification=str(ass.get('classification') or ''),
                family=family))
    lines = ['# Death index', '',
             '*Generated by `scripts/refresh-sources.py`; do not edit. Every Borg death we know of:',
             'the copies of better-borg-c and borg-rs under `reference/deaths/`, and our own',
             '`docs/deaths/`. Cases sharing an encounter family start from the same state, so they',
             'are not independent.*', '',
             '| Registry | Case | Character | Cause | Turn | Depth | Level | Shares start with | Classification | Title |',
             '| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |']
    for r in rows:
        same = [c for c in families.get((r['registry'], r['family']), []) if c != r['case']] if r['family'] else []
        cell = lambda s: str(s).replace('|', '\\|').replace('\n', ' ')
        lines.append('| ' + ' | '.join(cell(x) for x in (
            r['registry'], f"[{r['case']}]({rel_link(root, r)})",
            r['character'], r['cause'], r['turn'], r['depth'], r['level'],
            ', '.join(same) or '—', r['classification'][:120], r['title'][:120])) + ' |')
    lines.append('')
    lines.append(f'{len(rows)} cases.')
    (root / INDEX).parent.mkdir(parents=True, exist_ok=True)
    (root / INDEX).write_text('\n'.join(lines) + '\n')
    return len(rows)


def rel_link(root, r):
    """Link from the index to the case's report (or its directory)."""
    case = dict(REGISTRIES)[r['registry']] / r['case']
    up = Path(*['..'] * len(INDEX.parent.parts))
    target = case / 'report.md' if (root / case / 'report.md').exists() else case
    return (up / target).as_posix()


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('pins', nargs='*', help='pins to report or update (default: all)')
    ap.add_argument('--update', action='store_true')
    ap.add_argument('--index', action='store_true')
    ap.add_argument('--no-fetch', action='store_true')
    ap.add_argument('--no-issues', action='store_true')
    ap.add_argument('--root', type=Path, default=ROOT, help=argparse.SUPPRESS)
    a = ap.parse_args(argv)
    data = json.loads((a.root / 'pins.json').read_text())
    pins = a.pins or list(data['pins'])
    unknown = [p for p in pins if p not in data['pins']]
    if unknown:
        ap.error(f'unknown pins: {unknown}')
    if a.index:
        print(f'death index: {write_index(a.root)} cases -> {INDEX}')
        return 0
    if a.update:
        for name, old, new in update(a.root, pins, fetch=not a.no_fetch):
            print(f'{name}: {old[:12]} -> {new[:12]}')
        print(f'pins, reference/ copies and {INDEX} updated; commit them in their own PR')
        return 0
    report(a.root, pins, fetch=not a.no_fetch, issues=not a.no_issues)
    return 0


if __name__ == '__main__':
    sys.exit(main())
