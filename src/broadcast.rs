use std::sync::Arc;
use std::time::Duration;

use rand::seq::IndexedRandom;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tracing::{error, info};

use crate::{decode, source};

const CHUNK_SIZE: usize = 960 * 2 * 4;

pub struct Broadcast {
    playlist: Vec<String>,
    sender: broadcast::Sender<Arc<Vec<u8>>>,
}

impl Broadcast {
    pub fn new(playlist: Vec<String>) -> Self {
        let (sender, _) = broadcast::channel(64);
        Self { playlist, sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Arc<Vec<u8>>> {
        self.sender.subscribe()
    }

    pub fn spawn(self: Arc<Self>) -> JoinHandle<()> {
        tokio::spawn(async move {
            self.run().await;
        })
    }

    async fn run(&self) {
        loop {
            let url = self
                .playlist
                .choose(&mut rand::rng())
                .expect("Playlist should be non-empty");

            info!(url, "Now playing");

            if let Err(error) = self.play_track(url).await {
                error!(%error, url, "Failed to play track, skipping");
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }

    async fn S(&self, youtube_url: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let stream_url = source::resolve_stream_url(youtube_url).await?;
        let mut pcm = decode::PcmSource::spawn(&stream_url)?;

        tokio::task::spawn_blocking({
            let sender = self.sender.clone();
            move || {
                use std::io::Read;
                let mut buf = vec![0u8; CHUNK_SIZE];
                loop {
                    match pcm.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            let chunk = Arc::new(buf[..n].to_vec());
                            if sender.send(chunk).is_err() {
                                break;
                            }
                        }
                        Err(error) => {
                            error!(%error, "Read error from decoder");
                            break;
                        }
                    }
                }
            }
        })
        .await?;

        Ok(())
    }
}
