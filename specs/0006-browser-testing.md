---
id: 0006
title: Browser testing with agent-driven and human handoff
status: draft
related: [5]
---

## Why

Some acceptance criteria can only be verified in a real browser — rendered DOM, network, click behavior. The agent must be able to drive a browser session, and the user must be able to take over the *same* session to poke at the running app without restarting the agent or losing state. Without this, UI-sensitive criteria either go untested or force the user to shell out to a separate browser.

## User-visible behavior

When an agent run needs browser verification, it asks the browser bridge to launch a Playwright session; that session is tied to the agent run. The user can open `/agents/:id/browser` and see the session embedded, with a "Take control" button. While the user holds control, the agent is blocked from driving the browser and the agent run sits in `needs_input`. When the user clicks "Release", the agent resumes. When the agent run finishes or is cancelled, the session is torn down.

## Acceptance criteria

- GIVEN an agent run `A` in status `running`, WHEN `browser::launch(A)` is invoked, THEN a row is inserted in `browser_sessions` with `agent_run_id = A`, status `active`, controller `agent`, and a non-empty `session_id`.
- GIVEN an active browser session for agent run `A`, WHEN I `GET /agents/A/browser`, THEN the response is 200, names the session id, and includes a "Take control" action.
- GIVEN an active browser session controlled by `agent`, WHEN I `POST /agents/A/browser/take-control`, THEN `browser_sessions.controller` flips to `user`, the agent run transitions to `needs_input`, and subsequent calls to `browser::drive(A, ...)` return `BrowserError::UserHasControl` without sending commands to Playwright.
- GIVEN a browser session controlled by `user`, WHEN I `POST /agents/A/browser/release`, THEN `browser_sessions.controller` flips back to `agent`, the agent run transitions back to `running`, and `browser::drive(A, ...)` is permitted again.
- GIVEN an active browser session, WHEN the owning agent run reaches status `stopped` or `cancelled`, THEN `browser::teardown(A)` is called, the session row transitions to status `closed`, and the underlying Playwright session is disposed.

## Out of scope

- The concrete Playwright transport — `BrowserBackend` is a trait; tests use an in-memory fake. Production wiring to the Playwright MCP server is covered by the implementation of that trait, not by this spec.
- Recording or replaying sessions for later review.
- Multi-user concurrent control of a single session.
- Browser verification for human-only tests that have no agent run (launch a browser without an agent is not supported here).

## Traceability

- Test file: `vibe-board/tests/spec_0006_browser_testing.rs`
- Implementation entry points:
  - `vibe-board/migrations/0006_browser.sql` — `browser_sessions` table
  - `vibe-board/src/browser.rs` — `BrowserSession`, `Controller`, `BrowserBackend`, `BrowserError`, `launch`, `drive`, `take_control`, `release`, `teardown`
  - `vibe-board/src/browser_routes.rs` — `get_agent_browser`, `post_take_control`, `post_release`, `browser_router`
  - `vibe-board/templates/browser_session.html` — embedded session view
