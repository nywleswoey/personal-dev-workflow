---
id: 0002
title: Workspaces with worktree isolation
status: draft
---

## Why

Every other capability in this tool assumes "the thing I'm working on right now" has a bounded identity — a set of repos, a branch strategy, and a place for agents to run without stepping on each other or on the user's editor. Without workspaces, parallel agent work is unsafe and context bleeds across unrelated features. Workspaces are the container the rest of the loop hangs off.

## User-visible behavior

A user opens `http://localhost:3000/workspaces` and sees a list of workspaces with their name, linked repos, and status (`active` or `archived`). They can create a new workspace by naming it and listing one or more git repository paths. On creation, the app makes a git worktree per repo under a workspace-scoped directory and records the workspace in the database. Two workspaces may list the same underlying repository and each gets its own independent worktree. Archiving a workspace removes it from the active list and deletes its worktrees, leaving the underlying repositories untouched.

## Acceptance criteria

- GIVEN no workspaces exist, WHEN I `GET /workspaces`, THEN the response status is 200 and the body contains an empty list under the heading "Active workspaces".
- GIVEN a valid git repository at path `R`, WHEN I `POST /workspaces` with form body `name=feat-login&repos=R`, THEN a row is inserted into the `workspaces` table with status `active`, a git worktree exists under `<workspace_root>/feat-login/<repo-slug>` pointing at repo `R`, and the response lists the new workspace.
- GIVEN a workspace `A` already references repo `R`, WHEN I `POST /workspaces` with `name=feat-billing&repos=R`, THEN a second workspace is created with its own independent worktree and both workspaces appear in the active list.
- GIVEN an active workspace `feat-login` with one worktree, WHEN I `POST /workspaces/feat-login/archive`, THEN the workspace status flips to `archived`, its worktree directory is removed from disk, the underlying repo `R` is untouched (`git worktree list` in `R` does not mention the removed worktree), and `GET /workspaces` no longer shows `feat-login` under "Active workspaces".
- GIVEN I `POST /workspaces` with `name=` (empty) or `repos=` (empty), THEN the response status is 400 and no row is inserted into the `workspaces` table.
- GIVEN I `POST /workspaces` with a `repos` entry that is not a valid git repository, THEN the response status is 400, no row is inserted, and no worktree directories are created on disk.
- GIVEN a workspace is successfully created with repo `R`, WHEN I inspect the resulting worktree for `R`, THEN `.git/hooks/pre-commit` exists and is executable, and invoking it with no staged changes exits 0. (The hook's policy logic and its acceptance rules are owned by spec 0005; this spec only asserts the hook is installed.)

## Out of scope

- Features, specs authored inside a workspace, issues, or agents (covered by specs 0003–0008).
- Authentication, multi-user visibility, or remote sync of workspaces.
- Editing a workspace's repo list after creation (archive + recreate instead).
- A UI for browsing worktree file contents.
- Any changes to the existing `/` Todo board from spec 0001.

## Traceability

- Test file: `vibe-board/tests/spec_0002_workspaces.rs`
- Implementation entry points:
  - `vibe-board/migrations/0002_workspaces.sql` — `workspaces`, `workspace_repos` tables
  - `vibe-board/src/workspace.rs` — `Workspace`, `WorkspaceStatus`, `create_workspace`, `archive_workspace`, `list_active_workspaces`, `install_worktree_hooks`
  - `vibe-board/src/workspace_routes.rs` — `get_workspaces`, `post_workspace`, `post_workspace_archive`, `workspace_router`
  - `vibe-board/templates/workspaces.html` — workspaces list layout
