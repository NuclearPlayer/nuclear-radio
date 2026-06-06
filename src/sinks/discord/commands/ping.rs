use serenity::all::{CommandInteraction, Context};
use serenity::builder::{CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage};
use tracing::error;

pub fn register() -> CreateCommand {
    CreateCommand::new("ping").description("pong")
}

pub async fn run(command: &CommandInteraction, ctx: &Context) {
    let response = CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .content("pong")
            .ephemeral(true),
    );

    if let Err(error) = command.create_response(&ctx.http, response).await {
        error!(%error, "Failed to respond to ping command");
    }
}
