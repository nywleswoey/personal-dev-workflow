# Project rules for AI assistants

This repository practices **Spec-Driven Development**. These rules are non-negotiable.

## The loop

1. **Spec first.** No code change happens without a matching file in `specs/`. If one doesn't exist, create it (copy `specs/_template.md`) before touching any `.rs` file.
2. **Acceptance criteria are Given/When/Then.** Each criterion must be testable and must map to at least one assertion in a test file named `vibe-board/tests/spec_NNNN_<slug>.rs`.
3. **Tests before implementation.** Write the acceptance tests against the spec. They must fail. Only then write production code until they pass.
4. **Traceability is mandatory.** Every spec file names its test file and its implementation entry points in a `## Traceability` section. Every test module starts with a comment linking back to its spec.
5. **Mark status.** Specs carry `status: draft | in-progress | done` in frontmatter. Flip it as you move through the loop.

## What NOT to do

- Don't add features beyond what the active spec's acceptance criteria require.
- Don't edit code in `vibe-board/src/` without an `in-progress` spec.
- Don't delete or weaken an acceptance criterion to make a test pass. Update the spec deliberately (and explain why in the commit message) if the behavior genuinely needs to change.
- Don't add speculative abstractions, config knobs, or "for later" hooks. If it's not in the spec, it doesn't exist.

## Stack conventions

- Rust, single crate at `vibe-board/`
- axum + sqlx (SQLite) + askama + htmx
- SQLite file path comes from `DATABASE_URL`; tests use per-test temp files
