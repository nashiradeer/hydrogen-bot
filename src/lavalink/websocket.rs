use std::{borrow::Borrow, ops::Deref, sync::RwLock};

use futures::StreamExt;
use tokio::sync::Mutex as AsyncMutex;

use super::{
    model::*,
    utils::{connect, parse_message, resume_session},
    Error, LavalinkConnection, Rest, Result,
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
        Ok(Self::new(
            connect(&rest.host(), &rest.host(), rest.tls(), user_id).await?,
            rest,
            user_id,
        ))
    }

    /// Reconnect to a Lavalink server, resuming a previous session.
    pub async fn resume_from(rest: Rest, user_id: &str, session_id: &str) -> Result<Self> {
        Ok(Self::new(
            resume_session(&rest.host(), &rest.host(), rest.tls(), user_id, session_id).await?,
            rest,
            user_id,
        ))
    }

    /// Connect to the Lavalink server.
    ///
    /// WARNING: This method locks the internal connection mutex.
    pub async fn connect(&self) -> Result<()> {
        *self.connection.lock().await =
            connect(&self.host(), &self.host(), self.tls(), &self.user_id).await?;

        Ok(())
    }

    /// Resume the connection to the Lavalink server.
    ///
    /// WARNING: This method locks the internal connection mutex.
    pub async fn resume(&self) -> Result<()> {
        *self.connection.lock().await = resume_session(
            &self.host(),
            &self.host(),
            self.tls(),
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

    /// Get the session ID.
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
    ///
    /// WARNING: This method locks the internal connection mutex.
    pub async fn next(&self) -> Option<Result<Message>> {
        let mut connection = self.connection.lock().await;

        while let Some(msg) = connection.next().await {
            let data = parse_message(msg);

            if let Ok(ref data) = data {
                match data {
                    Message::Ready {
                        resumed: _,
                        ref session_id,
                    } => {
                        *self.session_id.write().unwrap() = Some(session_id.clone());
                    }
                    _ => {}
                };
            }

            return Some(data);
        }

        None
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
