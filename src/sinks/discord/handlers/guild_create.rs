use serenity::all::{ChannelId, ChannelType, Context, CreateChannel, Guild};

const CHANNEL_NAME: &str = "Nuclear Radio";

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
        .expect("Songbird registered at startup");

    if let Err(error) = manager.join(guild.id, channel_id).await {
        eprintln!("Failed to join voice channel in {}: {error}", guild.id);
    }
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
