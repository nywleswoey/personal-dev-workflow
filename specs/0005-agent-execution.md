---
id: 0005
title: Agent execution with enforced TDD and atomic commits
status: draft
related: [2, 4]
---

## Why

Agents must write a failing test tied to an acceptance criterion *before* any production code, and commits must be small and atomic so review stays cheap. The assistant can't simply be trusted to follow those rules — a git `pre-commit` hook installed at worktree creation (spec 0002) enforces them regardless of which backend is driving. Agent runs also need to be resumable: when the user unblocks a `needs_input` run, the backend must continue the same conversation, not start over. Without enforced TDD, commit discipline, and session continuity, traceability degrades and agent work loses context between turns.

## User-visible behavior

The user picks a ready issue and clicks "Run agent". A new agent run starts in the workspace's worktree for the issue's repo, and a live activity stream shows every command and test output as it happens. Every `git commit` the agent attempts passes through the worktree's pre-commit hook, which rejects the commit if no test file matching the issue's spec id is staged, or if the commit touches more files than policy allows. When every test tied to the issue's acceptance criterion passes, the runner stops the agent and moves the issue to `review`. When the agent needs user input, the run parks in `needs_input` and later resumes the *same* backend session rather than starting a fresh conversation.

## Acceptance criteria

- GIVEN a ready issue `I` in workspace `W` for repo `R`, WHEN I `POST /issues/I/agent` with body `agent=stub`, THEN an `agent_runs` row is inserted with status `running`, `working_dir` equal to `W`'s worktree path for `R`, `external_session_id` null, and the response returns the new agent run id. The backend is invoked with the environment variable `VIBE_BOARD_AGENT_RUN_ID` set so the pre-commit hook can identify the run.
- GIVEN a running agent `A`, WHEN I `GET /agents/A/stream` (SSE), THEN the response content type is `text/event-stream` and each activity entry appended via `record_activity` is delivered as a `data:` event in order.
- GIVEN a running agent `A` whose backend emits a session identifier in its first streamed event, WHEN that event is observed, THEN `agent_runs.external_session_id` is persisted with that value and a subsequent `get_backend_session(A)` returns it.
- GIVEN a running agent `A` on issue `I`, WHEN `vibe-board-commit-check` is invoked with `VIBE_BOARD_AGENT_RUN_ID=A` and the staged tree contains no file matching `vibe-board/tests/spec_NNNN_*.rs` where `NNNN` is the id of `I`'s spec, THEN the checker exits non-zero, a `commit rejected: missing spec test` activity entry is written against `A`, and no commit is created on the branch.
- GIVEN a running agent `A`, WHEN `vibe-board-commit-check` is invoked with a staged tree that touches more files than `CommitPolicy::max_files_per_commit` (default 5), THEN the checker exits non-zero, an activity entry naming the policy is written, and no commit is created.
- GIVEN a running agent `A` on issue `I` whose linked tests all pass after a commit, WHEN `check_progress` is invoked, THEN `I` transitions to status `review`, `A` transitions to status `stopped`, and a final activity entry `issue ready for review` is appended.
- GIVEN a running agent, WHEN the agent requires user input and calls `request_input(prompt)`, THEN the agent run transitions to status `needs_input`, the prompt is stored, and the agent loop blocks until a response is posted to `POST /agents/A/input`.
- GIVEN an agent run `A` in status `needs_input` with a non-null `external_session_id`, WHEN I `POST /agents/A/input` with body `response=...`, THEN the backend is re-invoked with `resume = A.external_session_id` (not as a fresh session), the run transitions back to `running`, and the response is delivered to the resumed session.

## Out of scope

- The identity of the agent backend (Claude Code, a local model, a shell script) — `AgentBackend` is a trait; the default in tests is a scripted stub. A `ClaudeCodeBackend` implementation is expected but its wiring is not governed by this spec.
- Installation of the pre-commit hook into a worktree — that is owned by spec 0002. This spec owns the checker binary the hook invokes and the policy definition.
- Automatic rollback or revert of rejected commits beyond "don't create them".
- Parallel agent runs on the same issue (one active run at a time per issue).
- Any UI rendering concerns beyond the SSE endpoint and JSON responses.
- Browser-driving agent operations (spec 0006).

## Traceability

- Test file: `vibe-board/tests/spec_0005_agent_execution.rs`
- Implementation entry points:
  - `vibe-board/migrations/0005_agents.sql` — `agent_runs` (including `external_session_id` column), `agent_activity` tables
  - `vibe-board/src/agent.rs` — `AgentRun`, `AgentStatus`, `AgentBackend`, `BackendSession`, `CommitPolicy`
  - `vibe-board/src/agent_runner.rs` — `start_run`, `check_progress`, `record_activity`, `record_backend_session`, `request_input`, `submit_input`, `resume_run`
  - `vibe-board/src/bin/commit_check.rs` — `vibe-board-commit-check` binary; reads `VIBE_BOARD_AGENT_RUN_ID`, enforces `CommitPolicy`, writes rejection activity
  - `vibe-board/src/agent_routes.rs` — `post_issue_agent`, `get_agent_stream`, `post_agent_input`, `agent_router`
