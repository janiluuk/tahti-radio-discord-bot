use serenity::all::Context;
use serenity::gateway::ActivityData;
use tokio::sync::watch;

use crate::track::TrackMetadata;

pub async fn sync(ctx: Context, mut now_playing: watch::Receiver<Option<TrackMetadata>>) {
    while now_playing.changed().await.is_ok() {
        let activity = now_playing
            .borrow_and_update()
            .as_ref()
            .map(activity_for_track);
        ctx.set_activity(activity);
    }
}

fn activity_for_track(track: &TrackMetadata) -> ActivityData {
    let mut activity = ActivityData::listening(track.to_string());
    activity.state = Some(track.youtube_url.clone());
    activity
}
