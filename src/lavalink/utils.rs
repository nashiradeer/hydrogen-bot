use http::Uri;
use tokio_tungstenite::{connect_async, tungstenite::ClientRequestBuilder};

use super::{Error, LavalinkConnection, Result, LAVALINK_CLIENT_NAME};

/// Connect to a Lavalink server.
pub async fn connect(
    host: &str,
    password: &str,
    tls: bool,
    user_id: &str,
) -> Result<LavalinkConnection> {
    let uri = Uri::builder()
        .scheme(if tls { "wss" } else { "ws" })
        .authority(host)
        .path_and_query("/v4/websocket")
        .build()
        .map_err(Error::from)?;

    let request = ClientRequestBuilder::new(uri)
        .with_header("Authorization", password)
        .with_header("User-Id", user_id)
        .with_header("Client-Name", LAVALINK_CLIENT_NAME);

    let (connection, _) = connect_async(request).await.map_err(Error::from)?;

    Ok(connection)
}

/// Reconnect to a Lavalink server, resuming a previous session.
pub async fn resume_session(
    host: &str,
    password: &str,
    tls: bool,
    user_id: &str,
    session_id: &str,
) -> Result<LavalinkConnection> {
    let uri = Uri::builder()
        .scheme(if tls { "wss" } else { "ws" })
        .authority(host)
        .path_and_query("/v4/websocket")
        .build()
        .map_err(Error::from)?;

    let request = ClientRequestBuilder::new(uri)
        .with_header("Authorization", password)
        .with_header("User-Id", user_id)
        .with_header("Client-Name", LAVALINK_CLIENT_NAME)
        .with_header("Session-Id", session_id);

    let (connection, _) = connect_async(request).await.map_err(Error::from)?;

    Ok(connection)
}
