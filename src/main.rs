mod source;

fn load_env() {
    dotenvy::from_filename(".env.local").ok();
    dotenvy::from_filename(".env").ok();
}

#[tokio::main]
async fn main() {
    load_env();

    let url = std::env::args()
        .nth(1)
        .expect("usage: nuclear-radio <youtube-url>");

    match source::resolve_stream_url(&url).await {
        Ok(stream_url) => println!("{stream_url}"),
        Err(error) => {
            eprintln!("error: {error}");
            std::process::exit(1);
        }
    }
}
