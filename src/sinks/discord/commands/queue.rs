use std::fmt::Write;

use serenity::all::{CommandInteraction, Context};
use serenity::builder::{
    CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use tracing::error;

use crate::broadcast::Queue;

pub fn register() -> CreateCommand {
    CreateCommand::new("queue").description("Show queued tracks")
}

pub async fn run(command: &CommandInteraction, ctx: &Context, queue: Queue) {
    let items = queue.lock().await;

    let content = if items.is_empty() {
        "The queue is empty.".to_owned()
    } else {
        items.iter().enumerate().fold(String::new(), |mut buf, (i, track)| {
            let _ = writeln!(buf, "{}. {track}", i + 1);
            buf
        })
    };

    let response = CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .content(content)
            .ephemeral(true),
    );

    if let Err(error) = command.create_response(&ctx.http, response).await {
        error!(%error, "Failed to respond to queue command");
    }
}
