use std::io::{self, Read, Seek, SeekFrom};
use std::sync::Arc;

use songbird::input::core::io::MediaSource;
use tokio::sync::broadcast;

pub struct BroadcastReceiver {
    rx: broadcast::Receiver<Arc<Vec<u8>>>,
    buffer: Vec<u8>,
    pos: usize,
}

impl BroadcastReceiver {
    pub fn new(rx: broadcast::Receiver<Arc<Vec<u8>>>) -> Self {
        Self {
            rx,
            buffer: Vec::new(),
            pos: 0,
        }
    }
}

impl Read for BroadcastReceiver {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        while self.pos >= self.buffer.len() {
            match self.rx.blocking_recv() {
                Ok(chunk) => {
                    self.buffer = (*chunk).clone();
                    self.pos = 0;
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!(skipped = n, "Broadcast receiver lagged, skipping frames");
                    continue;
                }
                Err(broadcast::error::RecvError::Closed) => {
                    return Ok(0);
                }
            }
        }

        let available = &self.buffer[self.pos..];
        let n = available.len().min(buf.len());
        buf[..n].copy_from_slice(&available[..n]);
        self.pos += n;
        Ok(n)
    }
}

impl Seek for BroadcastReceiver {
    fn seek(&mut self, _: SeekFrom) -> io::Result<u64> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Broadcast receiver cannot seek",
        ))
    }
}

impl MediaSource for BroadcastReceiver {
    fn is_seekable(&self) -> bool {
        false
    }

    fn byte_len(&self) -> Option<u64> {
        None
    }
}
