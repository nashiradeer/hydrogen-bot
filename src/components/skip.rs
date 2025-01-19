//! 'skip' component execution.

use serenity::all::{ComponentInteraction, Context};
use tracing::{event, Level};

use crate::{
    handler::{Response, ResponseType, ResponseValue},
    i18n::t_vars,
    music::Track,
    PLAYER_MANAGER,
};

/// Executes the `skip` command.
pub async fn execute<'a>(context: &Context, interaction: &ComponentInteraction) -> Response<'a> {
    let guild_id = match interaction.guild_id {
        Some(v) => v,
        None => {
            event!(Level::WARN, "interaction.guild_id is None");
            return Response::new(
                "skip.embed_title",
                "error.not_in_guild",
                ResponseType::Error,
            );
        }
    };

    let manager = match PLAYER_MANAGER.get() {
        Some(v) => v,
        None => {
            event!(Level::ERROR, "PLAYER_MANAGER.get() returned None");
            return Response::new("skip.embed_title", "error.unknown", ResponseType::Error);
        }
    };

    let Some(voice_channel_id) = context.cache.guild(guild_id).and_then(|guild| {
        guild
            .voice_states
            .get(&interaction.user.id)
            .and_then(|voice_state| voice_state.channel_id)
    }) else {
        event!(Level::INFO, "user voice state is None");
        return Response::new(
            "skip.embed_title",
            "error.unknown_voice_state",
            ResponseType::Error,
        );
    };

    if let Some(my_channel_id) = manager.get_voice_channel_id(guild_id) {
        if my_channel_id == voice_channel_id {
            let music = match manager.skip(guild_id).await {
                Ok(v) => v,
                Err(e) => {
                    event!(Level::ERROR, error = ?e, "cannot go to the previous track");
                    return Response::new("skip.embed_title", "error.unknown", ResponseType::Error);
                }
            };

            let Some(music) = music else {
                return Response::new("skip.embed_title", "error.empty_queue", ResponseType::Error);
            };

            Response::raw(
                ResponseValue::TranslationKey("skip.embed_title"),
                ResponseValue::RawString(get_message(music, interaction)),
                ResponseType::Success,
            )
        } else {
            Response::new(
                "skip.embed_title",
                "error.not_in_voice_channel",
                ResponseType::Error,
            )
        }
    } else {
        Response::new(
            "skip.embed_title",
            "error.player_not_exists",
            ResponseType::Error,
        )
    }
}

/// Get the message to send to the user.
fn get_message(track: Track, interaction: &ComponentInteraction) -> String {
    if let Some(uri) = track.url {
        t_vars(
            &interaction.locale,
            "prev.returning_url",
            [
                ("name", track.title),
                ("author", track.author),
                ("url", uri),
            ],
        )
    } else {
        t_vars(
            &interaction.locale,
            "prev.returning",
            [("name", track.title), ("author", track.author)],
        )
    }
}
