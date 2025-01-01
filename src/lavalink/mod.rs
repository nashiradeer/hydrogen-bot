//! # Hydrolink
//!
//! An [tokio](https://tokio.rs) based [Lavalink](https://lavalink.dev/) client, with support for any Discord library.

mod cluster;
mod model;
mod rest;
mod websocket;

pub use model::*;
pub use rest::*;
pub use websocket::*;

/// User agent for the REST client.
pub static LAVALINK_USER_AGENT: &str =
    concat!("Hydrogen/", env!("CARGO_PKG_VERSION"), " Hydrolink/2.0.0");

/// Client name for the WebSocket client.
pub static LAVALINK_CLIENT_NAME: &str = "Hydrolink/2.0.0";

/// Result type used by this crate.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
/// Errors that can occur when interacting with Lavalink.
pub enum Error {
    /// An error from [`reqwest`].
    Reqwest(reqwest::Error),
    /// An error from [`serde_json`].
    Serde(serde_json::Error),
    /// An error from the Lavalink server.
    Lavalink(model::Error),
    /// An error from [`http`].
    Http(http::Error),
    /// An error from [`tokio_tungstenite`].
    Tungstenite(tokio_tungstenite::tungstenite::Error),
    /// No session ID was provided.
    NoSessionId,
    /// The message received from the Lavalink server was invalid.
    InvalidMessage,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reqwest(e) => e.fmt(f),
            Self::Serde(e) => e.fmt(f),
            Self::Lavalink(e) => e.fmt(f),
            Self::Http(e) => e.fmt(f),
            Self::Tungstenite(e) => e.fmt(f),
            Self::NoSessionId => write!(f, "No session ID was provided"),
            Self::InvalidMessage => write!(
                f,
                "The message received from the Lavalink server was invalid"
            ),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Self::Reqwest(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Serde(e)
    }
}

impl From<model::Error> for Error {
    fn from(e: model::Error) -> Self {
        Self::Lavalink(e)
    }
}

impl From<http::Error> for Error {
    fn from(e: http::Error) -> Self {
        Self::Http(e)
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for Error {
    fn from(e: tokio_tungstenite::tungstenite::Error) -> Self {
        Self::Tungstenite(e)
    }
}

impl std::error::Error for Error {}
