use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::{Arc, Condvar, Mutex};

use songbird::input::core::io::MediaSource;

const BUFFER_SIZE: usize = 16 * 1024 * 1024;

#[derive(Clone)]
pub struct AudioStream {
    inner: Arc<(Mutex<Vec<u8>>, Condvar)>,
}

impl AudioStream {
    pub fn new() -> Self {
        Self {
            inner: Arc::new((Mutex::new(Vec::new()), Condvar::new())),
        }
    }
}

impl Read for AudioStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let (mutex, condvar) = &*self.inner;
        let mut buffer = mutex.lock().expect("Mutex was poisoned");

        if buffer.is_empty() {
            buf.fill(0);
            condvar.notify_all();
            return Ok(buf.len());
        }

        let n = buf.len().min(buffer.len());
        buf[..n].copy_from_slice(&buffer[..n]);
        buffer.drain(..n);
        condvar.notify_all();

        Ok(n)
    }
}

impl Write for AudioStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let (mutex, condvar) = &*self.inner;
        let mut buffer = mutex.lock().expect("Mutex was poisoned");

        while buffer.len() + buf.len() > BUFFER_SIZE {
            buffer = condvar.wait(buffer).expect("Mutex was poisoned");
        }

        buffer.extend_from_slice(buf);
        condvar.notify_all();

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let (mutex, condvar) = &*self.inner;
        let mut buffer = mutex.lock().expect("Mutex was poisoned");
        buffer.clear();
        condvar.notify_all();
        Ok(())
    }
}

impl Seek for AudioStream {
    fn seek(&mut self, _: SeekFrom) -> std::io::Result<u64> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "AudioStream cannot seek",
        ))
    }
}

impl MediaSource for AudioStream {
    fn is_seekable(&self) -> bool {
        false
    }

    fn byte_len(&self) -> Option<u64> {
        None
    }
}
