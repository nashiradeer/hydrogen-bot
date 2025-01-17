//! '/seek' command registration and execution.

use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
};
use tracing::{event, Level};

use crate::{
    handler::{Response, ResponseType, ResponseValue},
    i18n::{
        serenity_command_description, serenity_command_name, serenity_command_option_description,
        serenity_command_option_name, t_vars,
    },
    utils::{
        progress_bar,
        time_parsers::{semicolon_syntax, suffix_syntax},
        time_to_string,
    },
    PLAYER_MANAGER,
};

/// Executes the `/seek` command.
pub async fn execute<'a>(context: &Context, interaction: &CommandInteraction) -> Response<'a> {
    let guild_id = match interaction.guild_id {
        Some(v) => v,
        None => {
            event!(Level::WARN, "interaction.guild_id is None");
            return Response::new(
                "seek.embed_title",
                "error.not_in_guild",
                ResponseType::Error,
            );
        }
    };

    let manager = match PLAYER_MANAGER.get() {
        Some(v) => v,
        None => {
            event!(Level::ERROR, "PLAYER_MANAGER.get() returned None");
            return Response::new("seek.embed_title", "error.unknown", ResponseType::Error);
        }
    };

    let Some(time) = interaction
        .data
        .options
        .first()
        .and_then(|v| v.value.as_str())
    else {
        event!(Level::WARN, "no time provided");
        return Response::new("seek.embed_title", "error.unknown", ResponseType::Error);
    };

    let Some(voice_channel_id) = context.cache.guild(guild_id).and_then(|guild| {
        guild
            .voice_states
            .get(&interaction.user.id)
            .and_then(|voice_state| voice_state.channel_id)
    }) else {
        event!(Level::INFO, "user voice state is None");
        return Response::new(
            "seek.embed_title",
            "error.unknown_voice_state",
            ResponseType::Error,
        );
    };

    if let Some(my_channel_id) = manager.get_voice_channel_id(guild_id) {
        if my_channel_id == voice_channel_id {
            let seek_time = match suffix_syntax(time) {
                Some(v) => v,
                None => match semicolon_syntax(time) {
                    Some(v) => v,
                    None => {
                        event!(Level::INFO, syntax = time, "invalid syntax provided");
                        return Response::new(
                            "seek.embed_title",
                            "error.invalid_syntax",
                            ResponseType::Error,
                        );
                    }
                },
            };

            let seek_result = match manager.seek(guild_id, seek_time).await {
                Ok(Some(v)) => v,
                Ok(None) => {
                    return Response::new(
                        "seek.embed_title",
                        "error.empty_queue",
                        ResponseType::Error,
                    );
                }
                Err(e) => {
                    event!(Level::ERROR, error = ?e, "cannot seek the player");
                    return Response::new("seek.embed_title", "error.unknown", ResponseType::Error);
                }
            };

            let current_time = time_to_string(seek_result.position / 1000);
            let total_time = time_to_string(seek_result.total / 1000);
            let progress_bar = progress_bar(seek_result.position, seek_result.total);

            let translation_message = if let Some(uri) = seek_result.track.url {
                t_vars(
                    &interaction.locale,
                    "seek.seeking_url",
                    [
                        ("name", seek_result.track.title),
                        ("author", seek_result.track.author),
                        ("url", uri),
                        ("current", current_time),
                        ("total", total_time),
                        ("progress", progress_bar),
                    ],
                )
            } else {
                t_vars(
                    &interaction.locale,
                    "seek.seeking",
                    [
                        ("name", seek_result.track.title),
                        ("author", seek_result.track.author),
                        ("current", current_time),
                        ("total", total_time),
                        ("progress", progress_bar),
                    ],
                )
            };
            Response::raw(
                ResponseValue::TranslationKey("seek.embed_title"),
                ResponseValue::RawString(translation_message),
                ResponseType::Success,
            )
        } else {
            Response::new(
                "seek.embed_title",
                "error.not_in_voice_chat",
                ResponseType::Error,
            )
        }
    } else {
        Response::new(
            "seek.embed_title",
            "error.player_not_exists",
            ResponseType::Error,
        )
    }
}

/// Creates the `/seek` [CreateCommand].
pub fn create_command() -> CreateCommand {
    let mut command = CreateCommand::new("seek");

    command = serenity_command_name("seek.name", command);
    command = serenity_command_description("seek.description", command);

    command
        .description("Seek for the time in the current music playing.")
        .add_option({
            let mut option = CreateCommandOption::new(
                CommandOptionType::String,
                "time",
                "Time in seconds or a supported syntax.",
            )
            .required(true);

            option = serenity_command_option_name("seek.time_name", option);
            option = serenity_command_option_description("seek.time_description", option);

            option
        })
        .dm_permission(false)
}
