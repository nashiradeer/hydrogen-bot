//! 'skip' component execution.

use serenity::all::{ComponentInteraction, Context};
use tracing::{error, info, warn};

use crate::{
    handler::{Response, ResponseType, ResponseValue},
    i18n::t_vars,
    player::HydrogenMusic,
    MANAGER,
};

/// Executes the `skip` command.
pub async fn execute<'a>(context: &Context, interaction: &ComponentInteraction) -> Response<'a> {
    let guild_id = match interaction.guild_id {
        Some(v) => v,
        None => {
            info!(
                "(components::skip): the user {} is not in a guild",
                interaction.user.id
            );
            return Response::new("skip.name", "error.not_in_guild", ResponseType::Error);
        }
    };

    let manager = match MANAGER.get() {
        Some(v) => v,
        None => {
            error!("(components::skip): the manager is not initialized");
            return Response::new("skip.name", "error.unknown", ResponseType::Error);
        }
    };

    let Some(voice_channel_id) = context.cache.guild(guild_id).and_then(|guild| {
        guild
            .voice_states
            .get(&interaction.user.id)
            .and_then(|voice_state| voice_state.channel_id)
    }) else {
        info!(
            "(components::skip): the user {} is not in a voice chat in the guild {}",
            interaction.user.id, guild_id
        );
        return Response::new(
            "skip.name",
            "error.unknown_voice_state",
            ResponseType::Error,
        );
    };

    if let Some(my_channel_id) = manager.get_voice_channel_id(guild_id).await {
        if my_channel_id == voice_channel_id.into() {
            let music = match manager.skip(guild_id).await {
                Ok(v) => v,
                Err(e) => {
                    error!(
                        "(components::skip): cannot go to the next track in the guild {}: {}",
                        guild_id, e
                    );
                    return Response::new("skip.name", "error.unknown", ResponseType::Error);
                }
            };

            let Some(music) = music else {
                warn!(
                    "(components::skip): the queue is empty in the guild {}",
                    guild_id
                );
                return Response::new("skip.name", "error.empty_queue", ResponseType::Error);
            };

            Response::raw(
                ResponseValue::TranslationKey("skip.name"),
                ResponseValue::RawString(get_message(music, interaction)),
                ResponseType::Success,
            )
        } else {
            Response::new(
                "skip.name",
                "error.not_in_voice_channel",
                ResponseType::Error,
            )
        }
    } else {
        Response::new("skip.name", "error.player_not_exists", ResponseType::Error)
    }
}

/// Get the message to send to the user.
fn get_message(track: HydrogenMusic, interaction: &ComponentInteraction) -> String {
    if let Some(uri) = track.uri {
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
