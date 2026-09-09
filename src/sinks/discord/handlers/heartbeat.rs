use std::time::{Duration, Instant};

use serde::Serialize;
use serenity::all::Context;
use tokio::sync::watch;
use tokio::time;

use crate::track::TrackMetadata;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(20);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HeartbeatPayload {
    guild_count: u64,
    uptime_secs: u64,
    current_track: Option<String>,
}

pub async fn sync(
    ctx: Context,
    now_playing: watch::Receiver<Option<TrackMetadata>>,
    api_base: Option<String>,
    internal_secret: Option<String>,
) {
    let (Some(base), Some(secret)) = (api_base, internal_secret) else {
        tracing::debug!("heartbeat disabled: TAHTI_API_BASE/INTERNAL_SECRET not set");
        return;
    };

    let url = format!(
        "{}/api/v1/internal/discord-bot/heartbeat",
        base.trim_end_matches('/')
    );
    let client = reqwest::Client::new();
    let started_at = Instant::now();
    let mut interval = time::interval(HEARTBEAT_INTERVAL);
    let mut warned = false;

    loop {
        interval.tick().await;

        let payload = HeartbeatPayload {
            guild_count: ctx.cache.guild_count() as u64,
            uptime_secs: started_at.elapsed().as_secs(),
            current_track: now_playing.borrow().as_ref().map(|track| track.to_string()),
        };

        match client
            .post(&url)
            .bearer_auth(&secret)
            .json(&payload)
            .timeout(REQUEST_TIMEOUT)
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                if warned {
                    tracing::info!("heartbeat recovered");
                    warned = false;
                }
            }
            Ok(response) => {
                if !warned {
                    tracing::warn!(status = %response.status(), "heartbeat rejected by API");
                    warned = true;
                }
            }
            Err(error) => {
                if !warned {
                    tracing::warn!(%error, "heartbeat request failed");
                    warned = true;
                }
            }
        }
    }
}
