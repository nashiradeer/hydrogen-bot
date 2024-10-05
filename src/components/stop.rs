//! 'stop' component execution.

use serenity::all::{ComponentInteraction, Context};
use tracing::{error, info};

use crate::{
    handler::{Response, ResponseType},
    MANAGER,
};

/// Executes the `stop` command.
pub async fn execute<'a>(context: &Context, interaction: &ComponentInteraction) -> Response<'a> {
    let guild_id = match interaction.guild_id {
        Some(v) => v,
        None => {
            info!(
                "(components::stop): the user {} is not in a guild",
                interaction.user.id
            );
            return Response::new("stop.name", "error.not_in_guild", ResponseType::Error);
        }
    };

    let manager = match MANAGER.get() {
        Some(v) => v,
        None => {
            error!("(components::stop): the manager is not initialized");
            return Response::new("stop.name", "error.unknown", ResponseType::Error);
        }
    };

    let Some(voice_channel_id) = context.cache.guild(guild_id).and_then(|guild| {
        guild
            .voice_states
            .get(&interaction.user.id)
            .and_then(|voice_state| voice_state.channel_id)
    }) else {
        info!(
            "(components::stop): the user {} is not in a voice chat in the guild {}",
            interaction.user.id, guild_id
        );
        return Response::new(
            "stop.name",
            "error.unknown_voice_state",
            ResponseType::Error,
        );
    };

    if let Some(my_channel_id) = manager.get_voice_channel_id(guild_id).await {
        if my_channel_id == voice_channel_id.into() {
            if let Err(e) = manager.destroy(guild_id).await {
                error!(
                    "(components::stop): cannot stop the player in the guild {}: {}",
                    guild_id, e
                );
                return Response::new("stop.name", "error.unknown", ResponseType::Error);
            }

            Response::new("stop.name", "stop.stopped", ResponseType::Success)
        } else {
            Response::new(
                "stop.name",
                "error.not_in_voice_channel",
                ResponseType::Error,
            )
        }
    } else {
        Response::new("stop.name", "error.player_not_exists", ResponseType::Error)
    }
}
