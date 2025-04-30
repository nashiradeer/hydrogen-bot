use std::{borrow::Borrow, ops::Deref};

use futures::StreamExt;
use tokio::sync::Mutex as AsyncMutex;

#[cfg(feature = "parking-lot")]
use parking_lot::RwLock;

#[cfg(not(feature = "parking-lot"))]
use std::sync::RwLock;

use super::{
    Error, LavalinkConnection, Rest, Result,
    model::*,
    utils::{connect, parse_message, resume_session},
};

#[derive(Debug)]
/// A connection to a Lavalink server.
pub struct Lavalink {
    /// Session ID.
    session_id: RwLock<Option<String>>,
    /// WebSocket stream.
    connection: AsyncMutex<LavalinkConnection>,
    /// Client used to the Lavalink REST API.
    client: Rest,
    /// Bot's user ID.
    user_id: String,
}

impl Lavalink {
    /// Create a new Lavalink connection.
    pub fn new(connection: LavalinkConnection, client: Rest, user_id: &str) -> Self {
        Self {
            session_id: RwLock::new(None),
            connection: AsyncMutex::new(connection),
            client,
            user_id: user_id.to_owned(),
        }
    }

    /// Connect to a Lavalink server.
    pub async fn connect_from(rest: Rest, user_id: &str) -> Result<Self> {
        Ok(Self::new(connect(&rest, user_id).await?, rest, user_id))
    }

    /// Reconnect to a Lavalink server, resuming a previous session.
    pub async fn resume_from(rest: Rest, user_id: &str, session_id: &str) -> Result<Self> {
        Ok(Self::new(
            resume_session(&rest, user_id, session_id).await?,
            rest,
            user_id,
        ))
    }

    /// Connect to the Lavalink server.
    ///
    /// WARNING: This method locks the internal connection mutex.
    pub async fn connect(&self) -> Result<()> {
        *self.connection.lock().await = connect(self, &self.user_id).await?;

        Ok(())
    }

    /// Resume the connection to the Lavalink server.
    ///
    /// WARNING: This method locks the internal connection mutex.
    pub async fn resume(&self) -> Result<()> {
        *self.connection.lock().await = resume_session(
            self,
            &self.user_id,
            self.session_id().as_ref().ok_or(Error::NoSessionId)?,
        )
        .await?;

        Ok(())
    }

    /// Get the user ID.
    pub fn user_id(&self) -> &str {
        &self.user_id
    }

    #[cfg(feature = "parking-lot")]
    /// Get the session ID.
    ///
    /// This method clones the session ID to avoid locking the internal mutex.
    pub fn session_id(&self) -> Option<String> {
        self.session_id.read().clone()
    }

    #[cfg(not(feature = "parking-lot"))]
    /// Get the session ID.
    ///
    /// This method clones the session ID to avoid locking the internal mutex.
    pub fn session_id(&self) -> Option<String> {
        self.session_id.read().unwrap().clone()
    }

    /// Get the REST client.
    pub fn client(&self) -> &Rest {
        &self.client
    }

    /// Get all players in the session.
    pub async fn get_players(&self) -> Result<Vec<Player>> {
        self.client
            .get_players(&self.session_id().ok_or(Error::NoSessionId)?)
            .await
    }

    /// Get the player in the session.
    pub async fn get_player(&self, guild_id: &str) -> Result<Option<Player>> {
        self.client
            .get_player(&self.session_id().ok_or(Error::NoSessionId)?, guild_id)
            .await
    }

    /// Update the player in the session.
    pub async fn update_player(
        &self,
        guild_id: &str,
        player: &UpdatePlayer,
        no_replace: bool,
    ) -> Result<Player> {
        self.client
            .update_player(
                &self.session_id().ok_or(Error::NoSessionId)?,
                guild_id,
                player,
                no_replace,
            )
            .await
    }

    /// Destroy the player in the session.
    pub async fn destroy_player(&self, guild_id: &str) -> Result<()> {
        self.client
            .destroy_player(&self.session_id().ok_or(Error::NoSessionId)?, guild_id)
            .await
    }

    /// Update the session.
    pub async fn update_session(
        &self,
        session: &UpdateSessionRequest,
    ) -> Result<UpdateSessionResponse> {
        self.client
            .update_session(&self.session_id().ok_or(Error::NoSessionId)?, session)
            .await
    }

    #[cfg(feature = "parking-lot")]
    /// Receive the next message from the Lavalink server.
    ///
    /// WARNING: This method locks the internal connection mutex.
    pub async fn next(&self) -> Option<Result<Message>> {
        let data = parse_message(self.connection.lock().await.next().await?);

        if let Some(msg) = data.as_ref().ok().and_then(|v| v.as_ready()) {
            *self.session_id.write() = Some(msg.session_id.clone());
        }

        Some(data)
    }

    #[cfg(not(feature = "parking-lot"))]
    /// Receive the next message from the Lavalink server.
    ///
    /// WARNING: This method locks the internal connection mutex.
    pub async fn next(&self) -> Option<Result<Message>> {
        let data = parse_message(self.connection.lock().await.next().await?);

        if let Some(msg) = data.as_ref().ok().and_then(|v| v.as_ready()) {
            *self.session_id.write().unwrap() = Some(msg.session_id.clone());
        }

        Some(data)
    }

    /// Close the connection to the Lavalink server.
    ///
    /// WARNING: This method locks the internal connection mutex.
    pub async fn close(&self) -> Result<()> {
        self.connection
            .lock()
            .await
            .close(None)
            .await
            .map_err(Error::from)
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
