//! '/skip' command registration and execution.

use beef::lean::Cow;
use serenity::{all::CommandInteraction, builder::CreateCommand, client::Context};

use crate::shared::SharedInteraction;
use crate::{
    i18n::{serenity_command_description, serenity_command_name},
    shared,
};

/// Executes the `/skip` command.
pub async fn execute<'a>(context: &Context, interaction: &CommandInteraction) -> Cow<'a, str> {
    shared::skip::execute(context, &SharedInteraction::Command(interaction)).await
}

/// Creates the `/skip` [CreateCommand].
pub fn create_command() -> CreateCommand {
    let mut command = CreateCommand::new("skip");

    command = serenity_command_name("skip.name", command);
    command = serenity_command_description("skip.description", command);

    command
        .description("Skips to the next song in the queue.")
        .dm_permission(false)
}
