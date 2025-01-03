//! Static and constant values used to configure Hydrogen.

/// The embed color used for success messages.
pub const HYDROGEN_PRIMARY_COLOR: i32 = 0x5865f2;

/// The embed color used for error messages.
pub const HYDROGEN_ERROR_COLOR: i32 = 0xf04747;

/// Time in seconds to wait before exit from an empty voice channel.
pub const HYDROGEN_EMPTY_CHAT_TIMEOUT: u64 = 10;

/// How many music tracks can be stored in the queue.
pub const HYDROGEN_QUEUE_LIMIT: usize = 1000;

/// The search prefixes for the music.
pub static HYDROGEN_SEARCH_PREFIXES: [&str; 3] = ["ytsearch:", "dzsearch:", "scsearch:"];

/// Connection timeout for the Lavalink node in seconds.
pub const LAVALINK_RECONNECTION_DELAY: u64 = 5;

/// Hydrogen's logo URL, used in embed's footers.
pub static HYDROGEN_LOGO_URL: &str =
    "https://raw.githubusercontent.com/nashiradeer/hydrogen/main/assets/icons/hydrogen-circular.png";

#[allow(dead_code)]
/// Hydrogen's version.
pub static HYDROGEN_VERSION: &str = env!("CARGO_PKG_VERSION");

#[allow(dead_code)]
/// Hydrogen's project name.
pub static HYDROGEN_NAME: &str = "Hydrogen";
