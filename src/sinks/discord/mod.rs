mod handlers;

use std::sync::Arc;

use async_trait::async_trait;
use serenity::all::{Context, Guild, Ready, ShardManager};
use serenity::async_trait as serenity_async_trait;
use serenity::prelude::*;
use songbird::{SerenityInit, Songbird};

use crate::config::Config;

use super::{Sink, SinkResult};

struct Handler;

#[serenity_async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        handlers::ready::ready(ctx, ready).await;
    }

    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: Option<bool>) {
        handlers::guild_create::guild_create(ctx, guild, is_new).await;
    }
}

pub struct DiscordSink {
    token: String,
    running: Option<Running>,
}

struct Running {
    voice: Arc<Songbird>,
    shards: Arc<ShardManager>,
}

impl DiscordSink {
    pub fn new(config: &Config) -> Self {
        Self {
            token: config.discord_token.clone(),
            running: None,
        }
    }
}

#[async_trait]
impl Sink for DiscordSink {
    async fn start(&mut self) -> SinkResult {
        let intents = GatewayIntents::GUILDS | GatewayIntents::GUILD_VOICE_STATES;

        let voice = Songbird::serenity();
        let mut client = Client::builder(&self.token, intents)
            .event_handler(Handler)
            .register_songbird_with(voice.clone())
            .await?;

        let shards = client.shard_manager.clone();
        tokio::spawn(async move {
            if let Err(error) = client.start().await {
                eprintln!("discord client error: {error}");
            }
        });

        self.running = Some(Running { voice, shards });
        Ok(())
    }

    async fn cleanup(&mut self) {
        let Some(running) = self.running.take() else {
            return;
        };

        let guild_ids: Vec<_> = running.voice.iter().map(|(guild_id, _)| guild_id).collect();
        for guild_id in guild_ids {
            if let Err(error) = running.voice.remove(guild_id).await {
                eprintln!("Failed to leave voice channel in {guild_id}: {error}");
            }
        }

        running.shards.shutdown_all().await;
    }
}
