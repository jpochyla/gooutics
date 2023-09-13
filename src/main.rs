use std::net::SocketAddr;

use anyhow::Result;
use axum::Server;
use server::get_events;
use tokio::runtime::Builder;
use tracing::{subscriber, Level};
use tracing_subscriber::FmtSubscriber;

mod goout;
mod ical;
mod server;

pub fn main() -> Result<()> {
    let flags = xflags::parse_or_exit! {
        /// Print debug logs
        optional -d,--debug
        /// Lookup venue events and exit instead of starting a server
        optional -v,--venue venue_id: String
        /// Override language for the venue lookup
        optional -l,--language language: String
    };

    // Configure tracing.
    let subscriber = FmtSubscriber::builder()
        .with_max_level(if flags.debug {
            Level::DEBUG
        } else {
            Level::INFO
        })
        .finish();
    subscriber::set_global_default(subscriber)?;

    // Start tokio runtime.
    let runtime = Builder::new_multi_thread().enable_all().build()?;

    // Either start the server or performa a command.
    if let Some(venue_id) = flags.venue {
        let language = flags.language.as_deref().unwrap_or("en");
        runtime.block_on(lookup_venue(language, &venue_id));
    } else {
        runtime.block_on(start_server());
    }
    Ok(())
}

async fn lookup_venue(language: &str, short_id: &str) {
    let cal = get_events(language, short_id).await.unwrap();
    println!("{cal}");
}

async fn start_server() {
    let app = server::create_router();
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
