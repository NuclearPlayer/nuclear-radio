mod config;
mod decode;
mod runtime;
mod sinks;
mod source;

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
async fn main() {
    load_env();
    init_tracing();

    let config = match config::load() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("Invalid configuration: {error}");
            std::process::exit(1);
        }
    };

    let mut runtime = Runtime::new();
    runtime.add(DiscordSink::new(&config));

    if let Err(error) = runtime.run().await {
        eprintln!("Runtime error: {error}");
        std::process::exit(1);
    }
}
