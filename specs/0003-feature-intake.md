---
id: 0003
title: Feature intake and spec authoring loop
status: draft
related: [2]
---

## Why

The user describes a feature in plain English and the assistant interviews them until the shape of the work is clear, then writes a spec file into `specs/`. Conflict detection runs automatically before the spec is accepted so authoring can't silently clash with existing work. Without this, spec authoring stays a manual shell dance and the loop can't start from inside the app.

## User-visible behavior

Inside an active workspace, the user clicks "Describe a feature", types a freeform description, and is asked a series of clarifying questions one at a time. When the assistant has enough information, the user clicks "Finalize" and the app writes a new `NNNN-<slug>.md` spec file under `specs/` with status `draft`, prefilled with `Why`, `User-visible behavior`, `Acceptance criteria`, `Out of scope`, and `Traceability` sections. If finalization would create a spec whose Traceability overlaps with an existing live spec, finalization is rejected and the conflicting spec IDs are shown.

## Acceptance criteria

- GIVEN an active workspace with id `W`, WHEN I `POST /workspaces/W/features` with form body `description=add login`, THEN a row is inserted into the `feature_drafts` table with status `drafting`, workspace_id `W`, and the given description, and the response is 201.
- GIVEN a draft feature with id `F`, WHEN I `GET /features/F`, THEN the response contains the description, the list of questions asked so far with their answers, and the next question to ask (or `null` if intake is complete).
- GIVEN a draft feature `F` with a pending question, WHEN I `POST /features/F/answers` with form body `answer=...`, THEN the answer is persisted, the next question (produced by the `QuestionGenerator` trait) is stored as pending, and the response returns the new pending question.
- GIVEN a draft feature `F` whose intake is complete, WHEN I `POST /features/F/finalize`, THEN a new file `specs/NNNN-<slug>.md` is written where `NNNN` is the next unused 4-digit ID, the file parses with `status: draft` frontmatter, the feature row transitions to status `finalized` with `spec_id = NNNN`, and the response returns the new spec id.
- GIVEN a draft feature whose finalized Traceability would claim a file already claimed by a live spec, WHEN I `POST /features/F/finalize`, THEN the response status is 409, no spec file is written, the feature stays in `drafting`, and the response body lists the conflicting spec IDs.
- GIVEN I `POST /workspaces/W/features` with `description=` (empty) or against a workspace id that does not exist, THEN the response status is 400 or 404 respectively and no row is inserted.

## Out of scope

- The actual LLM prompt used by the default `QuestionGenerator` — the trait is a seam; tests use a scripted stub. Choosing the production prompt is its own work.
- Editing an already-finalized spec (a new spec must be authored instead).
- Bulk import of existing markdown as specs.
- Any UI concerns beyond the forms and JSON responses named above.

## Traceability

- Test file: `vibe-board/tests/spec_0003_feature_intake.rs`
- Implementation entry points:
  - `vibe-board/migrations/0003_features.sql` — `feature_drafts`, `feature_questions` tables
  - `vibe-board/src/feature_intake.rs` — `FeatureDraft`, `FeatureStatus`, `create_draft`, `record_answer`, `finalize_draft`, `QuestionGenerator`
  - `vibe-board/src/spec_authoring.rs` — `write_spec_file`, `next_spec_id`, `check_traceability_conflicts`
  - `vibe-board/src/feature_routes.rs` — `post_feature`, `get_feature`, `post_feature_answer`, `post_feature_finalize`, `feature_router`
  - `vibe-board/templates/feature_intake.html` — intake form and question/answer pane
