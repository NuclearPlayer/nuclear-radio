use serenity::all::{ChannelId, ChannelType, Context, CreateChannel, Guild};
use songbird::input::{Input, RawAdapter};

use crate::{decode, source};

const CHANNEL_NAME: &str = "Nuclear Radio";

const TEST_TRACK_URL: &str = "https://youtu.be/iqomTAiRnVM";

pub async fn guild_create(ctx: Context, guild: Guild, _is_new: Option<bool>) {
    let channel_id = match find_radio_channel(&guild) {
        Some(id) => id,
        None => match create_radio_channel(&ctx, &guild).await {
            Ok(id) => id,
            Err(error) => {
                eprintln!("Failed to create radio channel in {}: {error}", guild.id);
                return;
            }
        },
    };

    let manager = songbird::get(&ctx)
        .await
        .expect("Songbird should be registered at startup");

    let call = match manager.join(guild.id, channel_id).await {
        Ok(call) => call,
        Err(error) => {
            eprintln!("Failed to join voice channel in {}: {error}", guild.id);
            return;
        }
    };

    match build_input(TEST_TRACK_URL).await {
        Ok(input) => {
            call.lock().await.play_input(input);
        }
        Err(error) => {
            eprintln!("Failed to start playback in {}: {error}", guild.id);
        }
    }
}

async fn build_input(youtube_url: &str) -> Result<Input, Box<dyn std::error::Error + Send + Sync>> {
    let stream_url = source::resolve_stream_url(youtube_url).await?;
    let pcm = decode::PcmSource::spawn(&stream_url)?;
    Ok(RawAdapter::new(pcm, 48000, 2).into())
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
