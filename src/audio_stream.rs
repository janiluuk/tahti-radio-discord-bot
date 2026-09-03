use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};

use songbird::input::core::io::MediaSource;

const BUFFER_SIZE: usize = 16 * 1024 * 1024;

struct Inner {
    buffer: Mutex<Vec<u8>>,
    condvar: Condvar,
    closed: AtomicBool,
}

#[derive(Clone)]
pub struct AudioStream {
    inner: Arc<Inner>,
}

impl AudioStream {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Inner {
                buffer: Mutex::new(Vec::new()),
                condvar: Condvar::new(),
                closed: AtomicBool::new(false),
            }),
        }
    }

    pub fn close(&self) {
        self.inner.closed.store(true, Ordering::Relaxed);
        self.inner.condvar.notify_all();
    }
}

impl Read for AudioStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let inner = &*self.inner;
        let mut buffer = inner.buffer.lock().expect("Mutex was poisoned");

        if buffer.is_empty() {
            if inner.closed.load(Ordering::Relaxed) {
                return Ok(0);
            }

            buf.fill(0);
            inner.condvar.notify_all();
            return Ok(buf.len());
        }

        let n = buf.len().min(buffer.len());
        buf[..n].copy_from_slice(&buffer[..n]);
        buffer.drain(..n);
        inner.condvar.notify_all();

        Ok(n)
    }
}

impl Write for AudioStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let inner = &*self.inner;
        let mut buffer = inner.buffer.lock().expect("Mutex was poisoned");

        while buffer.len() + buf.len() > BUFFER_SIZE {
            if inner.closed.load(Ordering::Relaxed) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::BrokenPipe,
                    "AudioStream closed",
                ));
            }
            buffer = inner.condvar.wait(buffer).expect("Mutex was poisoned");
        }

        buffer.extend_from_slice(buf);
        inner.condvar.notify_all();

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let inner = &*self.inner;
        let mut buffer = inner.buffer.lock().expect("Mutex was poisoned");
        buffer.clear();
        inner.condvar.notify_all();
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
