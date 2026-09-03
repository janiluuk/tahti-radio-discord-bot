use serenity::all::{Context, Ready};
use tokio::sync::watch;
use tracing::info;

use crate::sinks::discord::commands;
use crate::track::TrackMetadata;

use super::activity;

pub async fn ready(ctx: Context, ready: Ready, now_playing: watch::Receiver<Option<TrackMetadata>>) {
    info!(name = %ready.user.name, id = %ready.user.id, "Logged in");
    commands::register(&ctx).await;
    tokio::spawn(activity::sync(ctx, now_playing));
}
