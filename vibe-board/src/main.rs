use std::error::Error;
use std::net::SocketAddr;
use std::path::PathBuf;

use vibe_board::{build_router, db::init_pool};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "vibe_board=info,tower_http=info".into()),
        )
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://data/vibe-board.db?mode=rwc".to_string());

    if let Some(path) = database_url.strip_prefix("sqlite://") {
        let file = path.split('?').next().unwrap_or(path);
        if let Some(parent) = std::path::Path::new(file).parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).ok();
            }
        }
    }

    let workspace_root = resolve_workspace_root();
    std::fs::create_dir_all(&workspace_root).ok();
    let checker_path = which::which("vibe-board-commit-check").ok();
    if checker_path.is_none() {
        tracing::warn!(
            "vibe-board-commit-check not found on PATH; workspace creation will return 400"
        );
    }

    let pool = init_pool(&database_url).await?;
    let app = build_router(pool, workspace_root, checker_path);

    let addr: SocketAddr = "127.0.0.1:3000".parse()?;
    tracing::info!("listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

fn resolve_workspace_root() -> PathBuf {
    if let Ok(p) = std::env::var("VIBE_BOARD_WORKSPACE_ROOT") {
        return PathBuf::from(p);
    }
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("vibe-board")
        .join("workspaces")
}
