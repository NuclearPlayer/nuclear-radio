use serenity::all::{Context, Ready};
use tokio::sync::watch;
use tracing::info;

use crate::track::Track;

use super::activity;

pub async fn ready(ctx: Context, ready: Ready, now_playing: watch::Receiver<Option<Track>>) {
    info!(name = %ready.user.name, id = %ready.user.id, "Logged in");
    tokio::spawn(activity::sync(ctx, now_playing));
}
