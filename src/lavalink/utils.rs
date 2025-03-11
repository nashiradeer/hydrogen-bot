use tokio_tungstenite::{
    connect_async,
    tungstenite::{ClientRequestBuilder, Message as WsMessage, Result as WsResult},
};

use super::{Error, LavalinkConnection, Message, Rest, Result, LAVALINK_CLIENT_NAME};

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
