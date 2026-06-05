use serenity::async_trait;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use crate::config::Config;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("logged in as {} (id={})", ready.user.name, ready.user.id);
    }
}

pub async fn run(config: &Config) -> Result<(), serenity::Error> {
    let intents = GatewayIntents::GUILDS | GatewayIntents::GUILD_VOICE_STATES;

    let mut client = Client::builder(&config.discord_token, intents)
        .event_handler(Handler)
        .await?;

    client.start().await
}
