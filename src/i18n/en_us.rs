//! English (United States) translations.

use phf::{phf_map, Map};

// This macro generates a static map with the translations.
pub static TRANSLATIONS: Map<&'static str, &'static str> = phf_map! {
    "generic.embed_footer" => "Hydrogen by Nashira Deer",
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
    "play.embed_title" => "Enqueuing/Playing songs",
    "play.play_single" => "Playing: **{name}** by **{author}**.",
    "play.play_single_url" => "Playing: [**{name}**]({url}) by **{author}**.",
    "play.play_multi" => "**{count}** songs from your playlist have been queued, **{name}** from **{author}** has been selected to play now.",
    "play.play_multi_url" => "**{count}** songs from your playlist have been queued, [**{name}**]({url}) from **{author}** has been selected to play now.",
    "play.enqueue_single" => "**{name}** by **{author}** has been added to the queue.",
    "play.enqueue_single_url" => "[**{name}**]({url}) by **{author}** has been added to the queue.",
    "play.enqueue_multi" => "**{count}** songs from your playlist have been queued.",
    "play.not_found" => "I can't find the requested song.",
    "play.truncated" => "You can't add more songs to the queue as it's already at the allowed limit. Please remove some songs before trying again.",
    "play.truncated_warn" => "**Warning: I need to exclude some songs from your playlist because it exceeds the allowed limit.**",
    "player.empty" => "_There's nothing currently playing._",
    "player.timeout" => "There's no one else connected to me in the voice chat. I will leave in {time} seconds.",
    "join.name" => "join",
    "join.description" => "Make me join your voice channel without playing anything.",
    "join.embed_title" => "Joining the voice channel",
    "join.joined" => "I have joined your voice channel, and now you can request any music using {play}.",
    "stop.embed_title" => "Stopping the music player",
    "stop.stopped" => "I'm leaving the voice channel. Hope to see you soon.",
    "loop.embed_title" => "Looping the queue",
    "loop.looping" => "Queue's loop mode has changed to **{loop}**.",
    "loop.autostart" => "Normal",
    "loop.no_autostart" => "Normal without auto-playing",
    "loop.music" => "Repeat Song",
    "loop.queue" => "Repeat Queue",
    "loop.random" => "Next Random Song",
    "pause.embed_title" => "Pause/Resume the Music Player",
    "pause.paused" => "You have paused the music player.",
    "pause.resumed" => "You have resumed the music player.",
    "skip.embed_title" => "Skipping to the next song",
    "skip.skipping" => "Skipping to the song **{name}** by **{author}**.",
    "skip.skipping_url" => "Skipping to the song [**{name}**]({url}) by **{author}**.",
    "prev.embed_title" => "Backing to the previous song",
    "prev.returning" => "Backing to the song **{name}** by **{author}**.",
    "prev.returning_url" => "Backing to the song [**{name}**]({url}) by **{author}**.",
    "seek.name" => "seek",
    "seek.description" => "Seek for the time in the current song playing.",
    "seek.time_name" => "time",
    "seek.time_description" => "Time in seconds or a supported syntax.",
    "seek.embed_title" => "Seeking song time",
    "seek.invalid_syntax" => "Invalid time time syntax. You can use numbers as seconds or suffix them with `m` to be minutes or `h` to be hours. You can also use `00:00` or `00:00:00` to set the hours.",
    "seek.seeking" => "**{name}**\n{author}\n``{current}/{total}``\n{progress}",
    "seek.seeking_url" => "[**{name}**]({url})\n{author}\n``{current}/{total}``\n{progress}",
};
