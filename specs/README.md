# Specs

Each file in this directory (other than this README and `_template.md`) is one feature spec. Specs are the source of truth for *what the code must do*. The code in `vibe-board/` exists to satisfy them.

## Filename convention

`NNNN-<slug>.md` — zero-padded sequential ID, short slug. Example: `0001-create-task.md`.

## Lifecycle

A spec moves through three statuses, tracked in its frontmatter:

| Status | Meaning |
|---|---|
| `draft` | Written but no tests or code exist yet. Safe to edit freely. |
| `in-progress` | Acceptance tests exist (and may be failing). Implementation is underway. Changes to acceptance criteria at this stage should be deliberate and explained in commit messages. |
| `done` | All acceptance tests pass. Further changes require a *new* spec (or a deliberate amendment with a rationale). |
| `superseded` | Replaced by a newer spec listed in its `supersedes:` field. Tests and code may still exist but are owned by the superseding spec. |

## The loop

1. Copy `_template.md` to `NNNN-<slug>.md`. Fill in `Why`, `User-visible behavior`, `Acceptance criteria`, `Out of scope`. Status: `draft`.
2. Create `vibe-board/tests/spec_NNNN_<slug>.rs`. Write one test per acceptance criterion. They must fail (or fail to compile — that counts). Flip status to `in-progress`.
3. Implement in `vibe-board/src/` until every test passes.
4. Flip status to `done`. Commit.

## Rules

- **Every acceptance criterion maps to at least one test assertion.** Coverage is checked by eye during review — if a criterion has no test, the spec is not done.
- **The `Traceability` section must name the test file and the implementation entry points.** This is how future changes find what to touch.
- **Out of scope is enforced.** Anything listed there is off-limits for this spec. It needs its own spec to be built.

## Conflicts

Two specs **conflict** when their `Traceability` sections claim the same implementation file or the same function/entry point. Conflicts must be resolved before a spec moves to `in-progress`.

**Detection.** Run `python3 scripts/spec_index.py` before authoring a new spec and again before flipping any spec to `in-progress`. The script parses every spec's `Traceability` section, builds a file→spec and entry-point→spec index, and exits non-zero if any live spec overlaps with another.

**Resolution rule.** The newer spec (higher `id`) wins. The author of the new spec MUST do one of:

1. Narrow the new spec's `Traceability` so it no longer overlaps, or
2. Add the clashing spec ID(s) to the new spec's `supersedes:` frontmatter list and flip those older specs to `status: superseded` in the same commit.

There is no "both specs own this file" state.

**`supersedes:` vs `related:`.** Both are optional frontmatter fields containing lists of spec IDs.

- `supersedes:` is a *hard replacement*. Listed specs are excluded from conflict checks against this spec, and they should be flipped to `status: superseded`.
- `related:` is a *soft cross-reference*. Listed specs share context but are not replaced; conflict warnings are NOT suppressed.
