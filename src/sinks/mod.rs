pub mod discord;

use async_trait::async_trait;

pub type SinkResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

/// A sink reports here when it can no longer run (e.g. the Discord gateway
/// rejected the token), so the runtime shuts down and exits non-zero and the
/// container's restart policy can bring the bot back.
pub type FatalSender = tokio::sync::mpsc::UnboundedSender<String>;

#[async_trait]
pub trait Sink: Send {
    async fn start(&mut self, fatal: FatalSender) -> SinkResult;
    async fn cleanup(&mut self);
}
