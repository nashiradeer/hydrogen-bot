//! Static and constant values used to configure Hydrogen.

use std::time::Duration;

/// The embed color used for success messages.
pub const HYDROGEN_PRIMARY_COLOR: i32 = 0x0db363;

/// Time in seconds to wait before exit from an empty voice channel.
pub const HYDROGEN_EMPTY_CHAT_TIMEOUT: u64 = 10;

/// How many music tracks can be stored in the queue.
pub const HYDROGEN_QUEUE_LIMIT: usize = 1000;

/// The search prefixes for the music.
pub static HYDROGEN_SEARCH_PREFIXES: [&str; 4] =
    ["spsearch:", "ytsearch:", "dzsearch:", "scsearch:"];

/// Connection timeout for the Lavalink node in seconds.
pub const LAVALINK_RECONNECTION_DELAY: u64 = 5;

/// The user agent used for the Lavalink node.
pub static HYDROGEN_USER_AGENT: &str = concat!("Hydrogen/", env!("CARGO_PKG_VERSION"),);

/// The time in milliseconds to consider a ready event as slow.
pub const HYDROGEN_READY_THRESHOLD: Duration = Duration::from_millis(600);

/// The time in milliseconds to consider an interaction create event as slow.
pub const HYDROGEN_INTERACTION_CREATE_THRESHOLD: Duration = Duration::from_millis(15000);

/// The time in milliseconds to consider an update voice state event as slow.
pub const HYDROGEN_UPDATE_VOICE_STATE_THRESHOLD: Duration = Duration::from_millis(1000);

/// The time in milliseconds to consider an update voice server event as slow.
pub const HYDROGEN_UPDATE_VOICE_SERVER_THRESHOLD: Duration = Duration::from_millis(350);

/// The time in milliseconds to consider a lavalink event as slow.
pub const HYDROGEN_LAVALINK_EVENT_THRESHOLD: Duration = Duration::from_millis(1000);
