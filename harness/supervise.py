#!/usr/bin/env python3
"""Run one engine process once; retain failure and checkpoint receipts.

This is an execution wrapper, not an oracle. It never retries, changes a
scenario, invents an engine summary, or labels partial logs as valid evidence.
Only the exact child PID it created can receive a signal from this process.
"""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import re
import signal
import subprocess
import sys
import time


def atomic_json(path, value):
    tmp = path.with_name(path.name + ".tmp")
    with tmp.open("w") as f:
        json.dump(value, f, indent=2, sort_keys=True)
        f.write("\n")
        f.flush()
        os.fsync(f.fileno())
    tmp.replace(path)


def inventory(directory, status):
    """Index only complete, successful captures, even when the game crashed."""
    index = directory / "checkpoints" / "index.jsonl"
    if not index.exists():
        return [], []
    captures, errors, seen = [], [], set()
    for number, line in enumerate(index.read_bytes().splitlines(keepends=True), 1):
        try:
            if not line.endswith(b"\n"):
                raise ValueError("incomplete index line")
            row = json.loads(line)
            name = row["file"]
            if not re.fullmatch(r"checkpoint-[0-9]+-t[0-9]+\.save", name):
                raise ValueError("invalid checkpoint filename")
            if name in seen:
                raise ValueError("duplicate checkpoint filename")
            seen.add(name)
            for key in ("due_turn", "game_turn", "keys", "seed", "race_idx",
                        "class_idx", "depth", "max_depth", "clevel"):
                if type(row[key]) is not int or row[key] < 0:
                    raise ValueError("invalid " + key)
            if row["game_turn"] < row["due_turn"]:
                raise ValueError("capture precedes scheduled threshold")
            path = index.parent / name
            if path.is_symlink() or not path.is_file() or not path.stat().st_size:
                raise ValueError("missing/empty/nonregular checkpoint")
            digest = hashlib.sha256()
            with path.open("rb") as f:
                for block in iter(lambda: f.read(1024 * 1024), b""):
                    digest.update(block)
            row.update(save_sha256=digest.hexdigest(), size_bytes=path.stat().st_size)
            captures.append(row)
        except (KeyError, OSError, TypeError, ValueError) as exc:
            errors.append({"line": number, "error": str(exc)})
    atomic_json(directory / "checkpoints.json", {
        "source_process_status": status, "replay": "fresh_borg_not_continuation",
        "captures": captures, "errors": errors,
    })
    return captures, errors


def tail(path, limit=16384):
    if not path.exists():
        return ""
    with path.open("rb") as f:
        f.seek(max(0, path.stat().st_size - limit))
        return f.read().decode("utf-8", errors="replace")


def file_hash(path):
    digest = hashlib.sha256()
    with path.open("rb") as f:
        for block in iter(lambda: f.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()



def complete_tail(path):
    """Return the last whole flushed row, never a partial UTF-8/JSON suffix."""
    try:
        with path.open('rb') as stream:
            stream.seek(0, os.SEEK_END)
            end = stream.tell()
            span = 65536
            while True:
                start = max(0, end - span)
                stream.seek(start)
                rows = stream.read().split(b'\n')
                if len(rows) >= 3 or start == 0:
                    return rows[-2] if len(rows) >= 2 else None
                if span >= 64 * 1024 * 1024:
                    raise ValueError('Live native row exceeds the declared resource bound')
                span *= 2
    except FileNotFoundError:
        return None


def observed_turn(directory):
    """Actual flushed native clock when available; legacy key-boundary fallback."""
    native = directory / 'native/frames.jsonl'
    if native.exists():
        line = complete_tail(native)
        if line is None:
            return None
        row = json.loads(line.decode('utf-8'))
        if type(row.get('turn')) is not int or row['turn'] < 0:
            raise ValueError('Invalid live native clock row')
        return row['turn']
    line = complete_tail(directory / 'keys.log')
    if line is not None:
        fields = line.split()
        if len(fields) == 4 and fields[0].isdigit():
            return int(fields[0])
    return None


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--receipt", required=True, type=Path)
    parser.add_argument("--diagnostics", required=True, type=Path)
    parser.add_argument("--timeout", type=float, default=0)
    parser.add_argument("--stall-timeout", type=float, default=0)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args(argv)
    command = args.command[1:] if args.command[:1] == ["--"] else args.command
    if not command or not all(math.isfinite(value) and value >= 0 for value in (args.timeout, args.stall_timeout)):
        parser.error("supply a command and a finite timeout >= 0 (0 disables it)")
    started = time.monotonic()
    record = {"status": "starting", "command": command, "timeout_seconds": args.timeout,
              "started_unix": time.time(), "supervisor_pid": os.getpid(),
              "stall_timeout_seconds": args.stall_timeout, "progress_scope": "latest complete flushed native-frame world tick; legacy key-log fallback"}
    child = None
    reserved = False
    interrupted = []

    def forward(signum, _frame):
        if not interrupted:
            interrupted.append(signum)
        if child is not None and child.poll() is None:
            child.send_signal(signum)

    for sig in (signal.SIGINT, signal.SIGTERM):
        signal.signal(sig, forward)
    try:
        # Reserve this result before launching. A new invocation must not
        # erase an earlier crash or a still-running process's receipt.
        fd = os.open(args.receipt, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o644)
        os.close(fd)
        reserved = True
        atomic_json(args.receipt, record)
        record["executable_sha256_before"] = file_hash(Path(command[0]))
        env = os.environ.copy()
        env.setdefault("RUST_BACKTRACE", "1")
        child = subprocess.Popen(command, env=env)
        record.update(status="running", child_pid=child.pid)
        atomic_json(args.receipt, record)
        timed_out, stalled, grace = False, False, None
        last_turn, last_progress = None, time.monotonic()
        while child.poll() is None:
            now = time.monotonic()
            current_turn = observed_turn(args.diagnostics) if args.stall_timeout else None
            if current_turn is not None and (last_turn is None or current_turn > last_turn):
                last_turn, last_progress = current_turn, now
            if args.stall_timeout and now - last_progress >= args.stall_timeout and grace is None:
                stalled = True
                child.terminate()
                grace = now + 2
            if args.timeout and now - started >= args.timeout and grace is None:
                timed_out = True
                child.terminate()
                grace = now + 2
            if interrupted and grace is None:
                grace = now + 2
            if grace is not None and now >= grace:
                child.kill()
                child.wait()
                break
            try:
                child.wait(timeout=0.1)
            except subprocess.TimeoutExpired:
                pass
        rc = child.returncode
        if stalled:
            status, exit_code = "stall", 125
        elif timed_out:
            status, exit_code = "timeout", 124
        elif interrupted:
            status, exit_code = "interrupted", 128 + interrupted[0]
        elif rc < 0:
            status, exit_code = "signal", 128 - rc
        elif rc:
            status, exit_code = "exit_failure", rc
        elif not (args.diagnostics / "summary.json").is_file():
            status, exit_code = "missing_summary", 1
        else:
            status, exit_code = "success", 0
        captures, errors = inventory(args.diagnostics, status)
        record["executable_sha256_after"] = file_hash(Path(command[0]))
        if record["executable_sha256_before"] != record["executable_sha256_after"]:
            status, exit_code = "binary_changed", 3
        if errors and not exit_code:
            status, exit_code = "checkpoint_error", 3
        record.update(status=status, engine_returncode=rc, exit_code=exit_code,
                      signal=(-rc if rc < 0 else None), timed_out=timed_out, stalled=stalled,
                      last_observed_turn=last_turn,
                      interrupted_signal=(interrupted[0] if interrupted else None),
                      checkpoint_count=len(captures), checkpoint_errors=errors,
                      elapsed_seconds=time.monotonic() - started,
                      stderr_tail=tail(args.diagnostics / "stderr.log"))
        atomic_json(args.receipt, record)
        return exit_code
    except (OSError, ValueError) as exc:
        if child is not None and child.poll() is None:
            child.kill()
            child.wait()
        record.update(status="supervisor_error", exit_code=3, error=str(exc),
                      engine_returncode=(child.returncode if child else None),
                      elapsed_seconds=time.monotonic() - started)
        if reserved:
            try:
                atomic_json(args.receipt, record)
            except OSError:
                pass
        print("supervisor: " + str(exc), file=sys.stderr)
        return 3


if __name__ == "__main__":
    sys.exit(main())
