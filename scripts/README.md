# scripts

Workflow tooling. Run from the repo root.

## `spec_index.py`

Parses every spec's `## Traceability` section and builds an index of which implementation files and entry points each spec claims. Exits non-zero if two live specs claim the same file or entry point.

```bash
python3 scripts/spec_index.py            # check all live specs
python3 scripts/spec_index.py --spec 1   # restrict to one spec id
python3 scripts/spec_index.py --json     # machine-readable output
```

Run this before authoring a new spec and again before flipping any spec to `in-progress`. Conflict resolution rules live in [`specs/README.md`](../specs/README.md#conflicts).
