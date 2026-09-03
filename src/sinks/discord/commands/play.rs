use serenity::all::{CommandInteraction, CommandOptionType, Context, ResolvedValue};
use serenity::builder::{
    CreateCommand, CreateCommandOption, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use tracing::{error, info};

use crate::broadcast::Queue;
use crate::source;

pub fn register() -> CreateCommand {
    CreateCommand::new("play").description("Add a track to the queue").add_option(
        CreateCommandOption::new(CommandOptionType::String, "url", "YouTube URL")
            .required(true),
    )
}

pub async fn run(command: &CommandInteraction, ctx: &Context, queue: Queue) {
    let Some(url) = extract_url(command) else {
        respond(command, ctx, "Missing URL").await;
        return;
    };

    respond(command, ctx, &format!("Queued <{url}>")).await;

    tokio::spawn(async move {
        match source::resolve_metadata(&url).await {
            Ok(metadata) => {
                info!(track = %metadata, url, "Queued track");
                queue.lock().await.push_back(metadata);
            }
            Err(error) => {
                error!(%error, url, "Failed to resolve queued track");
            }
        }
    });
}

fn extract_url(command: &CommandInteraction) -> Option<String> {
    command.data.options().into_iter().find_map(|option| {
        if option.name != "url" {
            return None;
        }
        match option.value {
            ResolvedValue::String(value) => Some(value.trim().to_owned()),
            _ => None,
        }
    })
}

async fn respond(command: &CommandInteraction, ctx: &Context, content: &str) {
    let response = CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .content(content)
            .ephemeral(true),
    );

    if let Err(error) = command.create_response(&ctx.http, response).await {
        error!(%error, "Failed to respond to play command");
    }
}
