---
id: 0004
title: Issue breakdown with dependency graph
status: draft
related: [3]
---

## Why

Specs are too coarse for agents to execute — a spec can span multiple files and repos. Issues are the unit of agent work; each issue links back to one or more acceptance criteria in a spec. Dependencies between issues must be explicit so the runner (spec 0005) can pick independent issues to hand to parallel agents without scheduling them to collide.

## User-visible behavior

From a finalized spec the user clicks "Break down", and the app produces a list of issues, each tied to a specific acceptance criterion bullet in the spec. The user can declare that issue `B` depends on issue `A` (cycles are rejected). A "ready" view lists only the issues whose dependencies are all done. A "coverage" view lists any acceptance criterion that has no issue linked to it — these are gaps that block the spec from moving to `done`.

## Acceptance criteria

- GIVEN a finalized spec with id `S` containing three acceptance criteria, WHEN I `POST /specs/S/breakdown` with a JSON body listing three issue titles each with an `acceptance_criterion_ref` pointing at one of the bullets in the spec, THEN three rows are inserted into the `issues` table, each with status `todo`, non-null `acceptance_criterion_ref`, and `spec_id = S`, and the response returns the new issue ids.
- GIVEN issues `A` and `B` in the same workspace, WHEN I `POST /issues/B/depends_on` with body `depends_on=A`, THEN a row is inserted into `issue_dependencies` linking `B` to `A`, and `GET /issues/B` shows `A` in its dependency list.
- GIVEN issues `A → B → C` with `C` depending on `A` already, WHEN I `POST /issues/A/depends_on` with body `depends_on=C` (which would create a cycle), THEN the response status is 400, no row is inserted, and the existing graph is unchanged.
- GIVEN a workspace `W` with issues `A` (done), `B` (todo, depends on `A`), and `C` (todo, depends on a non-done issue), WHEN I `GET /workspaces/W/issues/ready`, THEN the response lists only `B` (its sole dependency is done) and does not list `C`.
- GIVEN a spec `S` with acceptance criteria `[a1, a2, a3]` and issues linked to `a1` and `a2` only, WHEN I `GET /specs/S/coverage`, THEN the response flags `a3` as uncovered and returns `status: incomplete`.
- GIVEN a spec `S` with every acceptance criterion linked to at least one issue, WHEN I `GET /specs/S/coverage`, THEN the response returns `status: complete` and an empty uncovered list.

## Out of scope

- Automatic breakdown of a spec into issues by the assistant — this spec exposes the API; authoring decisions happen outside it (the assistant or the user populates the POST body).
- Cross-workspace dependencies.
- Priority or ordering beyond dependency edges (no "urgent" flag).
- Bulk re-parenting of issues when a spec is edited.

## Traceability

- Test file: `vibe-board/tests/spec_0004_issues.rs`
- Implementation entry points:
  - `vibe-board/migrations/0004_issues.sql` — `issues`, `issue_dependencies` tables
  - `vibe-board/src/issue.rs` — `Issue`, `IssueStatus`, `create_issues`, `add_dependency`, `ready_issues`, `spec_coverage`, `detect_cycle`
  - `vibe-board/src/issue_routes.rs` — `post_spec_breakdown`, `post_issue_dependency`, `get_issue`, `get_workspace_ready_issues`, `get_spec_coverage`, `issue_router`
  - `vibe-board/templates/issues.html` — issue list and coverage pane
