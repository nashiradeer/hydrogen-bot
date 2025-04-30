use tokio_tungstenite::{
    connect_async,
    tungstenite::{ClientRequestBuilder, Message as WsMessage, Result as WsResult},
};

use super::{Error, LAVALINK_CLIENT_NAME, LavalinkConnection, Message, Rest, Result};

/// Connect to a Lavalink server.
pub async fn connect(rest: &Rest, user_id: &str) -> Result<LavalinkConnection> {
    let request = ClientRequestBuilder::new(rest.websocket_uri().clone())
        .with_header("Authorization", rest.password())
        .with_header("User-Id", user_id)
        .with_header("Client-Name", LAVALINK_CLIENT_NAME);

    let (connection, _) = connect_async(request).await.map_err(Error::from)?;

    Ok(connection)
}

/// Reconnect to a Lavalink server, resuming a previous session.
pub async fn resume_session(
    rest: &Rest,
    user_id: &str,
    session_id: &str,
) -> Result<LavalinkConnection> {
    let request = ClientRequestBuilder::new(rest.websocket_uri().clone())
        .with_header("Authorization", rest.password())
        .with_header("User-Id", user_id)
        .with_header("Client-Name", LAVALINK_CLIENT_NAME)
        .with_header("Session-Id", session_id);

    let (connection, _) = connect_async(request).await.map_err(Error::from)?;

    Ok(connection)
}

/// Parse a message from the WebSocket connection.
pub fn parse_message(message: WsResult<WsMessage>) -> Result<Message> {
    let msg = message.map_err(Error::from)?;
    serde_json::from_slice(&msg.into_data()).map_err(Error::from)
}

#[cfg(feature = "parking-lot")]
/// A wrapper around `RwLock` to provide a consistent API.
#[derive(Debug)]
pub struct RwLock<T>(parking_lot::RwLock<T>);

#[cfg(feature = "parking-lot")]
impl<T> RwLock<T> {
    /// Create a new `RwLock` instance.
    pub fn new(value: T) -> Self {
        Self(parking_lot::RwLock::new(value))
    }

    /// Get a read lock on the `RwLock`.
    pub fn read(&self) -> parking_lot::RwLockReadGuard<'_, T> {
        self.0.read()
    }

    /// Get a write lock on the `RwLock`.
    pub fn write(&self) -> parking_lot::RwLockWriteGuard<'_, T> {
        self.0.write()
    }
}

#[cfg(not(feature = "parking-lot"))]
/// A wrapper around `RwLock` to provide a consistent API.
#[derive(Debug)]
pub struct RwLock<T>(std::sync::RwLock<T>);

#[cfg(not(feature = "parking-lot"))]
impl<T> RwLock<T> {
    /// Create a new `RwLock` instance.
    pub fn new(value: T) -> Self {
        Self(std::sync::RwLock::new(value))
    }

    /// Get a read lock on the `RwLock`.
    pub fn read(&self) -> std::sync::RwLockReadGuard<'_, T> {
        self.0.read().unwrap()
    }

    /// Get a write lock on the `RwLock`.
    pub fn write(&self) -> std::sync::RwLockWriteGuard<'_, T> {
        self.0.write().unwrap()
    }
}
