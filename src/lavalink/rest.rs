//! Lavalink REST client.

use std::time::Duration;

use http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use reqwest::Client;

use super::{model::*, Error, LavalinkResult, Result, LAVALINK_USER_AGENT};

#[derive(Debug, Clone)]
/// REST client for Lavalink.
pub struct Rest {
    /// The HTTP client.
    client: Client,
    /// The host of the Lavalink server.
    host: String,
    /// Whether to use TLS.
    tls: bool,
    /// The password for the Lavalink server.
    password: String,
    /// Whether to enable stack traces.
    pub trace: bool,
}

impl Rest {
    /// Create a new REST client.
    pub fn new(host: &str, password: &str, tls: bool) -> Result<Self> {
        let headers = [(
            HeaderName::from_static("authorization"),
            HeaderValue::from_str(password).map_err(Error::from)?,
        )];

        let client = Client::builder()
            .user_agent(LAVALINK_USER_AGENT)
            .default_headers(HeaderMap::from_iter(headers))
            .read_timeout(Duration::from_secs(10))
            .build()
            .map_err(Error::from)?;

        Ok(Self {
            client,
            host: host.to_owned(),
            password: password.to_owned(),
            tls,
            trace: false,
        })
    }

    /// Get the [reqwest] client.
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Get the host of the Lavalink server.
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Get whether to use TLS.
    pub fn tls(&self) -> bool {
        self.tls
    }

    /// Get the password for the Lavalink server.
    pub fn password(&self) -> &str {
        &self.password
    }

    /// Build a URL from a path.
    pub fn build_url(&self, path: &str) -> String {
        format!(
            "{}://{}{}",
            if self.tls { "https" } else { "http" },
            self.host,
            path
        )
    }

    /// Load a track from an identifier.
    pub async fn load_track(&self, identifier: &str) -> Result<LoadResult> {
        self.client
            .get(self.build_url(&format!(
                "/v4/loadtracks?identifier={}&trace={}",
                identifier, self.trace
            )))
            .send()
            .await
            .map_err(Error::from)?
            .json::<LavalinkResult<_>>()
            .await
            .map_err(Error::from)?
            .into()
    }

    /// Decode a base64 track.
    pub async fn decode_track(&self, encoded_track: &str) -> Result<Track> {
        self.client
            .get(self.build_url(&format!(
                "/v4/decodetrack?encodedTrack={}&trace={}",
                encoded_track, self.trace
            )))
            .send()
            .await
            .map_err(Error::from)?
            .json::<LavalinkResult<_>>()
            .await
            .map_err(Error::from)?
            .into()
    }

    /// Decode multiple base64 tracks.
    pub async fn decode_tracks(&self, encoded_tracks: &[&str]) -> Result<Vec<Track>> {
        self.client
            .post(self.build_url(&format!("/v4/decodetracks?trace={}", self.trace)))
            .json(&encoded_tracks)
            .send()
            .await
            .map_err(Error::from)?
            .json::<LavalinkResult<_>>()
            .await
            .map_err(Error::from)?
            .into()
    }

    /// Get all players in the session.
    pub async fn get_players(&self, session_id: &str) -> Result<Vec<Player>> {
        self.client
            .get(self.build_url(&format!(
                "/v4/sessions/{}/players?trace={}",
                session_id, self.trace
            )))
            .send()
            .await
            .map_err(Error::from)?
            .json::<LavalinkResult<_>>()
            .await
            .map_err(Error::from)?
            .into()
    }

    /// Get the player in the session.
    pub async fn get_player(&self, session_id: &str, guild_id: &str) -> Result<Option<Player>> {
        let response = self
            .client
            .get(self.build_url(&format!(
                "/v4/sessions/{}/players/{}?trace={}",
                session_id, guild_id, self.trace
            )))
            .send()
            .await
            .map_err(Error::from)?;

        if response.status() == StatusCode::NOT_FOUND {
            Ok(None)
        } else {
            response
                .json::<LavalinkResult<_>>()
                .await
                .map_err(Error::from)?
                .into()
        }
    }

    /// Update the player in the session.
    pub async fn update_player(
        &self,
        session_id: &str,
        guild_id: &str,
        player: UpdatePlayer,
        no_replace: bool,
    ) -> Result<Player> {
        self.client
            .patch(self.build_url(&format!(
                "/v4/sessions/{}/players/{}?noReplace={}&trace={}",
                session_id, guild_id, no_replace, self.trace
            )))
            .json(&player)
            .send()
            .await
            .map_err(Error::from)?
            .json::<LavalinkResult<_>>()
            .await
            .map_err(Error::from)?
            .into()
    }

    /// Destroy the player in the session.
    pub async fn destroy_player(&self, session_id: &str, guild_id: &str) -> Result<()> {
        self.client
            .delete(self.build_url(&format!(
                "/v4/sessions/{}/players/{}?trace={}",
                session_id, guild_id, self.trace
            )))
            .send()
            .await
            .map_err(Error::from)?
            .error_for_status()
            .map(|_| ())
            .map_err(Error::from)
    }

    /// Update the session.
    pub async fn update_session(
        &self,
        session_id: &str,
        session: &UpdateSessionRequest,
    ) -> Result<UpdateSessionResponse> {
        self.client
            .patch(self.build_url(&format!("/v4/sessions/{}?trace={}", session_id, self.trace)))
            .json(session)
            .send()
            .await
            .map_err(Error::from)?
            .json::<LavalinkResult<_>>()
            .await
            .map_err(Error::from)?
            .into()
    }

    /// Get information about the Lavalink server.
    pub async fn info(&self) -> Result<Info> {
        self.client
            .get(self.build_url(&format!("/v4/info?trace={}", self.trace)))
            .send()
            .await
            .map_err(Error::from)?
            .json::<LavalinkResult<_>>()
            .await
            .map_err(Error::from)?
            .into()
    }

    /// Get the Lavalink version.
    pub async fn version(&self) -> Result<String> {
        self.client
            .get(self.build_url("/version"))
            .send()
            .await
            .map_err(Error::from)?
            .text()
            .await
            .map_err(Error::from)
    }

    /// Get the status of the Route Planner.
    pub async fn routeplanner_status(&self) -> Result<Option<RoutePlanner>> {
        let response = self
            .client
            .get(self.build_url(&format!("/v4/routeplanner/status?trace={}", self.trace)))
            .send()
            .await
            .map_err(Error::from)?;

        if response.status() == StatusCode::NO_CONTENT {
            Ok(None)
        } else {
            response
                .json::<LavalinkResult<RoutePlanner>>()
                .await
                .map_err(Error::from)
                .and_then(|result| result.into())
                .map(Some)
        }
    }

    /// Unmark a failed address in the Route Planner.
    pub async fn routeplanner_unmark(&self, address: &str) -> Result<()> {
        self.client
            .post(self.build_url(&format!(
                "/v4/routeplanner/free/address?trace={}",
                self.trace
            )))
            .json(&UnmarkRoutePlanner {
                address: address.to_owned(),
            })
            .send()
            .await
            .map_err(Error::from)?
            .error_for_status()
            .map(|_| ())
            .map_err(Error::from)
    }

    /// Unmark all failed addresses in the Route Planner.
    pub async fn routeplanner_unmark_all(&self) -> Result<()> {
        self.client
            .post(self.build_url(&format!("/v4/routeplanner/free/all?trace={}", self.trace)))
            .send()
            .await
            .map_err(Error::from)?
            .error_for_status()
            .map(|_| ())
            .map_err(Error::from)
    }
}
