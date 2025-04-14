//! '/about' command registration and execution.

use beef::lean::Cow;
use serenity::{all::CommandInteraction, builder::CreateCommand, client::Context};

use crate::PLAYER_MANAGER;
use crate::i18n::{serenity_command_description, serenity_command_name, t_vars};
use crate::utils::constants::HYDROGEN_VERSION;

/// Executes the `/about` command.
pub async fn execute<'a>(context: &Context, interaction: &CommandInteraction) -> Cow<'a, str> {
    let guild_count = context.cache.guild_count().to_string();

    let player_count = PLAYER_MANAGER
        .get()
        .map(|i| i.get_player_count())
        .unwrap_or_default()
        .to_string();

    t_vars(
        &interaction.locale,
        "about.result",
        [HYDROGEN_VERSION, &guild_count, &player_count],
    )
}

/// Creates the `/join` [CreateCommand].
pub fn create_command() -> CreateCommand {
    let mut command = CreateCommand::new("about");

    command = serenity_command_name("about.name", command);
    command = serenity_command_description("about.description", command);

    command
        .description("Shows information about the bot.")
        .dm_permission(true)
}
