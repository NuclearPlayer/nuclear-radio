use serenity::all::{Context, Ready};
use tracing::info;

pub async fn ready(_ctx: Context, ready: Ready) {
    info!(name = %ready.user.name, id = %ready.user.id, "Logged in");
}
