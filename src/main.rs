use anyhow::Result;

use tokio::signal;
use tracing::error;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;

mod server;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    // let (console, server) = console_subscriber::ConsoleLayer::builder().build();
    // tokio::spawn(async move {
    //     server.serve().await.unwrap();
    // });

    Registry::default()
        .with(EnvFilter::from_default_env())
        .with(
            HierarchicalLayer::new(1)
                .with_targets(true)
                .with_bracketed_fields(true),
        )
        .init();

    tokio::spawn(server::run_server());

    match signal::ctrl_c().await {
        Ok(()) => {}
        Err(err) => {
            error!("Unable to listen for shutdown signal: {}", err);
        }
    }

    Ok(())
}
