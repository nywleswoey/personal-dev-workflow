---
id: 0002
title: Workspaces with worktree isolation
status: in-progress
---

## Why

Every other capability in this tool assumes "the thing I'm working on right now" has a bounded identity — a set of repos, a branch strategy, and a place for agents to run without stepping on each other or on the user's editor. Without workspaces, parallel agent work is unsafe and context bleeds across unrelated features. Workspaces are the container the rest of the loop hangs off.

## User-visible behavior

A user opens `http://localhost:3000/workspaces` and sees a list of workspaces with their name, linked repos, and status. They can create a new workspace by naming it and listing one or more absolute paths to local git repositories. On creation, the app creates a git worktree per repo under a workspace-scoped directory, checks each worktree out onto a new branch named `vibe-board/<workspace_name>` rooted at that repo's current HEAD, installs the `pre-commit` hook, and records the workspace in the database. Two workspaces may list the same underlying repository and each gets its own independent worktree on its own branch. Archiving a workspace removes its worktree directories from disk, leaving the underlying repositories and the `vibe-board/<workspace_name>` branches untouched. Only active workspaces are shown; archived workspaces are retained as database rows for referential integrity but are not rendered in the UI.

### Configuration

- `VIBE_BOARD_WORKSPACE_ROOT` — absolute directory under which workspace directories are created. If unset, defaults to `dirs::data_dir().join("vibe-board/workspaces")` (e.g. `~/Library/Application Support/vibe-board/workspaces` on macOS).
- `vibe-board-commit-check` — binary that the installed `pre-commit` hook execs. Must be resolvable on `PATH` at workspace-creation time; the absolute path is baked into the hook shim.

### Workspace names

Workspace names must match `^[a-z0-9][a-z0-9-]{0,62}$` (lowercase kebab-case, starts with an alphanumeric, max 63 chars). The name is used as the directory name, the branch suffix (`vibe-board/<name>`), the URL path segment, and the database unique key; one canonical form everywhere.

### Repo paths

Each entry in `repos` must be an absolute path (starts with `/`) to the root of an existing git repository with at least one commit. Paths are canonicalized (via `fs::canonicalize`) before validation and storage, so symlinks, trailing slashes, and `.`/`..` segments are normalized. A path is considered a valid repo root when `git -C <path> rev-parse --show-toplevel` equals the canonicalized input and `git -C <path> rev-parse HEAD` succeeds.

### Repo slugs

The on-disk directory for a repo inside a workspace is `<workspace_root>/<workspace_name>/<repo_slug>`. The slug is derived from the basename of the canonicalized repo path: lowercased, with a trailing `.git` stripped, and any run of non-`[a-z0-9-]` characters collapsed to a single `-`. If two repos in the same POST produce the same slug, creation fails with 400.

### Response format

All responses are HTML. The listing template emits stable hooks for tests and htmx: each workspace row carries `data-workspace-name="<name>"` and `data-workspace-status="active"`; validation errors render a fragment containing `data-error="<code>"` (e.g. `data-error="name-invalid"`, `data-error="repo-not-found"`, `data-error="branch-exists"`). The heading `Active workspaces` is always present.

### Creation and archive semantics

Creation is all-or-nothing. Every input is validated up front (name shape, name uniqueness, absolute paths, repo root check, HEAD exists, `vibe-board/<name>` branch does not already exist in any listed repo, slug collisions, `vibe-board-commit-check` resolvable on PATH) before any side effect. Only then does the server insert rows inside a SQL transaction and create worktrees sequentially. If any `git worktree add` fails, every worktree created earlier in the same request is removed, every branch created earlier in the same request is deleted (`git branch -D`, safe because the branches were made seconds ago in this same request), the SQL transaction is rolled back, and the response is an error.

Archive is best-effort and idempotent. For each repo the workspace references, the server `rm -rf`s `<worktree_path>` and runs `git -C <repo_path> worktree prune` (skipped if `<repo_path>` no longer exists on disk). The workspace row is flipped to `archived` and `archived_at` is set regardless of per-repo hiccups; any per-repo failures are logged but do not fail the request. Archiving does not delete the `vibe-board/<workspace_name>` branch in any repo — agent work is preserved.

## Acceptance criteria

### Listing

- GIVEN no active workspaces exist, WHEN I `GET /workspaces`, THEN the response status is 200, the body contains the heading `Active workspaces`, and the body contains no `data-workspace-name="..."` attributes.

### Creation — happy paths

- GIVEN a valid git repository at path `R`, WHEN I `POST /workspaces` with form body `name=feat-login&repos=R`, THEN a row is inserted into `workspaces` with `status='active'`, a `workspace_repos` row links it to `R`, a git worktree exists at `<workspace_root>/feat-login/<repo-slug>`, its HEAD is on branch `vibe-board/feat-login` rooted at `R`'s HEAD at creation time, and `GET /workspaces` contains `data-workspace-name="feat-login"`.
- GIVEN two valid git repositories at paths `R1` and `R2`, WHEN I `POST /workspaces` with `name=multi&repos=R1&repos=R2`, THEN one `workspaces` row and two `workspace_repos` rows are inserted, worktrees exist at `<workspace_root>/multi/<slug-R1>` and `<workspace_root>/multi/<slug-R2>`, and each is checked out on branch `vibe-board/multi` in its respective repo.
- GIVEN a workspace `feat-login` already references repo `R`, WHEN I `POST /workspaces` with `name=feat-billing&repos=R`, THEN a second workspace is created with its own worktree at `<workspace_root>/feat-billing/<repo-slug>` on branch `vibe-board/feat-billing`, and both workspaces appear in the active list.

### Creation — validation

- GIVEN I `POST /workspaces` with `name=` (empty) or `repos=` (empty), THEN the response is 400, no row is inserted, and no worktree directories are created.
- GIVEN I `POST /workspaces` with a name that does not match `^[a-z0-9][a-z0-9-]{0,62}$` (e.g. `Feat-Login`, `feat_login`, leading hyphen, 64+ chars), THEN the response is 400 with `data-error="name-invalid"` and no row is inserted.
- GIVEN a workspace `feat-login` already exists, WHEN I `POST /workspaces` with `name=feat-login&repos=R`, THEN the response is 400 with `data-error="name-taken"` and no additional row is inserted.
- GIVEN I `POST /workspaces` with a `repos` entry that is a relative path (e.g. `./foo`), THEN the response is 400 with `data-error="repo-not-absolute"` and no row is inserted.
- GIVEN I `POST /workspaces` with a `repos` entry that is not a valid git repository, is a subdirectory of a repo rather than its root, or is a repo with no commits (`HEAD` missing), THEN the response is 400 with `data-error="repo-invalid"`, no row is inserted, and no worktree directories are created on disk.
- GIVEN I `POST /workspaces` with two `repos` entries whose canonicalized basenames produce the same slug, THEN the response is 400 with `data-error="slug-collision"` and no row is inserted.
- GIVEN a repo `R` already has a branch named `vibe-board/feat-login` (e.g. from a previously archived workspace of the same name), WHEN I `POST /workspaces` with `name=feat-login&repos=R`, THEN the response is 400 with `data-error="branch-exists"` and no row is inserted.
- GIVEN `vibe-board-commit-check` is not resolvable on `PATH`, WHEN I `POST /workspaces` with otherwise valid input, THEN the response is 400 with `data-error="commit-check-missing"`, no row is inserted, and no worktree directories are created.
- GIVEN I `POST /workspaces` with `repos=R1&repos=R2` where `R1` is valid and `R2` is invalid, THEN the response is 400, no `workspaces` row exists, no `workspace_repos` rows exist, no worktree exists at `<workspace_root>/<name>/<slug-R1>`, and `vibe-board/<name>` is not a branch in `R1`.

### Archive

- GIVEN an active workspace `feat-login` with one worktree in repo `R`, WHEN I `POST /workspaces/feat-login/archive`, THEN the workspace `status` flips to `archived`, `archived_at` is set, its worktree directory is removed from disk, `git worktree list` in `R` does not mention the removed worktree, branch `vibe-board/feat-login` still exists in `R`, and `GET /workspaces` no longer contains `data-workspace-name="feat-login"`.
- GIVEN an active workspace with multiple repos, WHEN I `POST /workspaces/<name>/archive`, THEN every worktree directory for that workspace is removed from disk.
- GIVEN no workspace exists with name `nonexistent`, WHEN I `POST /workspaces/nonexistent/archive`, THEN the response is 404.
- GIVEN a workspace `feat-login` is already archived, WHEN I `POST /workspaces/feat-login/archive`, THEN the response is 200 and the workspace remains archived.

### Hook installation

- GIVEN a workspace is successfully created with repo `R`, WHEN I inspect the resulting worktree for `R`, THEN `.git/hooks/pre-commit` exists, is executable, its contents contain the absolute path to `vibe-board-commit-check` (as resolved via `which` at install time), and invoking it with no staged changes exits 0. (The hook's policy logic and its acceptance rules are owned by spec 0005; this spec only asserts the hook shim is installed and points at the checker.)

## Out of scope

- Features, specs authored inside a workspace, issues, or agents (covered by specs 0003–0008).
- Authentication, multi-user visibility, or remote sync of workspaces.
- Editing a workspace's repo list after creation (archive + recreate instead).
- Specifying a base branch other than each repo's current HEAD at creation time.
- Listing or browsing archived workspaces from the UI.
- A UI for browsing worktree file contents.
- Any changes to the existing `/` Todo board from spec 0001.
- The policy logic inside `vibe-board-commit-check` (owned by spec 0005). Spec 0002 depends on at least a stub implementation that exits 0 on an empty staged tree.

## Implementation plan

Eight atomic commits move this spec from `in-progress` to `done`. Each ticks one box. Tests live in `vibe-board/tests/spec_0002_workspaces.rs` and start as `todo!()` panics — the red baseline.

- [x] **Issue #1 — Scaffolding + red baseline.** Module shells, `0002_workspaces.sql`, `commit_check.rs` stub, `Cargo.toml` deps and bin target, `lib.rs`/`main.rs` wiring for `AppState.workspace_root`/`checker_path`, all 18 test stubs, status flipped to `in-progress`.
- [ ] **Issue #2 — GET /workspaces empty list** (AC `ac_listing_empty`).
- [ ] **Issue #3 — POST single-repo happy path + hook installation** (AC `ac_create_single_repo`, `ac_hook_installed`).
- [ ] **Issue #4 — Multi-repo + same-repo-two-workspaces + slug collision** (AC `ac_create_multi_repo`, `ac_two_workspaces_same_repo`, `ac_create_slug_collision`).
- [ ] **Issue #5 — Creation validation 400s** (AC `ac_create_empty_name_or_repos`, `ac_create_invalid_name`, `ac_create_name_taken`, `ac_create_relative_path`, `ac_create_repo_invalid`, `ac_create_commit_check_missing`).
- [ ] **Issue #6 — Branch-exists rejection + atomicity rollback** (AC `ac_create_branch_exists`, `ac_create_partial_failure_rollback`).
- [ ] **Issue #7 — Archive** (AC `ac_archive_single`, `ac_archive_multi`, `ac_archive_nonexistent`, `ac_archive_idempotent`).
- [ ] **Issue #8 — Finalize: run app end-to-end, run `scripts/spec_index.py`, flip status to `done`.**

## Traceability

- Test file: `vibe-board/tests/spec_0002_workspaces.rs`
- Implementation entry points:
  - `vibe-board/migrations/0002_workspaces.sql` — `workspaces`, `workspace_repos` tables
  - `vibe-board/src/workspace.rs` — `Workspace`, `WorkspaceStatus`, `create_workspace`, `archive_workspace`, `list_active_workspaces`, `install_worktree_hooks`
  - `vibe-board/src/workspace_routes.rs` — `get_workspaces`, `post_workspace`, `post_workspace_archive`, `workspace_router`
  - `vibe-board/templates/workspaces.html` — workspaces list layout
