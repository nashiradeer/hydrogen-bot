//! '/seek' command registration and execution.

use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
};
use tracing::{error, info};

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
    MANAGER,
};

/// Executes the `/seek` command.
pub async fn execute<'a>(context: &Context, interaction: &CommandInteraction) -> Response<'a> {
    let guild_id = match interaction.guild_id {
        Some(v) => v,
        None => {
            info!(
                "(commands::seek): the user {} is not in a guild",
                interaction.user.id
            );
            return Response::new("seek.name", "error.not_in_guild", ResponseType::Error);
        }
    };

    let manager = match MANAGER.get() {
        Some(v) => v,
        None => {
            error!("(commands::seek): the manager is not initialized");
            return Response::new("seek.name", "error.unknown", ResponseType::Error);
        }
    };

    let Some(time) = interaction
        .data
        .options
        .first()
        .and_then(|v| v.value.as_str())
    else {
        error!("(commands::seek): cannot get the 'time' option");
        return Response::new("seek.name", "error.unknown", ResponseType::Error);
    };

    let Some(voice_channel_id) = context.cache.guild(guild_id).and_then(|guild| {
        guild
            .voice_states
            .get(&interaction.user.id)
            .and_then(|voice_state| voice_state.channel_id)
    }) else {
        info!(
            "(commands::seek): the user {} is not in a voice chat in the guild {}",
            interaction.user.id, guild_id
        );
        return Response::new(
            "seek.name",
            "error.unknown_voice_state",
            ResponseType::Error,
        );
    };

    if let Some(my_channel_id) = manager.get_voice_channel_id(guild_id).await {
        if my_channel_id == voice_channel_id.into() {
            let seek_time = match suffix_syntax(time) {
                Some(v) => v,
                None => match semicolon_syntax(time) {
                    Some(v) => v,
                    None => {
                        info!(
                            "(commands::seek): the user {} provided an invalid syntax: {}",
                            interaction.user.id, time
                        );
                        return Response::new(
                            "seek.name",
                            "error.invalid_syntax",
                            ResponseType::Error,
                        );
                    }
                },
            };

            let converted_seek_time = match seek_time.try_into() {
                Ok(v) => v,
                Err(e) => {
                    error!(
                        "(commands::seek): cannot convert the seek time to a i32: {}",
                        e
                    );
                    return Response::new("seek.name", "error.unknown", ResponseType::Error);
                }
            };

            let seek_result = match manager.seek(guild_id, converted_seek_time).await {
                Ok(Some(v)) => v,
                Ok(None) => {
                    info!(
                        "(commands::seek): the player is empty in the guild {}",
                        guild_id
                    );
                    return Response::new("seek.name", "error.empty_queue", ResponseType::Error);
                }
                Err(e) => {
                    error!(
                        "(commands::seek): cannot seek the player in the guild {}: {}",
                        guild_id, e
                    );
                    return Response::new("seek.name", "error.unknown", ResponseType::Error);
                }
            };

            let current_time = time_to_string(seek_result.position / 1000);
            let total_time = time_to_string(seek_result.total / 1000);
            let progress_bar = progress_bar(seek_result.position, seek_result.total);

            let translation_message = if let Some(uri) = seek_result.track.uri {
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
                ResponseValue::TranslationKey("seek.name"),
                ResponseValue::RawString(translation_message),
                ResponseType::Success,
            )
        } else {
            Response::new("seek.name", "error.not_in_voice_chat", ResponseType::Error)
        }
    } else {
        Response::new("seek.name", "error.player_not_exists", ResponseType::Error)
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
