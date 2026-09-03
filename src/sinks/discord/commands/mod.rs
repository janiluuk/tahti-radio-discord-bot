pub mod play;
pub mod queue;

use serenity::all::Context;
use serenity::builder::CreateCommand;
use serenity::model::application::Command;
use tracing::{error, info};

fn all() -> Vec<CreateCommand> {
    vec![play::register(), queue::register()]
}

pub async fn register(ctx: &Context) {
    match Command::set_global_commands(&ctx.http, all()).await {
        Ok(cmds) => info!(count = cmds.len(), "Registered slash commands"),
        Err(error) => error!(%error, "Failed to register slash commands"),
    }
}
