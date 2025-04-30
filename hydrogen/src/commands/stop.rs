//! '/stop' command registration and execution.

use beef::lean::Cow;
use serenity::{all::CommandInteraction, builder::CreateCommand, client::Context};

use crate::shared::SharedInteraction;
use crate::{
    i18n::{serenity_command_description, serenity_command_name},
    shared,
};

/// Executes the `/stop` command.
pub async fn execute<'a>(context: &Context, interaction: &CommandInteraction) -> Cow<'a, str> {
    shared::stop::execute(context, &SharedInteraction::Command(interaction)).await
}

/// Creates the `/stop` [CreateCommand].
pub fn create_command() -> CreateCommand {
    let mut command = CreateCommand::new("stop");

    command = serenity_command_name("stop.name", command);
    command = serenity_command_description("stop.description", command);

    command
        .description("Stops the player.")
        .dm_permission(false)
}
