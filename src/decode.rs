use std::io::{self, BufReader, Read};
use std::process::{Child, ChildStdout, Command, Stdio};

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("failed to spawn ffmpeg: {0}")]
    Spawn(#[from] io::Error),
}

pub struct PcmSource {
    child: Child,
    stdout: BufReader<ChildStdout>,
}

impl PcmSource {
    pub fn spawn(url: &str) -> Result<Self, DecodeError> {
        let mut child = Command::new("ffmpeg")
            .args([
                "-nostdin",
                "-i",
                url,
                "-f",
                "f32le",
                "-ar",
                "48000",
                "-ac",
                "2",
                "-loglevel",
                "error",
                "pipe:1",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;

        let stdout = child.stdout.take().expect("stdout was piped");
        Ok(Self {
            child,
            stdout: BufReader::new(stdout),
        })
    }
}

impl Read for PcmSource {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stdout.read(buf)
    }
}

impl Drop for PcmSource {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
