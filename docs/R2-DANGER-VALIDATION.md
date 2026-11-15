# R2 danger slice validation (#281)

This slice ports danger only. Caution, escape and recovery remain #209;
R2 is not complete. Runtime comparator remains borg-rs v12 e6d13fc.

Off/check/switch smoke passes 3/3 in keys, messages and state. All three
comparison points have positive coverage and zero mismatches. Full-suite
and deliberate-disagreement evidence is added before the task merges.

Shared projection, spell-legality, blow classification and engine facts are
outside the port; agreeing danger results do not independently validate them.
No survival or performance improvement is claimed. No sealed suites opened.
