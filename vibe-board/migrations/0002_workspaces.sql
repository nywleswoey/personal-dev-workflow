CREATE TABLE workspaces (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT    NOT NULL UNIQUE,
    status      TEXT    NOT NULL CHECK (status IN ('active', 'archived')),
    created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    archived_at DATETIME NULL
);

CREATE TABLE workspace_repos (
    workspace_id  INTEGER NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    repo_path     TEXT    NOT NULL,
    repo_slug     TEXT    NOT NULL,
    worktree_path TEXT    NOT NULL,
    branch_name   TEXT    NOT NULL,
    PRIMARY KEY (workspace_id, repo_slug)
);
