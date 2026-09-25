---
id: 012
title: How does search work?
labels: [wayfinder:grilling]
parent: 000
blocked_by: []
assignee: emil.bohleber
status: closed
---

## Question

Decide search semantics: incremental vs. submit, plain/regex/SHA-prefix matching over message/author/SHA, scope (current pane vs. ref set vs. whole DAG cache), whether an index is needed at 200k commits, and how hits are shown/navigated when commit info is hidden by default (key, centred box, highlight, next/prev).


## Resolution

Ruled out of scope (2026-09-25): search is cut from the feature list for now; user unsure it's needed in this viewer. Not decided on the route - see map's Out of scope.

