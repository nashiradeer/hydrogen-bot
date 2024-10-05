//! 'pause' component execution.

use serenity::all::{ComponentInteraction, Context};
use tracing::{error, info};

use crate::{
    handler::{Response, ResponseType},
    MANAGER,
};

/// Executes the `pause` command.
pub async fn execute<'a>(context: &Context, interaction: &ComponentInteraction) -> Response<'a> {
    let guild_id = match interaction.guild_id {
        Some(v) => v,
        None => {
            info!(
                "(components::pause): the user {} is not in a guild",
                interaction.user.id
            );
            return Response::new("pause.name", "error.not_in_guild", ResponseType::Error);
        }
    };

    let manager = match MANAGER.get() {
        Some(v) => v,
        None => {
            error!("(components::pause): the manager is not initialized");
            return Response::new("pause.name", "error.unknown", ResponseType::Error);
        }
    };

    let Some(voice_channel_id) = context.cache.guild(guild_id).and_then(|guild| {
        guild
            .voice_states
            .get(&interaction.user.id)
            .and_then(|voice_state| voice_state.channel_id)
    }) else {
        info!(
            "(components::pause): the user {} is not in a voice chat in the guild {}",
            interaction.user.id, guild_id
        );
        return Response::new(
            "pause.name",
            "error.unknown_voice_state",
            ResponseType::Error,
        );
    };

    if let Some(my_channel_id) = manager.get_voice_channel_id(guild_id).await {
        if my_channel_id == voice_channel_id.into() {
            let paused = !manager.get_paused(guild_id).await;

            // Pause or resume the player.
            if let Err(e) = manager.set_paused(guild_id, paused).await {
                error!(
                    "(components::pause): cannot resume/pause the player in the guild {}: {}",
                    guild_id, e
                );
                return Response::new("pause.name", "error.unknown", ResponseType::Error);
            }

            let translation_key = if paused { "paused" } else { "resumed" };

            let key_with_prefix: &'a str = format!("pause.{}", translation_key).leak();

            Response::new("pause.name", key_with_prefix, ResponseType::Success)
        } else {
            Response::new(
                "pause.name",
                "error.not_in_voice_channel",
                ResponseType::Error,
            )
        }
    } else {
        Response::new("pause.name", "error.player_not_exists", ResponseType::Error)
    }
}
