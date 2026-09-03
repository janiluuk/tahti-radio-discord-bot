use serenity::all::{Context, Interaction};
use tracing::warn;

use crate::broadcast::Queue;
use crate::sinks::discord::commands;

pub async fn interaction_create(ctx: Context, interaction: Interaction, queue: Queue) {
    let Interaction::Command(command) = interaction else {
        return;
    };

    match command.data.name.as_str() {
        "play" => commands::play::run(&command, &ctx, queue).await,
        "queue" => commands::queue::run(&command, &ctx, queue).await,
        name => warn!(name, "Unknown command"),
    }
}
