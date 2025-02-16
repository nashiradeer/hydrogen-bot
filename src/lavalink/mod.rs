//! # Hydrolink
//!
//! An [tokio](https://tokio.rs) based [Lavalink](https://lavalink.dev/) client, with support for any Discord library.

pub mod cluster;
pub mod hydrogen;
mod model;
mod rest;
pub(crate) mod utils;
mod websocket;

use http::header::InvalidHeaderValue;
pub use model::*;
pub use rest::*;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

/// User agent for the REST client.
pub const LAVALINK_USER_AGENT: &str =
    concat!("Hydrogen/", env!("CARGO_PKG_VERSION"), " Hydrolink/2.0.0");

/// Client name for the WebSocket client.
pub const LAVALINK_CLIENT_NAME: &str = "Hydrolink/2.0.0";

/// Result type used by this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// A connection to a Lavalink server.
pub type LavalinkConnection = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Debug)]
/// Errors that can occur when interacting with Lavalink.
pub enum Error {
    /// An error from [`reqwest`].
    Reqwest(reqwest::Error),

    /// An error from [`serde_json`].
    Serde(serde_json::Error),

    #[cfg(feature = "simd-json")]
    /// An error from [`simd_json`].
    SimdJson(simd_json::Error),

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

    /// The password provided to the Lavalink server was invalid.
    InvalidHeaderValue(InvalidHeaderValue),

    /// The Lavalink node is already connected.
    AlreadyConnected,

    /// The response from the Lavalink server had no body.
    NoResponseBody,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reqwest(e) => write!(f, "Reqwest error: {}", e),

            Self::Serde(e) => write!(f, "Serde error: {}", e),

            Self::Lavalink(e) => write!(f, "Lavalink error: {:?}", e),

            Self::Http(e) => write!(f, "HTTP error: {}", e),

            Self::Tungstenite(e) => write!(f, "Tungstenite error: {}", e),

            Self::InvalidHeaderValue(e) => write!(f, "Invalid header value: {}", e),

            Self::NoSessionId => write!(f, "No session ID was provided"),

            Self::InvalidMessage => write!(f, "Lavalink sent an invalid message"),

            Self::AlreadyConnected => write!(f, "Lavalink node is already connected"),

            Self::NoResponseBody => write!(f, "Lavalink response had no body"),

            #[cfg(feature = "simd-json")]
            Self::SimdJson(e) => write!(f, "SimdJson error: {}", e),
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

impl From<InvalidHeaderValue> for Error {
    fn from(e: InvalidHeaderValue) -> Self {
        Self::InvalidHeaderValue(e)
    }
}

#[cfg(feature = "simd-json")]
impl From<simd_json::Error> for Error {
    fn from(e: simd_json::Error) -> Self {
        Self::SimdJson(e)
    }
}

impl std::error::Error for Error {}
