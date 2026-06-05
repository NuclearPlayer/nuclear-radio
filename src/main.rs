mod config;
mod sinks;
mod source;

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

    if let Err(error) = sinks::discord::run(&config).await {
        eprintln!("Discord client error: {error}");
        std::process::exit(1);
    }
}
