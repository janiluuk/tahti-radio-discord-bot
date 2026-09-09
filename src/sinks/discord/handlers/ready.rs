use serenity::all::{Context, Ready};
use tokio::sync::watch;
use tracing::info;

use crate::sinks::discord::commands;
use crate::track::TrackMetadata;

use super::{activity, heartbeat};

pub async fn ready(
    ctx: Context,
    ready: Ready,
    now_playing: watch::Receiver<Option<TrackMetadata>>,
    now_playing_for_heartbeat: watch::Receiver<Option<TrackMetadata>>,
    api_base: Option<String>,
    internal_secret: Option<String>,
) {
    info!(name = %ready.user.name, id = %ready.user.id, "Logged in");
    commands::register(&ctx).await;
    tokio::spawn(activity::sync(ctx.clone(), now_playing));
    tokio::spawn(heartbeat::sync(
        ctx,
        now_playing_for_heartbeat,
        api_base,
        internal_secret,
    ));
}
