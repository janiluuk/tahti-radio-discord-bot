use serenity::all::{ChannelId, ChannelType, Context, CreateChannel, Guild};
use songbird::input::{Input, RawAdapter};
use tracing::error;

use crate::audio_stream::AudioStream;

const CHANNEL_NAME: &str = "Tahti Radio";

pub async fn guild_create(ctx: Context, guild: Guild, _is_new: Option<bool>, stream: &AudioStream) {
    let channel_id = match find_radio_channel(&guild) {
        Some(id) => id,
        None => match create_radio_channel(&ctx, &guild).await {
            Ok(id) => id,
            Err(err) => {
                error!(%err, guild = %guild.id, "Failed to create radio channel");
                return;
            }
        },
    };

    let manager = songbird::get(&ctx)
        .await
        .expect("Songbird should be registered at startup");

    let call = match manager.join(guild.id, channel_id).await {
        Ok(call) => call,
        Err(err) => {
            error!(%err, guild = %guild.id, "Failed to join voice channel");
            return;
        }
    };

    let input: Input = RawAdapter::new(stream.clone(), 48000, 2).into();
    call.lock().await.play_input(input);
}

fn find_radio_channel(guild: &Guild) -> Option<ChannelId> {
    guild
        .channels
        .values()
        .find(|channel| channel.kind == ChannelType::Voice && channel.name == CHANNEL_NAME)
        .map(|channel| channel.id)
}

async fn create_radio_channel(ctx: &Context, guild: &Guild) -> Result<ChannelId, serenity::Error> {
    let builder = CreateChannel::new(CHANNEL_NAME).kind(ChannelType::Voice);
    let channel = guild.id.create_channel(&ctx.http, builder).await?;
    Ok(channel.id)
}
