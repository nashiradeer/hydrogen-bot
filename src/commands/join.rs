//! '/join' command registration and execution.

use serenity::{all::CommandInteraction, builder::CreateCommand, client::Context};
use tracing::{error, info, warn};

use crate::{
    handler::{Response, ResponseType, ResponseValue},
    i18n::{serenity_command_description, serenity_command_name, t_vars},
    LOADED_COMMANDS, MANAGER,
};

/// Executes the `/join` command.
pub async fn execute<'a>(context: &Context, interaction: &CommandInteraction) -> Response<'a> {
    let guild_id = match interaction.guild_id {
        Some(v) => v,
        None => {
            info!(
                "(commands::join): the user {} is not in a guild",
                interaction.user.id
            );
            return Response::new("join.name", "error.not_in_guild", ResponseType::Error);
        }
    };

    let manager = match MANAGER.get() {
        Some(v) => v,
        None => {
            error!("(commands::join): the manager is not initialized");
            return Response::new("join.name", "error.unknown", ResponseType::Error);
        }
    };

    if manager.contains_player(guild_id).await {
        info!(
            "(commands::join): a player already exists in the guild {}",
            guild_id
        );
        return Response::new("join.name", "error.player_exists", ResponseType::Error);
    }

    let Some(voice_channel_id) = context.cache.guild(guild_id).and_then(|guild| {
        guild
            .voice_states
            .get(&interaction.user.id)
            .and_then(|voice_state| voice_state.channel_id)
    }) else {
        info!(
            "(commands::join): the user {} is not in a voice chat in the guild {}",
            interaction.user.id, guild_id
        );
        return Response::new(
            "join.name",
            "error.unknown_voice_state",
            ResponseType::Error,
        );
    };

    let voice_manager = match songbird::get(context).await {
        Some(v) => v,
        None => {
            error!("(commands::join): cannot get the voice manager");
            return Response::new("join.name", "error.unknown", ResponseType::Error);
        }
    };

    if let Err(e) = voice_manager.join_gateway(guild_id, voice_channel_id).await {
        warn!(
            "(commands::join): cannot connect to the voice channel in the guild {}: {}",
            guild_id, e
        );
        return Response::new("join.name", "error.cant_connect", ResponseType::Error);
    }

    // Initialize the player.
    if let Err(e) = manager
        .init(
            guild_id,
            &interaction
                .guild_locale
                .clone()
                .unwrap_or(interaction.locale.clone()),
            voice_manager.clone(),
            interaction.channel_id,
        )
        .await
    {
        error!(
            "(commands::join): cannot initialize the player in the guild {}: {}",
            guild_id, e
        );
        return Response::new("join.name", "error.unknown", ResponseType::Error);
    }

    let play_command = match LOADED_COMMANDS.get().and_then(|v| v.get("play")) {
        Some(v) => format!("</play:{}>", v.get()),
        None => "`/play`".to_owned(),
    };

    Response::raw(
        ResponseValue::TranslationKey("join.name"),
        ResponseValue::RawString(t_vars(
            &interaction.locale,
            "join.joined",
            [("play", play_command)],
        )),
        ResponseType::Success,
    )
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
