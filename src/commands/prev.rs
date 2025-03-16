//! '/previous' command registration and execution.

use beef::lean::Cow;
use serenity::{all::CommandInteraction, builder::CreateCommand, client::Context};

use crate::shared::SharedInteraction;
use crate::{
    i18n::{serenity_command_description, serenity_command_name},
    shared,
};

/// Executes the `/previous` command.
pub async fn execute<'a>(context: &Context, interaction: &CommandInteraction) -> Cow<'a, str> {
    shared::prev::execute(context, &SharedInteraction::Command(interaction)).await
}

/// Creates the `/previous` [CreateCommand].
pub fn create_command() -> CreateCommand {
    let mut command = CreateCommand::new("previous");

    command = serenity_command_name("prev.name", command);
    command = serenity_command_description("prev.description", command);

    command
        .description("Plays the previous song in the queue.")
        .dm_permission(false)
}
