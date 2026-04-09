---
id: 0008
title: Swimlanes, desktop notifications, and input handoff
status: draft
related: [2, 4, 5]
---

## Why

The user needs a single pane that shows every agent's state at a glance, surfaces anyone who needs them, and fires an OS-level notification the moment an agent blocks on input. Without this, parallelism is invisible and blocking agents stall silently until the user happens to look at the board. Spec 0001's single-column Todo board is intentionally untouched — this spec introduces a *new* workspace board, not a rewrite of it.

## User-visible behavior

Inside a workspace, the user opens a board at `/workspaces/:id/board` that shows five swimlanes — `todo`, `in-progress`, `review`, `needs-input`, `done` — and every issue in that workspace appears in exactly one lane based on its status. Whenever an agent run transitions to `needs_input`, an OS desktop notification fires identifying the workspace, the issue, and the agent run. A cross-workspace view at `/needs-me` lists every agent currently in `needs_input` across every active workspace, ordered by how long they've been blocked.

## Acceptance criteria

- GIVEN an active workspace `W` with issues in statuses `todo`, `in-progress`, `review`, `needs_input`, and `done`, WHEN I `GET /workspaces/W/board`, THEN the response is 200 and contains five swimlane sections with the labels above, and every issue for `W` appears in exactly one swimlane matching its status.
- GIVEN an active workspace `W` with zero issues, WHEN I `GET /workspaces/W/board`, THEN the response is 200 and all five swimlanes render empty.
- GIVEN an agent run, WHEN `on_status_change(run, needs_input)` is invoked, THEN `Notifier::notify` is called exactly once with a payload naming the workspace id, the issue id, and the agent run id, AND the call records an entry in `notification_log`.
- GIVEN the test configuration, WHEN `Notifier` is resolved, THEN the concrete implementation is a `SpyNotifier` that records calls in memory and exposes them for assertions; the production implementation is `DesktopNotifier`.
- GIVEN active workspaces `W1` and `W2` with agent runs `A1` (in `needs_input` for 2 minutes) and `A2` (in `needs_input` for 10 minutes) respectively, WHEN I `GET /needs-me`, THEN the response lists both agent runs with `A2` ordered before `A1`.
- GIVEN an agent run that transitions from `needs_input` back to `running`, WHEN the status change is observed, THEN `Notifier::notify` is NOT called (notifications fire on entry into `needs_input` only).

## Out of scope

- Changing spec 0001's Todo board at `/` — that page stays as-is. This spec introduces a separate workspace board route.
- Notifications over channels other than the OS notifier (no Telegram, no email).
- Per-user notification preferences (single-user tool).
- Drag-and-drop between swimlanes in the UI.

## Traceability

- Test file: `vibe-board/tests/spec_0008_swimlanes_and_notifications.rs`
- Implementation entry points:
  - `vibe-board/migrations/0008_notifications.sql` — `notification_log` table
  - `vibe-board/src/swimlanes.rs` — `Swimlane`, `WorkspaceBoard`, `build_workspace_board`, `on_status_change`
  - `vibe-board/src/notify.rs` — `Notifier` trait, `SpyNotifier`, `DesktopNotifier`, `NotificationPayload`
  - `vibe-board/src/swimlane_routes.rs` — `get_workspace_board`, `get_needs_me`, `swimlane_router`
  - `vibe-board/templates/workspace_board.html` — swimlane layout
  - `vibe-board/templates/needs_me.html` — cross-workspace blocked list
