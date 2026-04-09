---
id: 0007
title: Review, comment round-trip, MR creation, archival
status: draft
related: [2, 4, 5]
---

## Why

The gap between "agent finished" and "merged to main" is where the user actually spends attention. Diff review, comments going back to the agent, merge-request creation against the correct git host, and workspace archival on merge all need to live in one pane so the user is never bounced between tools. Without this, the spec-driven loop ends halfway and the last mile stays manual.

## User-visible behavior

When an issue is in `review`, the user opens `/issues/:id/review` and sees a syntax-highlighted diff for every commit the agent made on the issue branch. They can post a comment tied to a specific file and line; posting a comment transitions the issue back to `needs_input` and makes the comment visible to the next agent run. When all issues for a spec are `done`, the user clicks "Open MRs" and one merge request is created per touched repo via the configured `GitHost` — the default implementation is `GitHubHost`, which shells out to `gh`. When all merge requests for a workspace are merged, the workspace auto-archives.

## Acceptance criteria

- GIVEN an issue `I` in status `review`, WHEN I `GET /issues/I/review`, THEN the response is 200 and contains, for every commit on `I`'s branch, the commit SHA and a unified diff for each changed file.
- GIVEN an issue `I` in status `review`, WHEN I `POST /issues/I/comments` with body `file=src/foo.rs&line=12&body=wrong var name`, THEN a row is inserted in `review_comments` linked to `I`, the issue transitions to `needs_input`, and `GET /issues/I` lists the comment.
- GIVEN a spec `S` whose every issue is in status `done` and which touches repos `R1` and `R2`, WHEN I `POST /specs/S/mrs`, THEN `GitHost::open_merge_request` is called once per repo with the branch name, the returned URLs are persisted in `merge_requests` linked to `S`, and the response returns the list of URLs.
- GIVEN a spec `S` with at least one issue not in status `done`, WHEN I `POST /specs/S/mrs`, THEN the response status is 409, no `GitHost` call is made, and no row is inserted in `merge_requests`.
- GIVEN a workspace `W` with open merge requests, WHEN `poll_merge_status` observes that every `merge_requests` row for `W` has state `merged`, THEN `W` transitions to status `archived` (via the archive path from spec 0002) and a row is written to `archive_events` naming `W` and the triggering spec.
- GIVEN the default configuration, WHEN `GitHost::open_merge_request` is invoked, THEN the concrete implementation is `GitHubHost` which shells out to `gh pr create`, and tests may substitute a `FakeGitHost` for assertions.

## Out of scope

- Rendering the diff in the terminal or any non-HTML format.
- Threaded comments or comment resolution workflows.
- Merge-queue integration or stacked-PR workflows.
- Git hosts other than GitHub — the trait is the seam; other hosts get their own specs.
- Automatic rebase/conflict resolution on the agent branch.

## Traceability

- Test file: `vibe-board/tests/spec_0007_review_and_merge.rs`
- Implementation entry points:
  - `vibe-board/migrations/0007_review.sql` — `review_comments`, `merge_requests`, `archive_events` tables
  - `vibe-board/src/review.rs` — `ReviewComment`, `list_commits_for_issue`, `post_comment`, `poll_merge_status`
  - `vibe-board/src/git_host.rs` — `GitHost` trait, `MergeRequest`, `MergeState`, `FakeGitHost`
  - `vibe-board/src/github_host.rs` — `GitHubHost`, `open_merge_request`, `fetch_state`
  - `vibe-board/src/review_routes.rs` — `get_issue_review`, `post_issue_comment`, `post_spec_mrs`, `review_router`
  - `vibe-board/templates/review.html` — diff and comment pane
