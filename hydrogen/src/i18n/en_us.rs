//! English (United States) translations.

use phf::{Map, phf_map};

// This macro generates a static map with the translations.
pub static TRANSLATIONS: Map<&'static str, &'static str> = phf_map! {
    "error.unknown" => "An unknown error has happened!",
    "error.unknown_voice_state" => "I can't determine your voice state, check my permissions, or if you're in a voice chat.",
    "error.cant_connect" => "I can't join your voice channel. Check if I have permission to access it.",
    "error.not_in_voice_chat" => "You can't control the music player from outside the voice channel.",
    "error.player_exists" => "There's already a music player in another voice channel.",
    "error.player_not_exists" => "There's no music player on this server.",
    "error.empty_queue" => "There are no songs in the queue.",
    "error.not_in_guild" => "You can't use this command outside a server.",
    "play.name" => "play",
    "play.description" => "Request a song to play, adding it to the queue or playing immediately if empty.",
    "play.query_name" => "query",
    "play.query_description" => "A song or playlist URL, or a search term.",
    "play.mode_name" => "mode",
    "play.mode_description" => "The mode to play the song in.",
    "play.mode_end" => "Add To End",
    "play.mode_next" => "Add Next",
    "play.mode_now" => "Play Now",
    "play.play_single" => "Playing: **{0}** by **{1}**.",
    "play.play_single_url" => "Playing: [**{0}**](<{2}>) by **{1}**.",
    "play.play_multi" => "**{2}** songs from your playlist have been queued, **{0}** from **{1}** has been selected to play now.",
    "play.play_multi_url" => "**{2}** songs from your playlist have been queued, [**{0}**](<{3}>) from **{1}** has been selected to play now.",
    "play.enqueue_single" => "**{0}** by **{1}** has been added to the queue.",
    "play.enqueue_single_url" => "[**{0}**](<{2}>) by **{1}** has been added to the queue.",
    "play.enqueue_multi" => "**{0}** songs from your playlist have been queued.",
    "play.not_found" => "I can't find the requested song.",
    "play.truncated" => "You can't add more songs to the queue as it's already at the allowed limit. Please remove some songs before trying again.",
    "play.truncated_warn" => "**Warning: I need to exclude some songs from your playlist because it exceeds the allowed limit.**",
    "player.empty" => "_There's nothing currently playing._",
    "player.timeout" => "There's no one else connected to me in the voice chat. I will leave in {0} seconds.",
    "join.name" => "join",
    "join.description" => "Make me join your voice channel without playing anything.",
    "join.template_name" => "template",
    "join.template_description" => "The template to create the player with.",
    "join.template_default" => "Default",
    "join.template_music" => "Music",
    "join.template_queue" => "Queue",
    "join.template_manual" => "Manual",
    "join.template_rpg" => "RPG",
    "join.template_autoplay" => "Autoplay",
    "join.result" => "Created the player with the template **{0}**, now you can request any music using {1}.",
    "stop.name" => "stop",
    "stop.description" => "Stops the player.",
    "stop.stopped" => "I'm leaving the voice channel. Hope to see you soon.",
    "loop.name" => "loop",
    "loop.description" => "Changes the loop mode of the player.",
    "loop.mode_name" => "mode",
    "loop.mode_description" => "The loop mode to set.",
    "loop.mode_default" => "Default",
    "loop.mode_single" => "Single",
    "loop.mode_all" => "All",
    "loop.mode_auto_pause" => "Auto Pause",
    "loop.mode_autoplay" => "Autoplay",
    "loop.normal" => "Loop disabled, the player will start the next song automatically.",
    "loop.pause" => "Loop disabled, the player will wait for you to start the next song.",
    "loop.music" => "Looping the current song.",
    "loop.queue" => "Looping the entire queue.",
    "loop.autoplay" => "Autoplay enabled, the player will automatically add songs to the queue.",
    "pause.name" => "pause",
    "pause.description" => "Pauses or resumes the player.",
    "pause.paused" => "You have paused the music player.",
    "pause.resumed" => "You have resumed the music player.",
    "skip.name" => "skip",
    "skip.description" => "Skips to the next song in the queue.",
    "skip.skipping" => "Skipping to the song **{0}** by **{1}**.",
    "skip.skipping_url" => "Skipping to the song [**{0}**](<{2}>) by **{1}**.",
    "prev.name" => "previous",
    "prev.description" => "Plays the previous song in the queue.",
    "prev.returning" => "Backing to the song **{0}** by **{1}**.",
    "prev.returning_url" => "Backing to the song [**{0}**](<{2}>) by **{1}**.",
    "time.name" => "time",
    "time.description" => "See or change the current time of the playing track.",
    "time.time_name" => "time",
    "time.time_description" => "Time in seconds or a supported syntax.",
    "time.invalid_syntax" => "Invalid time time syntax. You can use numbers as seconds or suffix them with `m` to be minutes or `h` to be hours. You can also use `00:00` or `00:00:00` to set the hours.",
    "time.result" => "``{0}/{1}``\n{2}",
    "shuffle.name" => "shuffle",
    "shuffle.description" => "Shuffle the player queue.",
    "shuffle.result" => "The queue has been shuffled.",
};
