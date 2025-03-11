//! Lavalink REST client.

use std::time::Duration;

use super::{model::*, ApiResponse, Error, Result, LAVALINK_USER_AGENT};
use bytes::{Bytes, BytesMut};
use http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, Uri};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;
use url::Url;

#[derive(Debug, Clone)]
/// REST client for Lavalink.
pub struct Rest {
    /// The HTTP client with headers required for Lavalink.
    client: Client,
    /// The password for the Lavalink REST API, this is here for resuming sessions.
    password: String,
    /// The HTTP URL used to construct the URLs for the requests to the Lavalink REST API.
    http_url: Url,
    /// WebSocket URI used to connect to the Lavalink WebSocket.
    websocket_uri: Uri,
    /// Enables stack traces in all Lavalink REST API requests.
    pub trace: bool,
}

impl Rest {
    /// Create a new REST client.
    pub fn new(host: &str, password: &str, tls: bool) -> Result<Self> {
        let headers = [
            (
                HeaderName::from_static("authorization"),
                HeaderValue::from_str(password).map_err(Error::from)?,
            ),
            (
                HeaderName::from_static("content-type"),
                HeaderValue::from_static("application/json"),
            ),
        ];

        let client = Client::builder()
            .user_agent(LAVALINK_USER_AGENT)
            .default_headers(HeaderMap::from_iter(headers))
            .read_timeout(Duration::from_secs(60))
            .build()
            .map_err(Error::from)?;

        let http_url = Url::parse(&format!(
            "{}://{}",
            if tls { "https" } else { "http" },
            host
        ))
        .map_err(Error::from)?;

        let websocket_uri = Uri::builder()
            .scheme(if tls { "wss" } else { "ws" })
            .authority(host)
            .path_and_query("/v4/websocket")
            .build()
            .map_err(Error::from)?;

        Ok(Self {
            client,
            password: password.to_owned(),
            http_url,
            websocket_uri,
            trace: false,
        })
    }

    /// Get the [reqwest] client.
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Get the HTTP URL.
    pub fn http_url(&self) -> &Url {
        &self.http_url
    }

    /// Get the WebSocket URI.
    pub fn websocket_uri(&self) -> &Uri {
        &self.websocket_uri
    }

    /// Get the password for the Lavalink server.
    pub fn password(&self) -> &str {
        &self.password
    }

    /// Build a URL from a path.
    pub fn build_url(&self, path: &str) -> Result<Url> {
        self.http_url.join(path).map_err(Error::from)
    }

    #[cfg(feature = "simd-json")]
    #[cfg_attr(docsrs, doc(cfg(feature = "simd-json")))]
    /// Serialize the request in JSON using the selected JSON library.
    pub fn serialize_request<T: Serialize + ?Sized>(&self, value: &T) -> Result<Vec<u8>> {
        simd_json::to_vec(value).map_err(Error::from)
    }

    #[cfg(not(feature = "simd-json"))]
    #[cfg_attr(docsrs, doc(cfg(not(feature = "simd-json"))))]
    /// Serialize the request in JSON using the selected JSON library.
    pub fn serialize_request<T: Serialize + ?Sized>(&self, value: &T) -> Result<Vec<u8>> {
        serde_json::to_vec(value).map_err(Error::from)
    }

    #[cfg(feature = "simd-json")]
    #[cfg_attr(docsrs, doc(cfg(feature = "simd-json")))]
    /// Deserialize the response in JSON using the selected JSON library.
    pub fn deserialize_response<T: DeserializeOwned>(&self, value: Bytes) -> Result<T> {
        let mut value_mutable = value.try_into_mut().unwrap_or_else(BytesMut::from);
        simd_json::from_slice(value_mutable.as_mut()).map_err(Error::from)
    }

    #[cfg(not(feature = "simd-json"))]
    #[cfg_attr(docsrs, doc(cfg(not(feature = "simd-json"))))]
    /// Deserialize the response in JSON using the selected JSON library.
    pub fn deserialize_response<T: DeserializeOwned>(&self, value: Bytes) -> Result<T> {
        serde_json::from_slice(value.as_ref()).map_err(Error::from)
    }

    /// Parse a response from the Lavalink server.
    async fn parse_response<T: DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<Option<T>> {
        if response.status() == StatusCode::NO_CONTENT || response.status() == StatusCode::NOT_FOUND
        {
            Ok(None)
        } else {
            Into::<Result<T>>::into(self.deserialize_response::<ApiResponse<T>>(
                response.bytes().await.map_err(Error::from)?,
            )?)
            .map(Some)
        }
    }

    /// Call the Lavalink REST API with a request body and a response body.
    pub async fn call_req_res<Q: Serialize + ?Sized, I: Serialize + ?Sized, O: DeserializeOwned>(
        &self,
        method: Method,
        url: Url,
        query: &Q,
        input: &I,
    ) -> Result<Option<O>> {
        let response = self
            .client
            .request(method, url)
            .query(query)
            .body(self.serialize_request(input)?)
            .send()
            .await
            .map_err(Error::from)?;

        self.parse_response(response).await
    }

    /// Call the Lavalink REST API with a request body, but without a response body.
    ///
    /// All errors status codes (4xx and 5xx) will be returned as an error.
    pub async fn call_req<Q: Serialize + ?Sized, I: Serialize + ?Sized>(
        &self,
        method: Method,
        url: Url,
        query: &Q,
        input: &I,
    ) -> Result<()> {
        self.client
            .request(method, url)
            .query(query)
            .body(self.serialize_request(input)?)
            .send()
            .await
            .map_err(Error::from)?
            .error_for_status()
            .map(|_| ())
            .map_err(Error::from)
    }

    /// Call the Lavalink REST API without a request body, but with a response body.
    pub async fn call_res<Q: Serialize + ?Sized, O: DeserializeOwned>(
        &self,
        method: Method,
        url: Url,
        query: &Q,
    ) -> Result<Option<O>> {
        let response = self
            .client
            .request(method, url)
            .query(query)
            .send()
            .await
            .map_err(Error::from)?;

        self.parse_response(response).await
    }

    /// Call the Lavalink REST API without a request body and without a response body.\
    ///
    /// All errors status codes (4xx and 5xx) will be returned as an error.
    pub async fn call<Q: Serialize + ?Sized>(
        &self,
        method: Method,
        url: Url,
        query: &Q,
    ) -> Result<()> {
        self.client
            .request(method, url)
            .query(query)
            .send()
            .await
            .map_err(Error::from)?
            .error_for_status()
            .map(|_| ())
            .map_err(Error::from)
    }

    /// Load a track from an identifier.
    pub async fn load_track(&self, identifier: &str) -> Result<LoadResult> {
        self.call_res(
            Method::GET,
            self.build_url("/v4/loadtracks")?,
            &[
                ("identifier", identifier),
                ("trace", &self.trace.to_string()),
            ],
        )
        .await
        .transpose()
        .unwrap_or(Err(Error::NoResponseBody))
    }

    /// Decode a base64 track.
    pub async fn decode_track(&self, encoded_track: &str) -> Result<Track> {
        self.call_res(
            Method::GET,
            self.build_url("/v4/decodetrack")?,
            &[
                ("encodedTrack", encoded_track),
                ("trace", &self.trace.to_string()),
            ],
        )
        .await
        .transpose()
        .unwrap_or(Err(Error::NoResponseBody))
    }

    /// Decode multiple base64 tracks.
    pub async fn decode_tracks(&self, encoded_tracks: &[&str]) -> Result<Vec<Track>> {
        self.call_req_res(
            Method::POST,
            self.build_url("/v4/decodetracks")?,
            &[("trace", &self.trace.to_string())],
            encoded_tracks,
        )
        .await
        .transpose()
        .unwrap_or(Err(Error::NoResponseBody))
    }

    /// Get all players in the session.
    pub async fn get_players(&self, session_id: &str) -> Result<Vec<Player>> {
        self.call_res(
            Method::GET,
            self.build_url(&format!("/v4/sessions/{}/players", session_id))?,
            &[("trace", &self.trace.to_string())],
        )
        .await
        .transpose()
        .unwrap_or(Err(Error::NoResponseBody))
    }

    /// Get the player in the session.
    pub async fn get_player(&self, session_id: &str, guild_id: &str) -> Result<Option<Player>> {
        self.call_res(
            Method::GET,
            self.build_url(&format!("/v4/sessions/{}/players/{}", session_id, guild_id))?,
            &[("trace", &self.trace.to_string())],
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
        self.call_req_res(
            Method::PATCH,
            self.build_url(&format!("/v4/sessions/{}/players/{}", session_id, guild_id))?,
            &[
                ("noReplace", &no_replace.to_string()),
                ("trace", &self.trace.to_string()),
            ],
            player,
        )
        .await
        .transpose()
        .unwrap_or(Err(Error::NoResponseBody))
    }

    /// Destroy the player in the session.
    pub async fn destroy_player(&self, session_id: &str, guild_id: &str) -> Result<()> {
        self.call(
            Method::DELETE,
            self.build_url(&format!("/v4/sessions/{}/players/{}", session_id, guild_id))?,
            &[("trace", &self.trace.to_string())],
        )
        .await
    }

    /// Update the session.
    pub async fn update_session(
        &self,
        session_id: &str,
        session: &UpdateSessionRequest,
    ) -> Result<UpdateSessionResponse> {
        self.call_req_res(
            Method::PATCH,
            self.build_url(&format!("/v4/sessions/{}", session_id))?,
            &[("trace", &self.trace.to_string())],
            session,
        )
        .await
        .transpose()
        .unwrap_or(Err(Error::NoResponseBody))
    }

    /// Get information about the Lavalink server.
    pub async fn info(&self) -> Result<Info> {
        self.call_res(
            Method::GET,
            self.build_url("/v4/info")?,
            &[("trace", &self.trace.to_string())],
        )
        .await
        .transpose()
        .unwrap_or(Err(Error::NoResponseBody))
    }

    /// Get the Lavalink version.
    pub async fn version(&self) -> Result<String> {
        self.client
            .get(self.build_url("/version")?)
            .send()
            .await
            .map_err(Error::from)?
            .text()
            .await
            .map_err(Error::from)
    }

    /// Get the status of the Route Planner.
    pub async fn routeplanner_status(&self) -> Result<Option<RoutePlanner>> {
        self.call_res(
            Method::GET,
            self.build_url("/v4/routeplanner/status")?,
            &[("trace", &self.trace.to_string())],
        )
        .await
    }

    /// Unmark a failed address in the Route Planner.
    pub async fn routeplanner_unmark(&self, address: &str) -> Result<()> {
        self.call_req(
            Method::POST,
            self.build_url("/v4/routeplanner/free/address")?,
            &[("trace", &self.trace.to_string())],
            &UnmarkRoutePlanner {
                address: address.to_owned(),
            },
        )
        .await
    }

    /// Unmark all failed addresses in the Route Planner.
    pub async fn routeplanner_unmark_all(&self) -> Result<()> {
        self.call(
            Method::POST,
            self.build_url("/v4/routeplanner/free/all")?,
            &[("trace", &self.trace.to_string())],
        )
        .await
    }
}
