---
id: 0001
title: Create a task and see it on the board
status: done
---

## Why

This is the first vertical slice of `vibe-board`. Its purpose is to prove the spec-driven loop works end-to-end on this stack (axum + sqlx + askama + htmx) by exercising DB migration, HTTP routing, template rendering, and acceptance tests in a single minimal feature.

## User-visible behavior

A user opens `http://localhost:3000/` in a browser and sees a page titled "vibe-board" with a single column labeled "Todo". Below the column is a form with a text input for a task title and a submit button. Submitting the form with a non-empty title adds a card with that title to the Todo column. Submitting an empty title is rejected.

## Acceptance criteria

- GIVEN an empty database, WHEN I `GET /`, THEN the response status is 200 and the body contains a column labeled "Todo" with zero task cards.
- GIVEN an empty database, WHEN I `POST /tasks` with form body `title=Write spec`, THEN the response is successful, a row exists in the `tasks` table with `title = "Write spec"` and `status = "todo"`, and the response body contains a card with the text "Write spec" inside the Todo column.
- GIVEN a task titled "Write spec" already exists in the database, WHEN I `GET /`, THEN the response body contains the text "Write spec" inside the Todo column.
- GIVEN an empty database, WHEN I `POST /tasks` with form body `title=` (empty), THEN the response status is 400 and no row is inserted into the `tasks` table.

## Out of scope

- Moving cards between columns (no drag-drop, no status transitions)
- Editing or deleting tasks
- Columns other than "Todo"
- Styling beyond basic legibility
- Authentication, multi-user, persistence across machines
- Agent / Claude Code integration
- Any JavaScript beyond what htmx provides out of the box

## Traceability

- Test file: `vibe-board/tests/spec_0001_create_task.rs`
- Implementation entry points:
  - `vibe-board/migrations/0001_init.sql` — `tasks` table
  - `vibe-board/src/models.rs` — `Task`, `TaskStatus`
  - `vibe-board/src/db.rs` — `init_pool`, `insert_task`, `list_tasks_by_status`
  - `vibe-board/src/routes.rs` — `get_board`, `post_task`
  - `vibe-board/src/main.rs` — `build_router`
  - `vibe-board/templates/board.html` — board layout
