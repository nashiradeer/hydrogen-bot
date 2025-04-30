//! '/shuffle' command registration and execution.

use beef::lean::Cow;
use serenity::{all::CommandInteraction, builder::CreateCommand, client::Context};

use crate::shared::SharedInteraction;
use crate::{
    i18n::{serenity_command_description, serenity_command_name},
    shared,
};

/// Executes the `/shuffle` command.
pub async fn execute<'a>(context: &Context, interaction: &CommandInteraction) -> Cow<'a, str> {
    shared::shuffle::execute(context, &SharedInteraction::Command(interaction)).await
}

/// Creates the `/shuffle` [CreateCommand].
pub fn create_command() -> CreateCommand {
    let mut command = CreateCommand::new("shuffle");

    command = serenity_command_name("shuffle.name", command);
    command = serenity_command_description("shuffle.description", command);

    command
        .description("Shuffle the player queue.")
        .dm_permission(false)
}
