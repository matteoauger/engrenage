mod config;
mod error;
mod routes;
mod state;

use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;

use crate::state::AppState;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Path to the config file (default: ./engrenage.toml if present, else built-in defaults)
    #[arg(long)]
    config: Option<PathBuf>,

    /// Listen address override (e.g. 0.0.0.0:8080)
    #[arg(long)]
    listen: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let args = Args::parse();
    let mut config = config::Config::load(args.config.as_deref()).context("loading config")?;
    if let Some(listen) = args.listen {
        config.override_listen_addr(listen);
    }
    config.validate().context("invalid configuration")?;

    if !routes::assets::has_index() {
        tracing::warn!(
            "SPA assets not built — run `npm run build -w web` (dev: use the Vite dev server instead)"
        );
    }

    std::fs::create_dir_all(&config.data_dir)
        .with_context(|| format!("creating data dir {}", config.data_dir.display()))?;

    tracing::info!(
        listen_addr = %config.listen_addr,
        data_dir = %config.data_dir.display(),
        "starting engrenage"
    );

    let listener = tokio::net::TcpListener::bind(&config.listen_addr).await?;
    let app = routes::build_router(AppState::new(config));

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("shutdown complete");
    Ok(())
}

fn init_tracing() {
    use tracing_subscriber::EnvFilter;

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("install ctrl_c handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("shutdown signal received");
}
