use std::io::{self, BufReader, Read, Seek, SeekFrom};
use std::process::{Child, ChildStdout, Command, Stdio};

use songbird::input::core::io::MediaSource;

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

impl Seek for PcmSource {
    fn seek(&mut self, _: SeekFrom) -> io::Result<u64> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "PcmSource is a live stream and cannot seek",
        ))
    }
}

impl MediaSource for PcmSource {
    fn is_seekable(&self) -> bool {
        false
    }

    fn byte_len(&self) -> Option<u64> {
        None
    }
}

impl Drop for PcmSource {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
