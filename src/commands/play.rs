//! '/play' command registration and execution.

use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
};
use tracing::{event, Level};

use crate::{
    handler::{Response, ResponseType, ResponseValue},
    i18n::{
        serenity_command_description, serenity_command_name, serenity_command_option_description,
        serenity_command_option_name, t, t_vars,
    },
    music::PlayResult,
    PLAYER_MANAGER,
};

/// Executes the `/play` command.
pub async fn execute<'a>(context: &Context, interaction: &CommandInteraction) -> Response<'a> {
    let guild_id = match interaction.guild_id {
        Some(v) => v,
        None => {
            event!(Level::WARN, "interaction.guild_id is None");
            return Response::new(
                "play.embed_title",
                "error.not_in_guild",
                ResponseType::Error,
            );
        }
    };

    let manager = match PLAYER_MANAGER.get() {
        Some(v) => v,
        None => {
            event!(Level::ERROR, "PLAYER_MANAGER.get() returned None");
            return Response::new("play.embed_title", "error.unknown", ResponseType::Error);
        }
    };

    let Some(query) = interaction
        .data
        .options
        .first()
        .and_then(|v| v.value.as_str())
    else {
        event!(Level::WARN, "no query provided");
        return Response::new("play.embed_title", "error.unknown", ResponseType::Error);
    };

    let Some(voice_channel_id) = context.cache.guild(guild_id).and_then(|guild| {
        guild
            .voice_states
            .get(&interaction.user.id)
            .and_then(|voice_state| voice_state.channel_id)
    }) else {
        event!(Level::INFO, "user voice state is None");
        return Response::new(
            "play.embed_title",
            "error.unknown_voice_state",
            ResponseType::Error,
        );
    };

    let voice_manager = match songbird::get(context).await {
        Some(v) => v,
        None => {
            event!(Level::ERROR, "songbird::get() returned None");
            return Response::new("play.embed_title", "error.unknown", ResponseType::Error);
        }
    };

    let call = match voice_manager.get(guild_id) {
        Some(v) => {
            let has_connection = v.lock().await.current_connection().is_some();

            if !has_connection {
                // Join the voice channel.
                match voice_manager.join_gateway(guild_id, voice_channel_id).await {
                    Ok(v) => v.1,
                    Err(e) => {
                        event!(Level::INFO, voice_channel_id = %voice_channel_id, error = ?e, "cannot join the voice channel");
                        return Response::new(
                            "play.embed_title",
                            "error.cant_connect",
                            ResponseType::Error,
                        );
                    }
                }
            } else {
                v
            }
        }
        None => match voice_manager.join_gateway(guild_id, voice_channel_id).await {
            Ok(e) => e.1,
            Err(e) => {
                event!(Level::INFO, voice_channel_id = %voice_channel_id, error = ?e, "cannot join the voice channel");
                return Response::new(
                    "play.embed_title",
                    "error.cant_connect",
                    ResponseType::Error,
                );
            }
        },
    };

    if let Some(connection_info) = call.lock().await.current_connection() {
        if let Some(channel_id) = connection_info.channel_id {
            if channel_id != voice_channel_id.into() {
                return Response::new(
                    "play.embed_title",
                    "error.not_in_voice_chat",
                    ResponseType::Error,
                );
            }
        }
    }

    let result = match manager
        .play(
            query,
            interaction.user.id,
            guild_id,
            interaction.channel_id,
            &interaction
                .guild_locale
                .clone()
                .unwrap_or(interaction.locale.clone()),
        )
        .await
    {
        Ok(e) => e,
        Err(e) => {
            event!(Level::ERROR, error = ?e, guild_id = %guild_id, "cannot play the track");
            return Response::new("play.embed_title", "error.unknown", ResponseType::Error);
        }
    };

    if result.count > 0 {
        Response::raw(
            ResponseValue::TranslationKey("play.embed_title"),
            ResponseValue::RawString(generate_message(result, interaction)),
            ResponseType::Success,
        )
    } else if !result.truncated {
        Response::new("play.embed_title", "play.not_found", ResponseType::Error)
    } else {
        Response::new("play.embed_title", "play.truncated", ResponseType::Error)
    }
}

/// Creates the `/join` [CreateCommand].
pub fn create_command() -> CreateCommand {
    let mut command = CreateCommand::new("play");

    command = serenity_command_name("play.name", command);
    command = serenity_command_description("play.description", command);

    command
            .description(
                "Request music to be played, enqueuing it in the queue or playing immediately if empty.",
            )
            .add_option({
                let mut option = CreateCommandOption::new(
                    CommandOptionType::String,
                    "query",
                    "A music or playlist URL, or a search term.",
                )
                .required(true);

                    option =
                        serenity_command_option_name("play.query_name", option);
                    option = serenity_command_option_description(
                        "play.query_description",
                        option,
                    );

                option
            })
            .dm_permission(false)
}

/// Generates the message from the result from the player.
fn generate_message(result: PlayResult, interaction: &CommandInteraction) -> String {
    event!(
        Level::TRACE,
        result = ?result,
        "generating message from PlayResult"
    );
    if let Some(track) = result.track {
        if result.playing && result.count == 1 {
            return if let Some(url) = track.url {
                t_vars(
                    &interaction.locale,
                    "play.play_single_url",
                    [
                        ("name", track.title),
                        ("author", track.author),
                        ("url", url),
                    ],
                )
            } else {
                t_vars(
                    &interaction.locale,
                    "play.play_single",
                    [("name", track.title), ("author", track.author)],
                )
            };
        } else if result.count == 1 {
            return if let Some(url) = track.url {
                t_vars(
                    &interaction.locale,
                    "play.enqueue_single_url",
                    [
                        ("name", track.title),
                        ("author", track.author),
                        ("url", url),
                    ],
                )
            } else {
                t_vars(
                    &interaction.locale,
                    "play.enqueue_single",
                    [("name", track.title), ("author", track.author)],
                )
            };
        } else if result.playing {
            return if !result.truncated {
                if let Some(url) = track.url {
                    t_vars(
                        &interaction.locale,
                        "play.play_multi_url",
                        [
                            ("name", track.title),
                            ("author", track.author),
                            ("url", url),
                            ("count", result.count.to_string()),
                        ],
                    )
                } else {
                    t_vars(
                        &interaction.locale,
                        "play.play_multi",
                        [
                            ("name", track.title),
                            ("author", track.author),
                            ("count", result.count.to_string()),
                        ],
                    )
                }
            } else if let Some(url) = track.url {
                format!(
                    "{}\n\n{}",
                    t(&interaction.locale, "play.truncated_warn"),
                    t_vars(
                        &interaction.locale,
                        "play.play_multi_url",
                        [
                            ("name", track.title),
                            ("author", track.author),
                            ("url", url),
                            ("count", result.count.to_string()),
                        ]
                    ),
                )
            } else {
                format!(
                    "{}\n\n{}",
                    t(&interaction.locale, "play.truncated_warn"),
                    t_vars(
                        &interaction.locale,
                        "play.play_multi",
                        [
                            ("name", track.title),
                            ("author", track.author),
                            ("count", result.count.to_string()),
                        ]
                    ),
                )
            };
        }
    }

    if result.truncated {
        return format!(
            "{}\n\n{}",
            t(&interaction.locale, "play.truncated_warn"),
            t_vars(
                &interaction.locale,
                "play.enqueue_multi",
                [("count", result.count.to_string())]
            ),
        );
    }

    t_vars(
        &interaction.locale,
        "play.enqueue_multi",
        [("count", result.count.to_string())],
    )
}
