---
id: NNNN
title: <short title>
status: draft
---

## Why

<1–3 sentences: the problem or motivation. What changes for the user or developer if this ships?>

## User-visible behavior

<Plain-English description of what the user sees or can do after this spec is implemented.>

## Acceptance criteria

Each bullet is testable. Every bullet must map to at least one assertion in the test file below.

- GIVEN ... WHEN ... THEN ...
- GIVEN ... WHEN ... THEN ...

## Out of scope

<Explicit non-goals. Anything listed here is off-limits for this spec — it needs its own spec to be built.>

## Traceability

- Test file: `vibe-board/tests/spec_NNNN_<slug>.rs`
- Implementation entry points:
  - `vibe-board/src/<file>.rs` — `<function or struct>`
