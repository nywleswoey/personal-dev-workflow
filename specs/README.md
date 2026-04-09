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

## The loop

1. Copy `_template.md` to `NNNN-<slug>.md`. Fill in `Why`, `User-visible behavior`, `Acceptance criteria`, `Out of scope`. Status: `draft`.
2. Create `vibe-board/tests/spec_NNNN_<slug>.rs`. Write one test per acceptance criterion. They must fail (or fail to compile — that counts). Flip status to `in-progress`.
3. Implement in `vibe-board/src/` until every test passes.
4. Flip status to `done`. Commit.

## Rules

- **Every acceptance criterion maps to at least one test assertion.** Coverage is checked by eye during review — if a criterion has no test, the spec is not done.
- **The `Traceability` section must name the test file and the implementation entry points.** This is how future changes find what to touch.
- **Out of scope is enforced.** Anything listed there is off-limits for this spec. It needs its own spec to be built.
