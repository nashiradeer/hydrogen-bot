//! '/pause' command registration and execution.

use beef::lean::Cow;
use serenity::{all::CommandInteraction, builder::CreateCommand, client::Context};

use crate::shared::SharedInteraction;
use crate::{
    i18n::{serenity_command_description, serenity_command_name},
    shared,
};

/// Executes the `/pause` command.
pub async fn execute<'a>(context: &Context, interaction: &CommandInteraction) -> Cow<'a, str> {
    shared::pause::execute(context, &SharedInteraction::Command(interaction)).await
}

/// Creates the `/pause` [CreateCommand].
pub fn create_command() -> CreateCommand {
    let mut command = CreateCommand::new("pause");

    command = serenity_command_name("pause.name", command);
    command = serenity_command_description("pause.description", command);

    command
        .description("Pauses or resumes the player.")
        .dm_permission(false)
}
