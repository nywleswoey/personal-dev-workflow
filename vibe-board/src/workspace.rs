// Spec 0002: Workspaces with worktree isolation.
//
// This module is a red-baseline scaffold. Every function body is `todo!()`
// and will be filled in across issues #2-#7 of the implementation plan in
// specs/0002-workspaces.md. Type signatures are stable enough that the
// route layer and tests can compile against them.

use std::path::{Path, PathBuf};

use sqlx::SqlitePool;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceStatus {
    Active,
    Archived,
}

impl WorkspaceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            WorkspaceStatus::Active => "active",
            WorkspaceStatus::Archived => "archived",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Workspace {
    pub id: i64,
    pub name: String,
    pub status: WorkspaceStatus,
    pub repos: Vec<WorkspaceRepo>,
}

#[derive(Debug, Clone)]
pub struct WorkspaceRepo {
    pub repo_path: PathBuf,
    pub repo_slug: String,
    pub worktree_path: PathBuf,
    pub branch_name: String,
}

/// Errors `create_workspace` and `archive_workspace` return. The route
/// layer maps these to 4xx responses with stable `data-error="..."` hooks
/// (see User-visible behavior § Response format in specs/0002-workspaces.md).
#[derive(Debug)]
pub enum WorkspaceError {
    NameInvalid,
    NameTaken,
    ReposEmpty,
    RepoNotAbsolute(String),
    RepoInvalid(String),
    SlugCollision(String),
    BranchExists { repo: PathBuf, branch: String },
    NotFound,
    Db(sqlx::Error),
    Io(std::io::Error),
    Git(String),
}

impl From<sqlx::Error> for WorkspaceError {
    fn from(e: sqlx::Error) -> Self {
        WorkspaceError::Db(e)
    }
}

impl From<std::io::Error> for WorkspaceError {
    fn from(e: std::io::Error) -> Self {
        WorkspaceError::Io(e)
    }
}

/// Inputs to `create_workspace`. Repos are raw user input; canonicalization
/// and slugging happen inside the function.
#[derive(Debug, Clone)]
pub struct CreateWorkspace {
    pub name: String,
    pub repos: Vec<String>,
}

pub async fn create_workspace(
    _pool: &SqlitePool,
    _input: CreateWorkspace,
    _workspace_root: &Path,
    _checker_path: &Path,
) -> Result<Workspace, WorkspaceError> {
    todo!("issue #3-#6: implement create_workspace")
}

pub async fn archive_workspace(
    _pool: &SqlitePool,
    _name: &str,
) -> Result<(), WorkspaceError> {
    todo!("issue #7: implement archive_workspace")
}

pub async fn list_active_workspaces(
    _pool: &SqlitePool,
) -> Result<Vec<Workspace>, WorkspaceError> {
    todo!("issue #2: implement list_active_workspaces")
}

pub fn install_worktree_hooks(
    _worktree_path: &Path,
    _checker_path: &Path,
) -> Result<(), WorkspaceError> {
    todo!("issue #3: implement install_worktree_hooks")
}
