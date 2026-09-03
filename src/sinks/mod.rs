pub mod discord;

use async_trait::async_trait;

pub type SinkResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[async_trait]
pub trait Sink: Send {
    async fn start(&mut self) -> SinkResult;
    async fn cleanup(&mut self);
}
