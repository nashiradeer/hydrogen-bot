use std::{
    borrow::Borrow,
    ops::Deref,
    sync::{RwLock, RwLockReadGuard},
};

use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use http::Uri;
use tokio::{
    net::TcpStream,
    sync::{Mutex as AsyncMutex, MutexGuard as AsyncMutexGuard},
};
use tokio_tungstenite::{
    connect_async, tungstenite::ClientRequestBuilder, MaybeTlsStream, WebSocketStream,
};

use super::{model::*, Error, Rest, Result, LAVALINK_CLIENT_NAME};

/// A connection to a Lavalink server.
pub type LavalinkConnection = WebSocketStream<MaybeTlsStream<TcpStream>>;
/// A stream of messages from a Lavalink server.
pub type LavalinkStream = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
/// A sink for sending messages to a Lavalink server.
pub type LavalinkSink =
    SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, tokio_tungstenite::tungstenite::Message>;

#[derive(Clone, Debug)]
/// A connection to a Lavalink server.
pub struct Lavalink {
    /// Session ID.
    session_id: RwLock<Option<String>>,
    /// Stream used to receive messages from the Lavalink server.
    stream: AsyncMutex<LavalinkStream>,
    /// Sink used to send messages to the Lavalink server.
    sink: AsyncMutex<LavalinkSink>,
    /// Client used to the Lavalink REST API.
    client: Rest,
    /// Bot's user ID.
    user_id: String,
}

impl Lavalink {
    /// Create a new Lavalink connection.
    pub fn new(connection: LavalinkConnection, client: Rest, user_id: &str) -> Self {
        let (sink, stream) = connection.split();

        Self {
            session_id: RwLock::new(None),
            stream: AsyncMutex::new(stream),
            sink: AsyncMutex::new(sink),
            client,
            user_id: user_id.to_owned(),
        }
    }

    /// Connect to the Lavalink server.
    pub async fn connect(&self) -> Result<()> {
        let stream = self.stream().await;
        let sink = self.sink().await;

        let uri = Uri::builder()
            .scheme(if self.client.tls() { "wss" } else { "ws" })
            .authority(self.client.host())
            .path_and_query("/v4/websocket")
            .build()
            .map_err(Error::from)?;

        let request = ClientRequestBuilder::new(uri)
            .with_header("Authorization", self.client.password())
            .with_header("User-Id", self.user_id())
            .with_header("Client-Name", LAVALINK_CLIENT_NAME);

        let (connection, _) = connect_async(request).await.map_err(Error::from)?;

        let (new_sink, new_stream) = connection.split();

        *stream = new_stream;
        *sink = new_sink;

        Ok(())
    }

    /// Resume the connection to the Lavalink server.
    pub async fn resume(&self) -> Result<()> {
        let stream = self.stream().await;
        let sink = self.sink().await;

        let uri = Uri::builder()
            .scheme(if self.client.tls() { "wss" } else { "ws" })
            .authority(self.client.host())
            .path_and_query("/v4/websocket")
            .build()
            .map_err(Error::from)?;

        let request = ClientRequestBuilder::new(uri)
            .with_header("Authorization", self.client.password())
            .with_header("User-Id", self.user_id())
            .with_header("Client-Name", LAVALINK_CLIENT_NAME)
            .with_header(
                "Session-Id",
                self.session_id().as_ref().ok_or(Error::NoSessionId)?,
            );

        let (connection, _) = connect_async(request).await.map_err(Error::from)?;

        let (new_sink, new_stream) = connection.split();

        *stream = new_stream;
        *sink = new_sink;

        Ok(())
    }

    /// Get the user ID.
    pub fn user_id(&self) -> &str {
        &self.user_id
    }

    /// Get the session ID.
    pub fn session_id(&self) -> RwLockReadGuard<Option<String>> {
        self.session_id.read().unwrap()
    }

    /// Get the WebSocket sink.
    pub async fn sink(&self) -> AsyncMutexGuard<LavalinkSink> {
        self.sink.lock().await
    }

    /// Get the WebSocket stream.
    pub async fn stream(&self) -> AsyncMutexGuard<LavalinkStream> {
        self.stream.lock().await
    }

    /// Get the REST client.
    pub fn client(&self) -> &Rest {
        &self.client
    }

    /// Get all players in the session.
    pub async fn get_players(&self) -> Result<Vec<Player>> {
        self.client
            .get_players(self.session_id().as_ref().ok_or(Error::NoSessionId)?)
            .await
    }

    /// Get the player in the session.
    pub async fn get_player(&self, guild_id: &str) -> Result<Player> {
        self.client
            .get_player(
                self.session_id().as_ref().ok_or(Error::NoSessionId)?,
                guild_id,
            )
            .await
    }

    /// Update the player in the session.
    pub async fn update_player(
        &self,
        guild_id: &str,
        player: &UpdatePlayer,
        no_replace: Option<bool>,
    ) -> Result<Player> {
        self.client
            .update_player(
                self.session_id().as_ref().ok_or(Error::NoSessionId)?,
                guild_id,
                player,
                no_replace,
            )
            .await
    }

    /// Destroy the player in the session.
    pub async fn destroy_player(&self, guild_id: &str) -> Result<()> {
        self.client
            .destroy_player(
                self.session_id().as_ref().ok_or(Error::NoSessionId)?,
                guild_id,
            )
            .await
    }

    /// Update the session.
    pub async fn update_session(
        &self,
        session: &UpdateSessionRequest,
    ) -> Result<UpdateSessionResponse> {
        self.client
            .update_session(
                self.session_id().as_ref().ok_or(Error::NoSessionId)?,
                session,
            )
            .await
    }

    /// Receive the next message from the Lavalink server.
    pub async fn next(&mut self) -> Option<Result<Message>> {
        let mut connection = self.stream().await;

        while let Some(msg) = connection.next().await {
            match msg {
                Ok(msg) => {
                    let data = match serde_json::from_slice(&msg.into_data()) {
                        Ok(data) => data,
                        Err(e) => return Some(Err(Error::Serde(e))),
                    };

                    match data {
                        Message::Ready {
                            resumed: _,
                            ref session_id,
                        } => {
                            *self.session_id.write().unwrap() = Some(session_id.clone());
                        }
                        _ => {}
                    };

                    return Some(Ok(data));
                }
                Err(e) => return Some(Err(Error::Tungstenite(e))),
            }
        }

        None
    }

    /// Close the connection to the Lavalink server.
    pub async fn close(&self) -> Result<()> {
        self.sink().await.close().await.map_err(Error::Tungstenite)
    }
}

impl AsRef<Rest> for Lavalink {
    fn as_ref(&self) -> &Rest {
        &self.client
    }
}

impl Borrow<Rest> for Lavalink {
    fn borrow(&self) -> &Rest {
        &self.client
    }
}

impl Deref for Lavalink {
    type Target = Rest;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}
