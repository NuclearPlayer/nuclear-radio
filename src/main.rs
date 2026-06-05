mod config;
mod decode;
mod runtime;
mod sinks;
mod source;

use runtime::Runtime;
use sinks::discord::DiscordSink;

fn load_env() {
    dotenvy::from_filename(".env.local").ok();
    dotenvy::from_filename(".env").ok();
}

#[tokio::main]
async fn main() {
    load_env();

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
