#!/usr/bin/env python3
"""Runner contention between borg-reborn-rs's CI and borg-zoo's runs (#1437).

Both repositories share the owner's 20 concurrent Actions job slots. A job
waits for a runner from `created_at` to `started_at`; this reads those times
for borg-reborn-rs's `ci.yml` runs and borg-zoo's `compare.yml` runs over the last --days days (or a --since/--until window), and
reports for each side the wait (median, 90th percentile, longest), and how the
zoo's waits differ when a borg-reborn-rs CI job was queued or running at the
moment the zoo job was created.

    python3 scripts/ci-contention.py [--days 7 | --since 2026-11-26 [--until 2026-11-27]] [--json]

Read only; needs the gh CLI (public repositories). Decision rule (#1437):
if zoo jobs created while our CI ran wait clearly longer (say a median above
a minute, or a 90th percentile above five) than the zoo's other jobs, put
push runs on track-p into one cancelling concurrency group (the change is
written in docs/handover/2026-11-28-1437-ci-contention.md); otherwise leave
ci.yml alone.
"""
import argparse
import datetime
import json
import statistics
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor

REBORN = ('diegoami/borg-reborn-rs', 'ci.yml')
ZOO = ('diegoami/borg-zoo', 'compare.yml')


def parse_time(text):
    return datetime.datetime.fromisoformat(text.replace('Z', '+00:00')) if text else None


def gh(path):
    done = subprocess.run(['gh', 'api', path], capture_output=True, text=True)
    if done.returncode:
        raise SystemExit(f'gh api {path}: {done.stderr.strip()}')
    return json.loads(done.stdout)


def runs_since(repo, workflow, since, until=None):
    out, page = [], 1
    window = f'%3E%3D{since.strftime("%Y-%m-%d")}' if until is None else \
        f'{since.strftime("%Y-%m-%d")}..{until.strftime("%Y-%m-%d")}'
    while True:
        data = gh(f'repos/{repo}/actions/workflows/{workflow}/runs?per_page=100&page={page}&created={window}')
        batch = data.get('workflow_runs', [])
        out += batch
        if len(batch) < 100:
            return out
        page += 1


def jobs_of(repo, run_id):
    out, page = [], 1
    while True:
        data = gh(f'repos/{repo}/actions/runs/{run_id}/jobs?per_page=100&page={page}')
        batch = data.get('jobs', [])
        out += batch
        if len(batch) < 100:
            return out
        page += 1


def collect(repo, workflow, since, until=None):
    """[{created, started, completed, name, run, branch}] of the jobs that ran on a runner."""
    jobs = []
    runs = runs_since(repo, workflow, since, until)
    with ThreadPoolExecutor(8) as pool:
        listed = list(pool.map(lambda run: (run, jobs_of(repo, run['id'])), runs))
    for run, run_jobs in listed:
        for job in run_jobs:
            created, started = parse_time(job.get('created_at')), parse_time(job.get('started_at'))
            if job.get('conclusion') == 'skipped' or not created or not started or not job.get('runner_name'):
                continue  # never took a runner
            jobs.append({'created': created, 'started': started, 'completed': parse_time(job.get('completed_at')),
                         'name': job['name'], 'run': run['id'], 'branch': run.get('head_branch')})
    return jobs


def wait(job):
    return max(0.0, (job['started'] - job['created']).total_seconds())


def busy_at(moment, jobs):
    """How many of `jobs` were queued or running at `moment`."""
    return sum(1 for j in jobs if j['created'] <= moment and (j['completed'] is None or moment < j['completed']))


def running_at(moment, jobs):
    """How many of `jobs` held a runner at `moment`."""
    return sum(1 for j in jobs if j['started'] <= moment and (j['completed'] is None or moment < j['completed']))


def attribute_delays(reborn, zoo, slots=20, threshold=30.0):
    """Why the zoo's delayed jobs waited, from the runners each side held when the job was created:
    'self' the zoo's own running jobs filled every slot; 'reborn' the zoo alone would have fitted but
    its jobs plus borg-reborn-rs's did not; 'other' neither side filled them (other workflows or
    repositories of the account hold the rest, which this does not see)."""
    out = {'self': [], 'reborn': [], 'other': []}
    for job in zoo:
        if wait(job) <= threshold:
            continue
        mine, ours = running_at(job['created'], zoo), running_at(job['created'], reborn)
        out['self' if mine >= slots else 'reborn' if mine + ours >= slots else 'other'].append(wait(job))
    return {k: {'jobs': len(v), 'total_wait_h': round(sum(v) / 3600, 2)} for k, v in out.items()}


def percentile(values, q):
    values = sorted(values)
    if not values:
        return None
    return values[min(len(values) - 1, int(round(q * (len(values) - 1))))]


def describe(waits):
    if not waits:
        return {'jobs': 0}
    return {'jobs': len(waits), 'median_s': round(statistics.median(waits), 1),
            'p90_s': round(percentile(waits, 0.9), 1), 'max_s': round(max(waits), 1)}


def contention(reborn, zoo):
    """The waits of each side, and of the zoo's jobs split by whether a borg-reborn-rs job was in flight."""
    during = [wait(j) for j in zoo if busy_at(j['created'], reborn)]
    apart = [wait(j) for j in zoo if not busy_at(j['created'], reborn)]
    return {'reborn_ci': describe([wait(j) for j in reborn]), 'zoo': describe([wait(j) for j in zoo]),
            'zoo_while_reborn_in_flight': describe(during), 'zoo_otherwise': describe(apart),
            'peak_reborn_jobs_in_flight': max((busy_at(j['created'], reborn) for j in reborn), default=0),
            'zoo_delays_over_30s_by_cause': attribute_delays(reborn, zoo)}


def render(result, window):
    lines = [f'Runner waits over {window} (created_at to started_at)', '']
    for key, label in (('reborn_ci', 'borg-reborn-rs ci.yml jobs'), ('zoo', 'borg-zoo compare.yml jobs'),
                       ('zoo_while_reborn_in_flight', '  zoo jobs created while a borg-reborn-rs CI job was in flight'),
                       ('zoo_otherwise', '  zoo jobs created otherwise')):
        d = result[key]
        lines.append(f'{label}: ' + ('none' if not d['jobs'] else
                                     f'{d["jobs"]} jobs, median {d["median_s"]} s, p90 {d["p90_s"]} s, max {d["max_s"]} s'))
    lines.append(f'most borg-reborn-rs CI jobs in flight at the creation of one of ours: {result["peak_reborn_jobs_in_flight"]}')
    lines.append('zoo jobs that waited over 30 s, by what held the 20 slots when they were created:')
    for cause, d in result['zoo_delays_over_30s_by_cause'].items():
        lines.append(f'  {cause}: {d["jobs"]} jobs, {d["total_wait_h"]} job-hours waited')
    return '\n'.join(lines)


def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument('--days', type=int, default=7)
    p.add_argument('--since', help='first day, YYYY-MM-DD (instead of --days)')
    p.add_argument('--until', help='last day, YYYY-MM-DD (with --since)')
    p.add_argument('--json', action='store_true')
    args = p.parse_args(argv)
    day = lambda text: datetime.datetime.fromisoformat(text).replace(tzinfo=datetime.timezone.utc)  # noqa: E731
    if args.since:
        since, until = day(args.since), day(args.until or args.since)
        label = f'{args.since}..{args.until or args.since}'
    else:
        since, until = datetime.datetime.now(datetime.timezone.utc) - datetime.timedelta(days=args.days), None
        label = f'the last {args.days} days'
    result = contention(collect(*REBORN, since, until), collect(*ZOO, since, until))
    print(json.dumps(result, indent=2) if args.json else render(result, label))
    return 0


if __name__ == '__main__':
    sys.exit(main())
