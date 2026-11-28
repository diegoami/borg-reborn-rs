#!/usr/bin/env python3
"""Fail a pull request into `main` that belongs on a track branch (#1382).

Track P work goes to `track-p` and track R work to `feature/track-r`
(AGENTS.md); a P task PR merged into `main` by mistake (#1370) had to be
carried back by a sync PR. A PR into `main` is flagged when it changes
`borg-core/`, or when the issue it closes is labelled `track:P` or
`track:R`, unless its head is itself a track branch (a promotion or a
sync) or it carries the `base-main-ok` label.

    check-pr-base.py PR_NUMBER      # reads the PR with the gh CLI; exit 1 when flagged
"""
import json
import subprocess
import sys

TRACK_BRANCHES = {'track-p': 'track-p', 'feature/track-r': 'feature/track-r'}
TRACK_LABEL_BASE = {'track:P': 'track-p', 'track:R': 'feature/track-r'}
CORE_PREFIX = 'borg-core/'
ESCAPE_LABEL = 'base-main-ok'


def violation(base, head, pr_labels, files, issue_labels):
    """The message that explains what is wrong, or None when the PR may go to `base`."""
    if base != 'main' or head in TRACK_BRANCHES or ESCAPE_LABEL in pr_labels:
        return None
    wanted = sorted({TRACK_LABEL_BASE[l] for l in issue_labels if l in TRACK_LABEL_BASE})
    reasons = []
    if wanted:
        reasons.append('the issue it closes is labelled ' + ', '.join(
            l for l in sorted(issue_labels) if l in TRACK_LABEL_BASE))
    if any(f.startswith(CORE_PREFIX) for f in files):
        reasons.append('it changes ' + CORE_PREFIX)
        wanted = wanted or ['track-p', 'feature/track-r']
    if not reasons:
        return None
    return (f'This PR targets main, but {" and ".join(reasons)}. '
            f'Retarget it to {" or ".join(wanted)} (gh pr edit <n> --base <branch>), '
            f'or add the `{ESCAPE_LABEL}` label if main is intended.')


def gh(*args):
    return subprocess.run(['gh', *args], check=True, capture_output=True, text=True).stdout


def main(argv):
    pr = argv[1]
    info = json.loads(gh('pr', 'view', pr, '--json',
                         'baseRefName,headRefName,labels,closingIssuesReferences'))
    base, head = info['baseRefName'], info['headRefName']
    labels = [l['name'] for l in info['labels']]
    if base != 'main':
        return 0
    files = gh('api', '--paginate', f'repos/{{owner}}/{{repo}}/pulls/{pr}/files',
               '--jq', '.[].filename').split()
    issue_labels = set()
    for ref in info['closingIssuesReferences']:
        issue = json.loads(gh('issue', 'view', str(ref['number']), '--json', 'labels'))
        issue_labels.update(l['name'] for l in issue['labels'])
    message = violation(base, head, labels, files, issue_labels)
    if message:
        print(f'::error::{message}')
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv))
