//! 'stop' component execution.

use serenity::all::{ComponentInteraction, Context};
use tracing::{event, Level};

use crate::{
    handler::{Response, ResponseType},
    PLAYER_MANAGER,
};

/// Executes the `stop` command.
pub async fn execute<'a>(context: &Context, interaction: &ComponentInteraction) -> Response<'a> {
    let guild_id = match interaction.guild_id {
        Some(v) => v,
        None => {
            event!(Level::WARN, "interaction.guild_id is None");
            return Response::new(
                "stop.embed_title",
                "error.not_in_guild",
                ResponseType::Error,
            );
        }
    };

    let manager = match PLAYER_MANAGER.get() {
        Some(v) => v,
        None => {
            event!(Level::ERROR, "PLAYER_MANAGER.get() returned None");
            return Response::new("stop.embed_title", "error.unknown", ResponseType::Error);
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
            "stop.embed_title",
            "error.unknown_voice_state",
            ResponseType::Error,
        );
    };

    if let Some(my_channel_id) = manager.get_voice_channel_id(guild_id) {
        if my_channel_id == voice_channel_id {
            if let Err(e) = manager.destroy(guild_id).await {
                event!(Level::ERROR, error = ?e, "cannot stop the player");
                return Response::new("stop.embed_title", "error.unknown", ResponseType::Error);
            }

            Response::new("stop.embed_title", "stop.stopped", ResponseType::Success)
        } else {
            Response::new(
                "stop.embed_title",
                "error.not_in_voice_channel",
                ResponseType::Error,
            )
        }
    } else {
        Response::new(
            "stop.embed_title",
            "error.player_not_exists",
            ResponseType::Error,
        )
    }
}
