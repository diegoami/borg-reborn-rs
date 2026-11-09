# Adapted from better-borg-c a4084032485a7c7f29153d0b37ce63dede3e4d5a.
# GPL-2.0-or-later or Angband licence; see angband/docs/copying.rst.
"""Short progress reports and native character sheets from preserved run saves.

Reads the version-1 blocks written by src/savefile.c and src/save.c. Native
character formatting uses the run's frozen engine in an isolated save load,
with no Borg activation or gameplay input. It never replays from birth.
"""
import hashlib
import json
import os
from pathlib import Path
import shutil
import struct
import subprocess
import time


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


class Reader:
    def __init__(self, data):
        self.data, self.pos = data, 0

    def take(self, size):
        if size < 0 or self.pos + size > len(self.data):
            raise ValueError('Truncated native save block')
        value = self.data[self.pos:self.pos + size]
        self.pos += size
        return value

    def number(self, fmt):
        return struct.unpack('<' + fmt, self.take(struct.calcsize('<' + fmt)))[0]

    def string(self):
        end = self.data.find(b'\0', self.pos)
        if end < 0:
            raise ValueError('Unterminated native save string')
        return self.take(end - self.pos + 1)[:-1].decode('utf-8')


def saved_character(path):
    data = path.read_bytes()
    if data[:8] != b'SaveVNLA':
        raise ValueError('Unsupported native save format')
    pos, blocks = 8, {}
    while pos < len(data):
        if len(data) - pos < 28:
            raise ValueError('Truncated native save header')
        name, version, size, checksum = struct.unpack_from('<16sIII', data, pos)
        pos += 28
        payload = data[pos:pos + size]
        if len(payload) != size or sum(payload) & 0xffffffff != checksum:
            raise ValueError('Native save block checksum/size mismatch')
        name = name.rstrip(b'\0').decode('ascii')
        if name in blocks:
            raise ValueError('Duplicate native save block')
        blocks[name] = (version, payload)
        pos += size + (-size % 4)
        if pos > len(data):
            raise ValueError('Truncated native save padding')
    for name in ('player', 'misc', 'monster memory'):
        if name not in blocks or blocks[name][0] != 1:
            raise ValueError('Unsupported native character block version')
    p = Reader(blocks['player'][1])
    name, cause, history, race, shape, cls = [p.string() for _ in range(6)]
    p.take(3 + 6)  # suffix, hitdie, experience factor, age/height/weight
    stats = p.number('B')
    if not 1 <= stats <= 64:
        raise ValueError('Unsupported native stat count')
    p.take(stats * 8 + 6 + 4)  # four stat arrays, birth size/padding/gold
    p.string()  # body name
    slots = p.number('H')
    if slots > 128:
        raise ValueError('Unsupported native body size')
    for _ in range(slots):
        p.number('H'); p.string()
    p.take(4)
    gold, max_exp, exp = [p.number('I') for _ in range(3)]
    p.take(2)
    level, max_hp, hp = [p.number('h') for _ in range(3)]
    p.take(2)
    max_sp, sp = p.number('h'), p.number('h')
    p.take(2)
    max_level, max_depth = p.number('h'), p.number('h')
    p.take(2 + 2 + 2 + 2 + 1 + 1 + 2 + 2 + 2)
    timed = p.number('B'); p.take(2 * timed)
    energy, resting = p.number('I'), p.number('I')
    m = Reader(blocks['misc'][1]); m.take(12)
    dead, turn = m.number('B'), m.number('i')
    lore = Reader(blocks['monster memory'][1]); lore.take(1)
    kills = {}
    while True:
        monster = lore.string()
        if monster == 'No more monsters':
            break
        count = lore.number('H'); lore.number('H')  # this-life kills, thefts
        if monster in kills:
            raise ValueError('Duplicate native monster-memory entry')
        kills[monster] = count
    return {'name':name, 'race':race, 'class':cls, 'level':level, 'max_level':max_level,
            'max_depth':max_depth, 'gold':gold, 'experience':exp, 'max_experience':max_exp,
            'hp':hp, 'max_hp':max_hp, 'sp':sp, 'max_sp':max_sp,
            'dead':bool(dead), 'turn':turn, 'energy':energy, 'resting_turns':resting,
            'recorded_kills':sum(kills.values()), 'kills_saturated':any(n >= 32767 for n in kills.values()),
            'kills_by_race':{k:v for k,v in kills.items() if v}}



def native_dump(bundle, snapshot, info, target):
    """Isolated frozen engine load/format operation; validates unchanged clocks/RNG."""
    if info['dead']:
        return None
    target.mkdir(parents=True, exist_ok=False)
    # The original input remains immutable; runner loads a private copy.
    record = json.loads((bundle / 'recording.json').read_text())
    if not (record.get('toolkit') or record.get('build_id', {}).get('overlay')):
        raise ValueError('Legacy engine lacks qualified no-advance formatter')
    env = {k: v for k, v in os.environ.items() if not k.startswith(('BORG_', 'ANGBAND_', 'LLVM_PROFILE'))}
    env.update(BORG_TOOLKIT_REPORT=str(target / 'character.txt'), LC_ALL='C.UTF-8', LANG='C.UTF-8', TZ='UTC')
    binary = bundle / 'runtime/game/angband'
    data = bundle / 'runtime/game/lib'
    user = target / 'user'; user.mkdir()
    copy = target / 'origin.save'; shutil.copy2(snapshot, copy)
    indices = []
    for filename, key in (('p_race.txt', 'race'), ('class.txt', 'class')):
        names = [line.split(':', 1)[1] for line in (data / 'gamedata' / filename).read_text().splitlines() if line.startswith('name:')]
        if info[key] not in names:
            raise ValueError('Saved race/class missing from frozen native data')
        indices.append(names.index(info[key]))
    dirs = []
    for name in ('user', 'save', 'archive', 'panic', 'scores'):
        (target / name).mkdir(exist_ok=True)
        dirs.append(f'-d{name}={target / name}')
    dirs += [f'-d{option}={data / name}' for option, name in (('gamedata','gamedata'), ('pref','customize'), ('screens','screens'), ('help','help'))]
    args = [str(binary), '-mborgh', *dirs, '--', '-seed', '1', '-race', str(indices[0]), '-class', str(indices[1]),
            '-keep-load-rng', '-load', str(copy), '-turns', '2147483647']
    result = subprocess.run(args, cwd=binary.parent, env=env, capture_output=True, text=True, timeout=20)
    (target / 'console.log').write_text(result.stdout + result.stderr)
    if result.returncode or not (target / 'character.txt').exists():
        raise ValueError('Native no-advance formatter failed; diagnostic output retained')
    proof = json.loads((target / 'character.txt.json').read_text())
    if proof != {'turn': info['turn'], 'energy': info['energy'], 'unchanged': True}:
        raise ValueError('Native formatting advanced time/energy/RNG')
    if sha(copy) != sha(snapshot):
        raise ValueError('Native formatter changed its origin')
    return target / 'character.txt'


def create(bundle, entry, directory=None, format_character=True):
    """Reports are derived separately; never add files to a hashed raw bundle."""
    import borg_runtime as runtime
    import borg_gui
    bundle = Path(bundle).resolve()
    directory = Path(directory or bundle.parent.parent / 'reports').resolve()
    directory.mkdir(parents=True, exist_ok=True)
    manifest = json.loads((bundle / 'recording.json').read_text())
    backend = borg_gui if manifest.get('schema') == borg_gui.SCHEMA else runtime
    backend.validate(bundle, require_complete=False)
    final = entry.get('final') or manifest.get('summary') or {}
    if manifest.get('schema') == borg_gui.SCHEMA:
        from borg_replay_viewer import Frames
        movie = Frames(bundle)
        frame = movie.get(len(movie.index)-1)[0]; movie.close()
        final = {**final, 'game_turns': frame['turn'], 'clevel': frame['level'], 'max_clevel': frame.get('max_level'),
                 'max_depth': frame.get('max_depth'), 'borg_decisions': frame.get('borg_decisions')}
    process = json.loads((bundle / 'run/process.json').read_text()) if (bundle / 'run/process.json').exists() else {}
    report = {'schema': 'borg-rs/run-report@1', 'character': entry.get('character') or manifest.get('character'),
        'outcome': entry.get('outcome', manifest.get('status')), 'cause': entry.get('cause'), 'operational_error': entry.get('error'),
        'seed': manifest.get('profile', {}).get('seed') if not manifest.get('origin') else None,
        'requested_seed': manifest.get('profile', {}).get('seed'),
        'configuration_sha256': files_hash(manifest, 'run/user/borg.txt'), 'source': manifest.get('build_id'),
        'tools': manifest.get('collection_context', {}).get('tools_sha256'),
        'engine_ticks': final.get('game_turns'), 'level': final.get('clevel'), 'max_level': final.get('max_clevel'),
        'max_depth': final.get('max_depth'), 'decisions': final.get('borg_decisions'),
        'wall_seconds': process.get('elapsed_seconds', manifest.get('duration_seconds')),
        'snapshot': None, 'recorded_kills': None, 'experience': None, 'gold': None, 'resting_turns': None}
    files = manifest.get('files_sha256', {})
    snapshots = []
    for name in files:
        if name in ('run/final.save', 'final.save') or name.startswith('run/checkpoints/') and name.endswith('.save'):
            path = bundle / name
            try:
                if sha(path) != files[name]:
                    raise ValueError('Snapshot hash differs')
                info = saved_character(path)
                if report['engine_ticks'] is not None and info['turn'] > report['engine_ticks']:
                    raise ValueError('Snapshot is newer than recorded outcome')
                snapshots.append((info['turn'], path, info))
            except (OSError, ValueError, struct.error) as error:
                report.setdefault('snapshot_errors', {})[name] = str(error)
    if snapshots:
        _, snapshot, info = max(snapshots, key=lambda row: row[0])
        report['snapshot'] = {'path': str(snapshot), 'sha256': sha(snapshot), 'turn': info['turn'],
                              'stale': report['engine_ticks'] is not None and info['turn'] != report['engine_ticks']}
        for key in ('recorded_kills', 'experience', 'gold', 'resting_turns', 'level', 'max_level', 'max_depth'):
            report[key] = info[key]
        report['kills_scope'] = 'native this-life pkills; unseen non-unique kills may be absent'
        if format_character and not info['dead']:
            try:
                target = directory / ('native-' + sha(snapshot)[:16])
                cached = target / 'receipt.json'
                identity = {'snapshot': sha(snapshot), 'engine': sha(bundle / 'runtime/game/angband'),
                            'formatter': sha(Path(__file__))}
                if cached.exists():
                    receipt = json.loads(cached.read_text())
                    if receipt['identity'] != identity or receipt['dump_sha256'] != sha(target / 'character.txt'):
                        raise ValueError('Native report cache changed')
                    dump = target / 'character.txt'
                else:
                    if target.exists():
                        target = directory / (target.name + '-retry-' + str(time.time_ns()))
                    dump = native_dump(bundle, snapshot, info, target)
                    (target / 'receipt.json').write_text(json.dumps({'identity': identity, 'dump_sha256': sha(dump)}, indent=2)+'\n')
                report['native_character_dump'] = str(dump)
            except (OSError, ValueError, subprocess.SubprocessError) as error:
                report['character_dump_error'] = str(error)
    def value(key):
        return 'unavailable' if report.get(key) is None else str(report[key])
    text = f"# {report['character']}\n\nOutcome: {report['outcome']}.\n\n| Progress | Value |\n| --- | --- |\n"
    for title, key in [('Current level', 'level'), ('Highest level', 'max_level'), ('Maximum dungeon depth', 'max_depth'),
        ('Recorded this-life kills', 'recorded_kills'), ('XP', 'experience'), ('Gold', 'gold'), ('Engine ticks', 'engine_ticks'),
        ('Borg decisions', 'decisions'), ('Rest counter (player rest turns)', 'resting_turns'), ('Wall seconds', 'wall_seconds')]:
        text += f'| {title} | {value(key)} |\n'
    text += f"\nSeed: {value('seed')}. Source/build/tool identity: summary.json.\n"
    if report.get('operational_error'):
        text += '\nOperational error: ' + report['operational_error'] + '\n'
    if report['outcome'] == 'death':
        text += f"\nNative cause: {report.get('cause') or 'unavailable'}.\n"
    if report['snapshot']:
        text += f"\nNative counters at tick {report['snapshot']['turn']}. "
        if report['snapshot']['stale']:
            text += 'This snapshot predates the final outcome. '
        text += report['kills_scope'] + '.\n'
    if report.get('native_character_dump'):
        text += '\n[Native character sheet](<' + os.path.relpath(report['native_character_dump'], directory) + '>)\n'
    if report.get('character_dump_error'):
        text += '\nNative sheet unavailable: ' + report['character_dump_error'] + '\n'
    text += '\nLimited living runs are not victories or survival estimates. Missing counters are unavailable.\n'
    (directory / 'summary.md').write_text(text)
    (directory / 'summary.json').write_text(json.dumps(report, indent=2, sort_keys=True)+'\n')
    return directory / 'summary.md', report


def collection_summary(directory, plan):
    text = '# Collected attempts\n\n| Seed | Level | Max depth | Kills | Outcome | Report |\n| --- | --- | --- | --- | --- | --- |\n'
    for entry in plan.get('runs', []):
        path = Path(entry['report']).with_suffix('.json') if entry.get('report') else Path(entry['directory']) / 'reports/summary.json'
        report = json.loads(path.read_text()) if path.exists() else {}
        value = lambda key: str(report[key]) if report.get(key) is not None else 'unavailable'
        link = os.path.relpath(path.with_suffix('.md'), directory)
        text += f"| {entry['seed']} | {value('level')} | {value('max_depth')} | {value('recorded_kills')} | {entry['outcome']} | [Report](<{link}>) |\n"
    text += '\nIndependent attempts only. Replays and manual alternatives are excluded.\n\nDeaths:\n\n'
    text += ''.join(f"- Seed {e['seed']}: {e.get('dossier', 'export unavailable')}\n" for e in plan.get('runs', []) if e['outcome'] == 'death') or '- None.\n'
    path = directory / 'reports.md'; path.write_text(text)
    return path


def files_hash(manifest, name):
    return manifest.get('files_sha256', {}).get(name)
