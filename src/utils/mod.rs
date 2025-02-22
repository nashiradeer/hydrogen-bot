//! Utilities that can be shared between commands and components.

use crate::i18n::t;
use beef::lean::Cow;
use serenity::all::{ChannelId, Context, GuildId, UserId};
use songbird::Songbird;
use std::sync::Arc;
use tracing::{event, Level};

pub mod constants;
pub mod time_parsers;

/// Converts a time in seconds to a string.
pub fn time_to_string(seconds: u64) -> String {
    if seconds < 60 {
        return format!("00:{:02}", seconds);
    } else if seconds < 60 * 60 {
        let time = seconds as f32;
        let minutes = (time / 60.0).floor();
        let seconds = time - minutes * 60.0;
        return format!("{:02}:{:02}", minutes as u32, seconds as u32);
    }

    let time = seconds as f32;
    let hours = (time / 60.0 / 60.0).floor();
    let minutes = (time - hours * 60.0 * 60.0).floor();
    let seconds = time - minutes * 60.0 - hours * 60.0 * 60.0;
    format!(
        "{:02}:{:02}:{:02}",
        hours as u32, minutes as u32, seconds as u32
    )
}

/// Creates a progress bar.
pub fn progress_bar(current: u64, total: u64) -> String {
    let item_total = 30usize;
    let item_count = (current as f32 / (total as f32 / item_total as f32)).round();
    let bar = "▓".repeat(item_count as usize);
    format!("╣{:░<width$.width$}╠", bar, width = item_total)
}

/// Gets the voice essentials for a user.
pub async fn get_voice_essentials<'a>(
    context: &Context,
    locale: &str,
    guild_id: GuildId,
    user_id: UserId,
) -> Result<(Arc<Songbird>, ChannelId), Cow<'a, str>> {
    let voice_channel_id = get_voice_channel(context, locale, guild_id, user_id)?;

    let Some(voice_manager) = songbird::get(context).await else {
        event!(Level::ERROR, "songbird::get() returned None");
        return Err(Cow::borrowed(t(&locale, "error.unknown")));
    };

    Ok((voice_manager, voice_channel_id))
}

/// Gets the voice channel of a user.
pub fn get_voice_channel<'a>(
    context: &Context,
    locale: &str,
    guild_id: GuildId,
    user_id: UserId,
) -> Result<ChannelId, Cow<'a, str>> {
    let Some(voice_channel_id) = context.cache.guild(guild_id).and_then(|guild| {
        guild
            .voice_states
            .get(&user_id)
            .and_then(|voice_state| voice_state.channel_id)
    }) else {
        event!(Level::INFO, "user voice state is None");
        return Err(Cow::borrowed(t(&locale, "error.unknown_voice_state")));
    };

    Ok(voice_channel_id)
}
