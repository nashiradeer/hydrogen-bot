//! '/join' command registration and execution.

use serenity::{all::CommandInteraction, builder::CreateCommand, client::Context};
use tracing::{event, Level};

use crate::{
    handler::{Response, ResponseType, ResponseValue},
    i18n::{serenity_command_description, serenity_command_name, t_vars},
    LOADED_COMMANDS, PLAYER_MANAGER,
};

/// Executes the `/join` command.
pub async fn execute<'a>(context: &Context, interaction: &CommandInteraction) -> Response<'a> {
    let guild_id = match interaction.guild_id {
        Some(v) => v,
        None => {
            event!(Level::WARN, "interaction.guild_id is None");
            return Response::new(
                "join.embed_title",
                "error.not_in_guild",
                ResponseType::Error,
            );
        }
    };

    let manager = match PLAYER_MANAGER.get() {
        Some(v) => v,
        None => {
            event!(Level::ERROR, "PLAYER_MANAGER.get() returned None");
            return Response::new("join.embed_title", "error.unknown", ResponseType::Error);
        }
    };

    if manager.contains_player(guild_id) {
        event!(Level::INFO, "player already exists");
        return Response::new(
            "join.embed_title",
            "error.player_exists",
            ResponseType::Error,
        );
    }

    let Some(voice_channel_id) = context.cache.guild(guild_id).and_then(|guild| {
        guild
            .voice_states
            .get(&interaction.user.id)
            .and_then(|voice_state| voice_state.channel_id)
    }) else {
        event!(Level::INFO, "user voice state is None");
        return Response::new(
            "join.embed_title",
            "error.unknown_voice_state",
            ResponseType::Error,
        );
    };

    let voice_manager = match songbird::get(context).await {
        Some(v) => v,
        None => {
            event!(Level::ERROR, "songbird::get() returned None");
            return Response::new("join.embed_title", "error.unknown", ResponseType::Error);
        }
    };

    if let Err(e) = voice_manager.join_gateway(guild_id, voice_channel_id).await {
        event!(Level::INFO, voice_channel_id = %voice_channel_id, error = %e, "cannot join the voice channel");
        return Response::new(
            "join.embed_title",
            "error.cant_connect",
            ResponseType::Error,
        );
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
        return Response::new("join.embed_title", "error.unknown", ResponseType::Error);
    }

    let play_command = match LOADED_COMMANDS.get().and_then(|v| v.get("play")) {
        Some(v) => format!("</play:{}>", v.get()),
        None => "`/play`".to_owned(),
    };

    Response::raw(
        ResponseValue::TranslationKey("join.embed_title"),
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
