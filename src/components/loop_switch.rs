//! 'loop' component execution.

use serenity::all::{ComponentInteraction, Context};
use tracing::{event, Level};

use crate::{
    handler::{Response, ResponseType, ResponseValue},
    i18n::{t, t_vars},
    music::LoopMode,
    PLAYER_MANAGER,
};

/// Executes the `loop` command.
pub async fn execute<'a>(context: &Context, interaction: &ComponentInteraction) -> Response<'a> {
    let guild_id = match interaction.guild_id {
        Some(v) => v,
        None => {
            event!(Level::WARN, "interaction.guild_id is None");
            return Response::new(
                "loop.embed_title",
                "error.not_in_guild",
                ResponseType::Error,
            );
        }
    };

    let manager = match PLAYER_MANAGER.get() {
        Some(v) => v,
        None => {
            event!(Level::ERROR, "PLAYER_MANAGER.get() returned None");
            return Response::new("loop.embed_title", "error.unknown", ResponseType::Error);
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
            "loop.embed_title",
            "error.unknown_voice_state",
            ResponseType::Error,
        );
    };

    let player_state = manager
        .get_voice_channel_id(guild_id)
        .zip(manager.get_loop_mode(guild_id));

    if let Some((my_channel_id, current_loop_mode)) = player_state {
        if my_channel_id == voice_channel_id {
            let new_loop_mode = current_loop_mode.next();

            manager.set_loop_mode(guild_id, new_loop_mode).await;

            let loop_type_translation_key = match new_loop_mode {
                LoopMode::None => "loop.autostart",
                LoopMode::Autopause => "loop.no_autostart",
                LoopMode::Single => "loop.music",
                LoopMode::All => "loop.queue",
            };

            let loop_type_translation = t(&interaction.locale, loop_type_translation_key);

            Response::raw(
                ResponseValue::TranslationKey("loop.embed_title"),
                ResponseValue::RawString(t_vars(
                    &interaction.locale,
                    "loop.looping",
                    [("loop", loop_type_translation)],
                )),
                ResponseType::Success,
            )
        } else {
            Response::new(
                "loop.embed_title",
                "error.not_in_voice_chat",
                ResponseType::Error,
            )
        }
    } else {
        Response::new(
            "loop.embed_title",
            "error.player_not_exists",
            ResponseType::Error,
        )
    }
}
