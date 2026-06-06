use std::process::Stdio;

use tokio::process::Command;

use crate::track::Track;

#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    #[error("failed to run yt-dlp: {0}")]
    Spawn(#[from] std::io::Error),

    #[error("yt-dlp exited unsuccessfully: {0}")]
    Failed(String),

    #[error("yt-dlp output was missing expected fields")]
    MissingFields,
}

pub async fn resolve(youtube_url: &str) -> Result<Track, SourceError> {
    let output = Command::new("yt-dlp")
        .args([
            "-f", "bestaudio",
            "--print", "%(title)s",
            "--print", "%(artist)s",
            "--print", "%(track)s",
            "--print", "%(duration)s",
            "--print", "%(thumbnail)s",
            "-g",
            youtube_url,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        return Err(SourceError::Failed(stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut lines = stdout.lines();

    Track::parse(youtube_url, &mut lines)
        .ok_or(SourceError::MissingFields)
}
