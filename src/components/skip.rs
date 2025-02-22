//! 'skip' component execution.

use beef::lean::Cow;
use serenity::all::{ComponentInteraction, Context};
use tracing::{event, Level};

use crate::i18n::t;
use crate::{i18n::t_vars, music::Track, PLAYER_MANAGER};

/// Executes the `skip` command.
pub async fn execute<'a>(context: &Context, interaction: &ComponentInteraction) -> Cow<'a, str> {
    let Some(guild_id) = interaction.guild_id else {
        event!(Level::WARN, "interaction.guild_id is None");
        return Cow::borrowed(t(&interaction.locale, "error.not_in_guild"));
    };

    let Some(manager) = PLAYER_MANAGER.get() else {
        event!(Level::ERROR, "PLAYER_MANAGER.get() returned None");
        return Cow::borrowed(t(&interaction.locale, "error.unknown"));
    };

    let Some(voice_channel_id) = context.cache.guild(guild_id).and_then(|guild| {
        guild
            .voice_states
            .get(&interaction.user.id)
            .and_then(|voice_state| voice_state.channel_id)
    }) else {
        event!(Level::INFO, "user voice state is None");
        return Cow::borrowed(t(&interaction.locale, "error.unknown_voice_state"));
    };

    if let Some(my_channel_id) = manager.get_voice_channel_id(guild_id) {
        if my_channel_id == voice_channel_id {
            let music = match manager.skip(guild_id).await {
                Ok(v) => v,
                Err(e) => {
                    event!(Level::ERROR, error = ?e, "cannot go to the previous track");
                    return Cow::borrowed(t(&interaction.locale, "error.unknown"));
                }
            };

            let Some(music) = music else {
                return Cow::borrowed(t(&interaction.locale, "error.empty_queue"));
            };

            get_message(music, interaction)
        } else {
            Cow::borrowed(t(&interaction.locale, "error.not_in_voice_channel"))
        }
    } else {
        Cow::borrowed(t(&interaction.locale, "error.player_not_exists"))
    }
}

/// Get the message to send to the user.
fn get_message<'a>(track: Track, interaction: &ComponentInteraction) -> Cow<'a, str> {
    if let Some(uri) = track.url {
        t_vars(
            &interaction.locale,
            "skip.returning_url",
            [track.title, track.author, uri],
        )
    } else {
        t_vars(
            &interaction.locale,
            "skip.returning",
            [track.title, track.author],
        )
    }
}
