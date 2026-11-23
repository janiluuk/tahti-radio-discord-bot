use tokio::sync::mpsc;

use crate::audio_stream::AudioStream;
use crate::sinks::{Sink, SinkResult};

pub struct Runtime {
    stream: AudioStream,
    sinks: Vec<Box<dyn Sink>>,
}

impl Runtime {
    pub fn new(stream: AudioStream) -> Self {
        Self {
            stream,
            sinks: Vec::new(),
        }
    }

    pub fn add(&mut self, sink: impl Sink + 'static) -> &mut Self {
        self.sinks.push(Box::new(sink));
        self
    }

    pub async fn run(mut self) -> SinkResult {
        let (fatal_tx, mut fatal_rx) = mpsc::unbounded_channel::<String>();
        for sink in self.sinks.iter_mut() {
            sink.start(fatal_tx.clone()).await?;
        }
        drop(fatal_tx);

        let failure = tokio::select! {
            signal = shutdown_signal() => {
                if let Err(error) = signal {
                    eprintln!("Failed to listen for shutdown signal: {error}");
                }
                None
            }
            Some(reason) = fatal_rx.recv() => Some(reason),
        };

        self.stream.close();

        for sink in self.sinks.iter_mut() {
            sink.cleanup().await;
        }

        match failure {
            Some(reason) => Err(reason.into()),
            None => Ok(()),
        }
    }
}

/// Ctrl+C locally, SIGTERM from `docker stop` / compose in the stack.
async fn shutdown_signal() -> std::io::Result<()> {
    #[cfg(unix)]
    {
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
        tokio::select! {
            result = tokio::signal::ctrl_c() => result,
            _ = terminate.recv() => Ok(()),
        }
    }
    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c().await
    }
}
