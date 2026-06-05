mod handlers;

use serenity::all::{Context, Guild, Ready};
use serenity::async_trait;
use serenity::prelude::*;
use songbird::SerenityInit;

use crate::config::Config;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        handlers::ready::ready(ctx, ready).await;
    }

    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: Option<bool>) {
        handlers::guild_create::guild_create(ctx, guild, is_new).await;
    }
}

pub async fn run(config: &Config) -> Result<(), serenity::Error> {
    let intents = GatewayIntents::GUILDS | GatewayIntents::GUILD_VOICE_STATES;

    let mut client = Client::builder(&config.discord_token, intents)
        .event_handler(Handler)
        .register_songbird()
        .await?;

    client.start().await
}
