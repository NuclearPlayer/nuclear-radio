use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;

use rand::seq::IndexedRandom;
use tokio::sync::{watch, Mutex};
use tokio::task::JoinHandle;
use tracing::{error, info};

use crate::audio_stream::AudioStream;
use crate::track::TrackMetadata;
use crate::{decode, source};

pub type Queue = Arc<Mutex<VecDeque<TrackMetadata>>>;

pub struct Broadcast {
    playlist: Vec<String>,
    stream: AudioStream,
    now_playing: watch::Sender<Option<TrackMetadata>>,
    queue: Queue,
}

impl Broadcast {
    pub fn new(playlist: Vec<String>, stream: AudioStream) -> Self {
        let (now_playing, _) = watch::channel(None);
        Self {
            playlist,
            stream,
            now_playing,
            queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn subscribe(&self) -> watch::Receiver<Option<TrackMetadata>> {
        self.now_playing.subscribe()
    }

    pub fn queue(&self) -> Queue {
        Arc::clone(&self.queue)
    }

    pub fn spawn(self: Arc<Self>) -> JoinHandle<()> {
        tokio::spawn(async move {
            self.run().await;
        })
    }

    async fn run(&self) {
        loop {
            let url = self.next_url().await;

            if let Err(error) = self.play_track(&url).await {
                error!(%error, url, "Failed to play track, skipping");
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }

    async fn next_url(&self) -> String {
        self.queue
            .lock()
            .await
            .pop_front()
            .map(|track| track.youtube_url)
            .unwrap_or_else(|| {
                self.playlist
                    .choose(&mut rand::rng())
                    .expect("Playlist should be non-empty")
                    .clone()
            })
    }

    async fn play_track(
        &self,
        youtube_url: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let track = source::resolve(youtube_url).await?;

        info!(track = %track, "Now playing");

        let mut pcm = decode::PcmSource::spawn(&track.stream_url)?;
        let _ = self.now_playing.send(Some(track.metadata));

        tokio::task::spawn_blocking({
            let mut stream = self.stream.clone();
            move || {
                use std::io::{Read, Write};
                let mut buf = vec![0u8; 960 * 2 * 4];
                loop {
                    match pcm.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            if stream.write_all(&buf[..n]).is_err() {
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
