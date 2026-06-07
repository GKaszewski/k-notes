mod handlers;

use std::sync::Arc;

use application::worker::WorkerService;
use domain::events::EventHandler;
use handlers::NoteEventHandler;
use wiring::{WiringConfig, build_context};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    init_tracing("worker");

    let wiring_cfg = WiringConfig::from_env()?;
    let ctx = build_context(&wiring_cfg).await?;

    let handlers: Vec<Arc<dyn EventHandler>> = vec![Arc::new(NoteEventHandler::new(ctx.clone()))];

    let consumer = Arc::clone(&ctx.services.event_consumer);
    let worker = WorkerService::new(consumer, handlers);

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        tracing::info!("shutdown signal received");
        let _ = shutdown_tx.send(true);
    });

    tracing::info!("worker started");
    worker.run(shutdown_rx).await;
    tracing::info!("worker stopped");

    Ok(())
}

fn init_tracing(service: &str) {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{service}=info").into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}
