//! 'loop' component execution.

use serenity::all::{ComponentInteraction, Context};
use tracing::{error, info};

use crate::{
    handler::{Response, ResponseType, ResponseValue},
    i18n::{t, t_vars},
    player::LoopType,
    MANAGER,
};

/// Executes the `loop` command.
pub async fn execute<'a>(context: &Context, interaction: &ComponentInteraction) -> Response<'a> {
    let guild_id = match interaction.guild_id {
        Some(v) => v,
        None => {
            info!(
                "(components::loop): the user {} is not in a guild",
                interaction.user.id
            );
            return Response::new("loop.name", "error.not_in_guild", ResponseType::Error);
        }
    };

    let manager = match MANAGER.get() {
        Some(v) => v,
        None => {
            error!("(components::loop): the manager is not initialized");
            return Response::new("loop.name", "error.unknown", ResponseType::Error);
        }
    };

    let Some(voice_channel_id) = context.cache.guild(guild_id).and_then(|guild| {
        guild
            .voice_states
            .get(&interaction.user.id)
            .and_then(|voice_state| voice_state.channel_id)
    }) else {
        info!(
            "(components::loop): the user {} is not in a voice chat in the guild {}",
            interaction.user.id, guild_id
        );
        return Response::new(
            "loop.name",
            "error.unknown_voice_state",
            ResponseType::Error,
        );
    };

    if let Some(my_channel_id) = manager.get_voice_channel_id(guild_id).await {
        if my_channel_id == voice_channel_id.into() {
            let current_loop_type = manager.get_loop_type(guild_id).await;

            let new_loop_type = match current_loop_type {
                LoopType::None => LoopType::NoAutostart,
                LoopType::NoAutostart => LoopType::Music,
                LoopType::Music => LoopType::Queue,
                LoopType::Queue => LoopType::Random,
                LoopType::Random => LoopType::None,
            };

            manager.set_loop_type(guild_id, new_loop_type.clone()).await;

            let loop_type_translation_key = match new_loop_type {
                LoopType::None => "autostart",
                LoopType::NoAutostart => "no_autostart",
                LoopType::Music => "music",
                LoopType::Queue => "queue",
                LoopType::Random => "random",
            };

            let translation_key: &'a str = format!("loop.{}", loop_type_translation_key).leak();

            let loop_type_translation: &'a str = t(&interaction.locale, &translation_key);

            Response::raw(
                ResponseValue::TranslationKey("loop.name"),
                ResponseValue::Raw(t_vars(
                    &interaction.locale,
                    "loop.looping",
                    [("loop", loop_type_translation)],
                )),
                ResponseType::Success,
            )
        } else {
            Response::new("loop.name", "error.not_in_voice_chat", ResponseType::Error)
        }
    } else {
        Response::new("loop.name", "error.player_not_exists", ResponseType::Error)
    }
}
