use std::process::Stdio;

use tokio::process::Command;

#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    #[error("failed to run yt-dlp: {0}")]
    Spawn(#[from] std::io::Error),

    #[error("yt-dlp exited unsuccessfully: {0}")]
    Failed(String),

    #[error("yt-dlp produced no stream URL")]
    NoUrl,
}

pub async fn resolve_stream_url(youtube_url: &str) -> Result<String, SourceError> {
    let output = Command::new("yt-dlp")
        .args(["-f", "bestaudio", "-g", youtube_url])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        return Err(SourceError::Failed(stderr));
    }

    let url = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .ok_or(SourceError::NoUrl)?;

    Ok(url)
}
