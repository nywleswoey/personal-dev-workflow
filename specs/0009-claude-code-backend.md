---
id: 0009
title: ClaudeCodeBackend via claude CLI shell-out
status: draft
related: [5]
---

## Why

Spec 0005 defines `AgentBackend` as a trait and uses a scripted stub in tests, leaving production wiring explicitly out of scope. Without a concrete backend, no real agent run can happen — the loop works only in fakes. This spec provides the one implementation that matters for day-one use: `ClaudeCodeBackend`, which drives Claude Code by spawning the `claude` CLI, streaming its `stream-json` output into the activity log, capturing the session id for resume, and re-invoking `claude --resume <id>` when a parked run is unblocked. Shelling out to the CLI keeps the Rust crate free of a Python/TypeScript SDK dependency while still getting every session primitive (cwd, streaming, resume, env passthrough).

## User-visible behavior

When a user starts an agent run and the configured backend is `claude_code`, the runner spawns `claude` as a child process inside the run's worktree, pipes the prompt in, and consumes its streaming JSON output. Each event in the stream — assistant message, tool use, tool result — is appended to the agent's activity log so the user sees it live via the SSE endpoint from spec 0005. The session id emitted in the first event is persisted to `agent_runs.external_session_id`. When the user later responds to a `needs_input` run, the runner re-invokes `claude --resume <session_id>` with the user's response as stdin, so the agent continues its prior conversation rather than starting over. The user never interacts with the CLI directly.

## Acceptance criteria

- GIVEN a `ClaudeCodeBackend` constructed with a `FakeClaudeCli` and a ready agent run `A` with `working_dir = /tmp/wt` and `id = 42`, WHEN `AgentBackend::start(A, prompt)` is called, THEN `FakeClaudeCli::spawn` is invoked exactly once with args including `--output-format stream-json` and `--print`, `cwd = /tmp/wt`, env containing `VIBE_BOARD_AGENT_RUN_ID=42`, and stdin containing the exact bytes of `prompt` followed by EOF.
- GIVEN a fake CLI scripted to emit a first stdout line `{"type":"system","session_id":"s_abc",...}`, WHEN the backend consumes that line, THEN `record_backend_session(A, "s_abc")` is called exactly once, and no subsequent line triggers another call to `record_backend_session` for `A`.
- GIVEN a fake CLI scripted to emit three stdout lines of types `assistant`, `tool_use`, `tool_result` in order, WHEN the backend consumes them, THEN `record_activity` is called three times against `A` with payloads preserving event type and content, in the same order the CLI emitted them.
- GIVEN an agent run `A` with `external_session_id = "s_abc"` in status `needs_input`, WHEN `AgentBackend::resume(A, user_response)` is called, THEN `FakeClaudeCli::spawn` is invoked with args including `--resume s_abc` and `--output-format stream-json`, `cwd = A.working_dir`, env containing `VIBE_BOARD_AGENT_RUN_ID = A.id`, and stdin containing the exact bytes of `user_response` followed by EOF.
- GIVEN a fake CLI scripted to exit with status 2 and stderr `auth failed`, WHEN the backend observes process exit, THEN `A` transitions to status `failed`, an activity entry `backend exited: 2: auth failed` is appended, and `AgentBackend::start` returns `Err(BackendError::ProcessExit { code: 2 })`.
- GIVEN a fake CLI scripted to emit a malformed line `not json` between two well-formed events, WHEN the parser encounters the malformed line, THEN an activity entry `parse error: not json` is appended, stream consumption continues, and the well-formed event after the malformed line is still recorded via `record_activity`.
- GIVEN a fake CLI that never emits a `session_id` event before exiting cleanly, WHEN the backend observes clean exit, THEN `agent_runs.external_session_id` for `A` remains null, an activity entry `backend exited without session id` is appended, and `AgentBackend::start` returns `Err(BackendError::MissingSessionId)`.
- GIVEN a `ClaudeCodeBackend::system()` constructor in test configuration that returns a `SystemClaudeCli` pointing at a binary that does not exist on `$PATH`, WHEN `AgentBackend::start` is called, THEN it returns `Err(BackendError::SpawnFailed)` without panicking and without writing any activity.

## Out of scope

- One-shot (non-session) `claude` calls used by spec 0003 (intake question generation) or spec 0004 (issue breakdown) — covered by a separate later spec.
- Authentication, API-key management, or login flows — this spec assumes `claude` is already authenticated on the host.
- A process pool, queue, or scheduler for concurrent agent runs — the runner owns concurrency (spec 0005); this spec is per-invocation.
- Retries, rate-limit handling, or backoff on backend errors — future work; this spec fails fast and lets the runner decide.
- Containerization or sandboxing of the spawned process — the worktree from spec 0002 is the only isolation boundary.
- Any HTTP or network client; this backend only speaks to `claude` via stdin/stdout/stderr + env.
- Exact `claude` CLI flag spellings — the ACs name behaviorally-significant flags (`--output-format stream-json`, `--print`, `--resume`) whose current names are verified during implementation. If upstream renames them, this spec's ACs remain valid in intent and the implementation adapts.

## Traceability

- Test file: `vibe-board/tests/spec_0009_claude_code_backend.rs`
- Implementation entry points:
  - `vibe-board/src/claude_code_backend.rs` — `ClaudeCodeBackend`, `ClaudeCli`, `FakeClaudeCli`, `SystemClaudeCli`, `BackendError`, `parse_stream_event`, `ChildHandle`
