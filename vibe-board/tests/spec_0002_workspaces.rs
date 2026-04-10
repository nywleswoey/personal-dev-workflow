// Acceptance tests for specs/0002-workspaces.md
//
// One #[test] per acceptance criterion. Each body is `todo!()` until the
// matching implementation issue lands. The CLAUDE.md TDD rule requires
// these to fail before any production code is written; `todo!()` panics
// at runtime, satisfying the red baseline.
//
// Test-binary helpers (setup, make_repo) live at the top so each test
// stays self-contained but shares the same fixture pattern.

use std::path::PathBuf;
use std::process::Command;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use sqlx::SqlitePool;
use tempfile::TempDir;
use tower::ServiceExt;

use vibe_board::{build_router, db::init_pool};

#[allow(dead_code)]
async fn body_string(resp: axum::response::Response) -> String {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    String::from_utf8(bytes.to_vec()).unwrap()
}

#[allow(dead_code)]
struct TestApp {
    router: axum::Router,
    pool: SqlitePool,
    workspace_root: PathBuf,
    checker_path: Option<PathBuf>,
    _db_tmp: TempDir,
    _ws_tmp: TempDir,
}

#[allow(dead_code)]
async fn setup() -> TestApp {
    setup_with_checker(Some(PathBuf::from(env!("CARGO_BIN_EXE_vibe-board-commit-check")))).await
}

#[allow(dead_code)]
async fn setup_with_checker(checker_path: Option<PathBuf>) -> TestApp {
    let db_tmp = tempfile::tempdir().expect("db tempdir");
    let ws_tmp = tempfile::tempdir().expect("workspace tempdir");
    let db_path = db_tmp.path().join("test.db");
    let url = format!("sqlite://{}?mode=rwc", db_path.display());
    let pool = init_pool(&url).await.expect("init pool");
    let workspace_root = ws_tmp.path().to_path_buf();
    let router = build_router(pool.clone(), workspace_root.clone(), checker_path.clone());
    TestApp {
        router,
        pool,
        workspace_root,
        checker_path,
        _db_tmp: db_tmp,
        _ws_tmp: ws_tmp,
    }
}

/// Create a real on-disk git repo with a single empty commit and return
/// its canonicalized absolute path. Used by happy-path tests; tests for
/// invalid repos build paths by hand.
#[allow(dead_code)]
fn make_repo(parent: &std::path::Path, name: &str) -> PathBuf {
    let path = parent.join(name);
    std::fs::create_dir_all(&path).expect("mkdir repo");
    run_git(&path, &["init", "--initial-branch=main"]);
    run_git(&path, &["config", "user.email", "test@example.com"]);
    run_git(&path, &["config", "user.name", "Test"]);
    run_git(&path, &["commit", "--allow-empty", "-m", "initial"]);
    std::fs::canonicalize(&path).expect("canonicalize repo path")
}

#[allow(dead_code)]
fn run_git(cwd: &std::path::Path, args: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(cwd)
        .args(args)
        .status()
        .expect("git spawn");
    assert!(status.success(), "git {args:?} failed in {cwd:?}");
}

// =====================================================================
// Listing
// =====================================================================

// AC: GIVEN no active workspaces exist, WHEN I GET /workspaces, THEN
// the response is 200 with the heading "Active workspaces" and no
// data-workspace-name attributes.
#[tokio::test]
async fn ac_listing_empty() {
    let app = setup().await;
    let resp = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/workspaces")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp).await;
    assert!(
        body.contains("Active workspaces"),
        "expected 'Active workspaces' heading, body was: {body}"
    );
    assert!(
        !body.contains("data-workspace-name="),
        "expected no data-workspace-name attributes, body was: {body}"
    );
}

// =====================================================================
// Creation — happy paths
// =====================================================================

// AC: GIVEN a valid git repository at path R, WHEN I POST /workspaces
// with name=feat-login&repos=R, THEN a workspace row is inserted, a
// worktree exists at <workspace_root>/feat-login/<slug> on branch
// vibe-board/feat-login, and the active list contains it.
#[tokio::test]
async fn ac_create_single_repo() {
    todo!("issue #3: implement single-repo creation happy path");
}

// AC: GIVEN two valid repos R1 and R2, WHEN I POST with both, THEN one
// workspace row + two workspace_repos rows are inserted and both
// worktrees exist on branch vibe-board/multi.
#[tokio::test]
async fn ac_create_multi_repo() {
    todo!("issue #4: implement multi-repo creation");
}

// AC: GIVEN a workspace already references repo R, WHEN I POST a second
// workspace also referencing R, THEN both workspaces are active and each
// has its own worktree on its own branch.
#[tokio::test]
async fn ac_two_workspaces_same_repo() {
    todo!("issue #4: two workspaces referencing the same repo");
}

// =====================================================================
// Creation — validation
// =====================================================================

// AC: empty name or empty repos → 400, no row inserted, no worktrees.
#[tokio::test]
async fn ac_create_empty_name_or_repos() {
    todo!("issue #5: empty name/repos validation");
}

// AC: name not matching ^[a-z0-9][a-z0-9-]{0,62}$ → 400 with
// data-error="name-invalid".
#[tokio::test]
async fn ac_create_invalid_name() {
    todo!("issue #5: invalid name validation");
}

// AC: existing workspace name → 400 with data-error="name-taken".
#[tokio::test]
async fn ac_create_name_taken() {
    todo!("issue #5: duplicate name validation");
}

// AC: relative repo path → 400 with data-error="repo-not-absolute".
#[tokio::test]
async fn ac_create_relative_path() {
    todo!("issue #5: relative path validation");
}

// AC: repo entry that is not a valid repo, is a subdirectory of one,
// or has no HEAD → 400 with data-error="repo-invalid", no row, no worktrees.
#[tokio::test]
async fn ac_create_repo_invalid() {
    todo!("issue #5: invalid repo validation (multiple cases)");
}

// AC: two repos in one POST whose canonical basenames produce the same
// slug → 400 with data-error="slug-collision".
#[tokio::test]
async fn ac_create_slug_collision() {
    todo!("issue #4: slug collision validation");
}

// AC: branch vibe-board/<name> already exists in any listed repo → 400
// with data-error="branch-exists".
#[tokio::test]
async fn ac_create_branch_exists() {
    todo!("issue #6: pre-existing branch rejection");
}

// AC: vibe-board-commit-check not on PATH at creation time → 400 with
// data-error="commit-check-missing", no row, no worktrees.
#[tokio::test]
async fn ac_create_commit_check_missing() {
    todo!("issue #5: missing commit-check binary");
}

// AC: POST with two repos where R1 is valid and R2 is invalid → 400,
// no rows, no worktree dir for R1, no leftover branch in R1.
#[tokio::test]
async fn ac_create_partial_failure_rollback() {
    todo!("issue #6: multi-repo atomicity rollback");
}

// =====================================================================
// Archive
// =====================================================================

// AC: archive an active single-repo workspace → status flips, archived_at
// set, worktree dir gone, branch vibe-board/<name> still exists, list no
// longer contains it.
#[tokio::test]
async fn ac_archive_single() {
    todo!("issue #7: archive single-repo workspace");
}

// AC: archive a multi-repo workspace → every worktree directory is removed.
#[tokio::test]
async fn ac_archive_multi() {
    todo!("issue #7: archive multi-repo workspace");
}

// AC: POST /workspaces/nonexistent/archive → 404.
#[tokio::test]
async fn ac_archive_nonexistent() {
    todo!("issue #7: archive nonexistent → 404");
}

// AC: archive an already-archived workspace → 200, idempotent no-op.
#[tokio::test]
async fn ac_archive_idempotent() {
    todo!("issue #7: archive idempotent");
}

// =====================================================================
// Hook installation
// =====================================================================

// AC: after creation, .git/hooks/pre-commit exists, is executable, its
// contents contain the absolute path to vibe-board-commit-check, and
// invoking it with no staged changes exits 0.
#[tokio::test]
async fn ac_hook_installed() {
    todo!("issue #3: hook shim installed with absolute checker path");
}
