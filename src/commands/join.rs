//! '/join' command registration and execution.

use beef::lean::Cow;
use serenity::{all::CommandInteraction, builder::CreateCommand, client::Context};
use tracing::{event, Level};

use crate::i18n::t;
use crate::{
    i18n::{serenity_command_description, serenity_command_name, t_vars},
    LOADED_COMMANDS, PLAYER_MANAGER,
};

/// Executes the `/join` command.
pub async fn execute<'a>(context: &Context, interaction: &CommandInteraction) -> Cow<'a, str> {
    let Some(guild_id) = interaction.guild_id else {
        event!(Level::WARN, "interaction.guild_id is None");
        return Cow::borrowed(t(&interaction.locale, "error.not_in_guild"));
    };

    let Some(manager) = PLAYER_MANAGER.get() else {
        event!(Level::ERROR, "PLAYER_MANAGER.get() returned None");
        return Cow::borrowed(t(&interaction.locale, "error.unknown"));
    };

    if manager.contains_player(guild_id) {
        event!(Level::INFO, "player already exists");
        return Cow::borrowed(t(&interaction.locale, "error.player_exists"));
    }

    let Some(voice_channel_id) = context.cache.guild(guild_id).and_then(|guild| {
        guild
            .voice_states
            .get(&interaction.user.id)
            .and_then(|voice_state| voice_state.channel_id)
    }) else {
        event!(Level::INFO, "user voice state is None");
        return Cow::borrowed(t(&interaction.locale, "error.unknown_voice_state"));
    };

    let Some(voice_manager) = songbird::get(context).await else {
        event!(Level::ERROR, "songbird::get() returned None");
        return Cow::borrowed(t(&interaction.locale, "error.unknown"));
    };

    if let Err(e) = voice_manager.join_gateway(guild_id, voice_channel_id).await {
        event!(Level::INFO, voice_channel_id = %voice_channel_id, error = %e, "cannot join the voice channel");
        return Cow::borrowed(t(&interaction.locale, "error.cant_connect"));
    }

    // Initialize the player.
    if let Err(e) = manager
        .init(
            guild_id,
            interaction.channel_id,
            &interaction
                .guild_locale
                .clone()
                .unwrap_or(interaction.locale.clone()),
        )
        .await
    {
        event!(Level::ERROR, error = %e, "cannot initialize the player");
        return Cow::borrowed(t(&interaction.locale, "error.unknown"));
    }

    let play_command = match LOADED_COMMANDS.get().and_then(|v| v.get("play")) {
        Some(v) => Cow::owned(format!("</play:{}>", v.get())),
        None => Cow::borrowed("`/play`"),
    };

    t_vars(&interaction.locale, "join.joined", [play_command.as_ref()])
}

/// Creates the `/join` [CreateCommand].
pub fn create_command() -> CreateCommand {
    let mut command = CreateCommand::new("join");

    command = serenity_command_name("join.name", command);
    command = serenity_command_description("join.description", command);

    command
        .description("Make me join your voice chat without playing anything.")
        .dm_permission(false)
}
