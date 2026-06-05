mod broadcast;
mod config;
mod decode;
mod playlist;
mod receiver;
mod runtime;
mod sinks;
mod source;

use std::process::ExitCode;
use std::sync::Arc;

use broadcast::Broadcast;
use runtime::Runtime;
use sinks::discord::DiscordSink;
use tracing_subscriber::EnvFilter;

fn load_env() {
    dotenvy::from_filename(".env.local").ok();
    dotenvy::from_filename(".env").ok();
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("nuclear_radio=debug,songbird=debug,serenity=warn"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

#[tokio::main]
async fn main() -> ExitCode {
    load_env();
    init_tracing();

    let config = match config::load() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("Invalid configuration: {error}");
            return ExitCode::FAILURE;
        }
    };

    let playlist = match playlist::load() {
        Ok(playlist) => playlist,
        Err(error) => {
            eprintln!("Invalid playlist: {error}");
            return ExitCode::FAILURE;
        }
    };

    let broadcast = Arc::new(Broadcast::new(playlist));
    Arc::clone(&broadcast).spawn();

    let mut runtime = Runtime::new();
    runtime.add(DiscordSink::new(&config, broadcast));

    if let Err(error) = runtime.run().await {
        eprintln!("Runtime error: {error}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
