use std::net::SocketAddr;

use anyhow::Result;
use axum::Server;
use tokio::runtime::Builder;
use tracing::{subscriber, Level};
use tracing_subscriber::FmtSubscriber;

mod app;
mod goout;
mod ical;

pub fn main() -> Result<()> {
    // Configure tracing.
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    subscriber::set_global_default(subscriber)?;

    // Start tokio runtime.
    Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(start_server());
    Ok(())
}

async fn start_server() {
    let app = app::create_router();
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
