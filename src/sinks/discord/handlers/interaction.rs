use serenity::all::{Context, Interaction};
use tracing::warn;

use crate::sinks::discord::commands;

pub async fn interaction_create(ctx: Context, interaction: Interaction) {
    let Interaction::Command(command) = interaction else {
        return;
    };

    match command.data.name.as_str() {
        "ping" => commands::ping::run(&command, &ctx).await,
        name => warn!(name, "Unknown command"),
    }
}
