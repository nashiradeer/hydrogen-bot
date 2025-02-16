//! Lavalink REST client.

use std::time::Duration;

use super::{model::*, Error, LavalinkResult, Result, LAVALINK_USER_AGENT};
use http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode};
use reqwest::Client;

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

    #[cfg(feature = "simd-json")]
    #[cfg_attr(docsrs, doc(cfg(feature = "simd-json")))]
    /// Serialize a value to JSON using the selected JSON library.
    pub fn serialize<T: serde::Serialize + ?Sized>(&self, value: &T) -> Result<Vec<u8>> {
        simd_json::to_vec(value).map_err(Error::from)
    }

    #[cfg(not(feature = "simd-json"))]
    #[cfg_attr(docsrs, doc(cfg(not(feature = "simd-json"))))]
    /// Serialize a value to JSON using the selected JSON library.
    pub fn serialize<T: serde::Serialize + ?Sized>(&self, value: &T) -> Result<Vec<u8>> {
        serde_json::to_vec(value).map_err(Error::from)
    }

    #[cfg(feature = "simd-json")]
    #[cfg_attr(docsrs, doc(cfg(feature = "simd-json")))]
    /// Deserialize a value from JSON using the selected JSON library.
    pub fn deserialize<T: serde::de::DeserializeOwned>(&self, value: &mut [u8]) -> Result<T> {
        simd_json::from_slice(value).map_err(Error::from)
    }

    #[cfg(not(feature = "simd-json"))]
    #[cfg_attr(docsrs, doc(cfg(not(feature = "simd-json"))))]
    /// Deserialize a value from JSON using the selected JSON library.
    pub fn deserialize<T: serde::de::DeserializeOwned>(&self, value: &mut [u8]) -> Result<T> {
        serde_json::from_slice(value).map_err(Error::from)
    }

    /// Parse a response from the Lavalink server.
    async fn parse_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<Option<T>> {
        if response.status() == StatusCode::NO_CONTENT || response.status() == StatusCode::NOT_FOUND
        {
            Ok(None)
        } else {
            let response_body = response.bytes().await.map_err(Error::from)?;

            Into::<Result<T>>::into(
                self.deserialize::<LavalinkResult<T>>(
                    response_body.try_into_mut().unwrap().as_mut(),
                )?,
            )
            .map(Some)
        }
    }

    /// Fetch a value from the Lavalink server.
    pub async fn fetch<I: serde::Serialize + ?Sized, O: serde::de::DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        input: &I,
    ) -> Result<Option<O>> {
        let response = self
            .client
            .request(method, &self.build_url(path))
            .body(self.serialize(input)?)
            .send()
            .await
            .map_err(Error::from)?;

        self.parse_response(response).await
    }

    /// Fetch a value from the Lavalink server with body in the request, but without response body.
    pub async fn fetch_only_input<I: serde::Serialize>(
        &self,
        method: Method,
        path: &str,
        input: &I,
    ) -> Result<()> {
        self.client
            .request(method, &self.build_url(path))
            .body(self.serialize(input)?)
            .send()
            .await
            .map_err(Error::from)?
            .error_for_status()
            .map(|_| ())
            .map_err(Error::from)
    }

    /// Fetch a value from the Lavalink server without body in the request.
    pub async fn fetch_only_output<O: serde::de::DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
    ) -> Result<Option<O>> {
        let url = format!(
            "{}://{}{}",
            if self.tls { "https" } else { "http" },
            self.host,
            path
        );

        let response = self
            .client
            .request(method, &url)
            .send()
            .await
            .map_err(Error::from)?;

        self.parse_response(response).await
    }

    /// Fetch a value from the Lavalink server without body in the request or in the response.
    pub async fn fetch_empty(&self, method: Method, path: &str) -> Result<()> {
        let url = format!(
            "{}://{}{}",
            if self.tls { "https" } else { "http" },
            self.host,
            path,
        );

        self.client
            .request(method, &url)
            .send()
            .await
            .map_err(Error::from)?
            .error_for_status()
            .map(|_| ())
            .map_err(Error::from)
    }

    /// Load a track from an identifier.
    pub async fn load_track(&self, identifier: &str) -> Result<LoadResult> {
        self.fetch_only_output(
            Method::GET,
            &format!(
                "/v4/loadtracks?identifier={}&trace={}",
                identifier, self.trace
            ),
        )
        .await
        .transpose()
        .unwrap_or(Err(Error::NoResponseBody))
    }

    /// Decode a base64 track.
    pub async fn decode_track(&self, encoded_track: &str) -> Result<Track> {
        self.fetch_only_output(
            Method::GET,
            &format!(
                "/v4/decodetrack?encodedTrack={}&trace={}",
                encoded_track, self.trace
            ),
        )
        .await
        .transpose()
        .unwrap_or(Err(Error::NoResponseBody))
    }

    /// Decode multiple base64 tracks.
    pub async fn decode_tracks(&self, encoded_tracks: &[&str]) -> Result<Vec<Track>> {
        self.fetch(
            Method::POST,
            &format!("/v4/decodetracks?trace={}", self.trace),
            encoded_tracks,
        )
        .await
        .transpose()
        .unwrap_or(Err(Error::NoResponseBody))
    }

    /// Get all players in the session.
    pub async fn get_players(&self, session_id: &str) -> Result<Vec<Player>> {
        self.fetch_only_output(
            Method::GET,
            &format!("/v4/sessions/{}/players?trace={}", session_id, self.trace),
        )
        .await
        .transpose()
        .unwrap_or(Err(Error::NoResponseBody))
    }

    /// Get the player in the session.
    pub async fn get_player(&self, session_id: &str, guild_id: &str) -> Result<Option<Player>> {
        self.fetch_only_output(
            Method::GET,
            &format!(
                "/v4/sessions/{}/players/{}?trace={}",
                session_id, guild_id, self.trace
            ),
        )
        .await
    }

    /// Update the player in the session.
    pub async fn update_player(
        &self,
        session_id: &str,
        guild_id: &str,
        player: &UpdatePlayer,
        no_replace: bool,
    ) -> Result<Player> {
        self.fetch(
            Method::PATCH,
            &format!(
                "/v4/sessions/{}/players/{}?noReplace={}&trace={}",
                session_id, guild_id, no_replace, self.trace
            ),
            player,
        )
        .await
        .transpose()
        .unwrap_or(Err(Error::NoResponseBody))
    }

    /// Destroy the player in the session.
    pub async fn destroy_player(&self, session_id: &str, guild_id: &str) -> Result<()> {
        self.fetch_empty(
            Method::DELETE,
            &format!(
                "/v4/sessions/{}/players/{}?trace={}",
                session_id, guild_id, self.trace
            ),
        )
        .await
    }

    /// Update the session.
    pub async fn update_session(
        &self,
        session_id: &str,
        session: &UpdateSessionRequest,
    ) -> Result<UpdateSessionResponse> {
        self.fetch(
            Method::PATCH,
            &format!("/v4/sessions/{}?trace={}", session_id, self.trace),
            session,
        )
        .await
        .transpose()
        .unwrap_or(Err(Error::NoResponseBody))
    }

    /// Get information about the Lavalink server.
    pub async fn info(&self) -> Result<Info> {
        self.fetch_only_output(Method::GET, &format!("/v4/info?trace={}", self.trace))
            .await
            .transpose()
            .unwrap_or(Err(Error::NoResponseBody))
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
        self.fetch_only_output(
            Method::GET,
            &format!("/v4/routeplanner/status?trace={}", self.trace),
        )
        .await
    }

    /// Unmark a failed address in the Route Planner.
    pub async fn routeplanner_unmark(&self, address: &str) -> Result<()> {
        self.fetch_only_input(
            Method::POST,
            &format!("/v4/routeplanner/free/address?trace={}", self.trace),
            &UnmarkRoutePlanner {
                address: address.to_owned(),
            },
        )
        .await
    }

    /// Unmark all failed addresses in the Route Planner.
    pub async fn routeplanner_unmark_all(&self) -> Result<()> {
        self.fetch_empty(
            Method::POST,
            &format!("/v4/routeplanner/free/all?trace={}", self.trace),
        )
        .await
    }
}
