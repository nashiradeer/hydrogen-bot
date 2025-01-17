//! 'pause' component execution.

use serenity::all::{ComponentInteraction, Context};
use tracing::{event, Level};

use crate::{
    handler::{Response, ResponseType},
    PLAYER_MANAGER,
};

/// Executes the `pause` command.
pub async fn execute<'a>(context: &Context, interaction: &ComponentInteraction) -> Response<'a> {
    let guild_id = match interaction.guild_id {
        Some(v) => v,
        None => {
            event!(Level::WARN, "interaction.guild_id is None");
            return Response::new(
                "pause.embed_title",
                "error.not_in_guild",
                ResponseType::Error,
            );
        }
    };

    let manager = match PLAYER_MANAGER.get() {
        Some(v) => v,
        None => {
            event!(Level::ERROR, "PLAYER_MANAGER.get() returned None");
            return Response::new("pause.embed_title", "error.unknown", ResponseType::Error);
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
            "pause.embed_title",
            "error.unknown_voice_state",
            ResponseType::Error,
        );
    };

    let player_state = manager
        .get_voice_channel_id(guild_id)
        .zip(manager.get_pause(guild_id));

    if let Some((my_channel_id, paused)) = player_state {
        if my_channel_id == voice_channel_id {
            // Pause or resume the player.
            if let Err(e) = manager.set_pause(guild_id, !paused).await {
                event!(Level::ERROR, error = ?e, pause = !paused, "cannot resume/pause the player");
                return Response::new("pause.embed_title", "error.unknown", ResponseType::Error);
            }

            let translation_key = if paused {
                "pause.paused"
            } else {
                "pause.resumed"
            };

            Response::new("pause.embed_title", translation_key, ResponseType::Success)
        } else {
            Response::new(
                "pause.embed_title",
                "error.not_in_voice_channel",
                ResponseType::Error,
            )
        }
    } else {
        Response::new(
            "pause.embed_title",
            "error.player_not_exists",
            ResponseType::Error,
        )
    }
}
