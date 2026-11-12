use std::hash::{BuildHasher, RandomState};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use futures_util::stream::{self, BoxStream, StreamExt, TryStreamExt};
use reqwest::Url;
use reqwest::header::{AUTHORIZATION, HeaderValue, RETRY_AFTER, USER_AGENT};
use serde::de::DeserializeOwned;

use crate::error::{Error, ErrorCode, Result};
use crate::models::{Medicine, MedicineDetail, Meta, Page, SearchResult};
use crate::time;

/// The address of the hosted API.
pub const DEFAULT_BASE_URL: &str = "https://api.oushodh.dev";

/// The environment variable the key is read from when none is passed.
pub const API_KEY_ENV: &str = "OUSHODH_API_KEY";

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_MAX_RETRIES: u32 = 2;
const UA: &str = concat!("oushodh-rust/", env!("CARGO_PKG_VERSION"));

/// The parameters of List medicines. Every one is optional.
///
/// ```
/// use std::time::{Duration, SystemTime};
///
/// let params = oushodh::ListParams {
///     limit: Some(500),
///     updated_since: Some(SystemTime::now() - Duration::from_secs(3600)),
///     ..Default::default()
/// };
/// # let _ = params;
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ListParams {
    /// Medicines on one page, 1 to 500. The API's default is 100.
    pub limit: Option<u32>,
    /// The `next` value from the page before. Leave it out for the first page.
    pub cursor: Option<String>,
    /// Only medicines changed at or after this time, oldest change first. It is sent in UTC to
    /// the whole second, like `2026-11-08T09:30:00Z`. [`Medicine::last_updated_at`] reads a
    /// medicine's time in this form.
    pub updated_since: Option<SystemTime>,
    /// Only medicines whose brand, generic or manufacturer starts with these words, or whose ID
    /// matches.
    pub q: Option<String>,
}

/// A client for the Oushodh API.
///
/// It is cheap to clone: clones share one connection pool.
#[derive(Debug, Clone)]
pub struct Client {
    inner: Arc<Inner>,
}

#[derive(Debug)]
struct Inner {
    http: reqwest::Client,
    base: Url,
    key: HeaderValue,
    timeout: Duration,
    max_retries: u32,
}

/// Builds a [`Client`] with options. Start it with [`Client::builder`].
#[derive(Default)]
pub struct ClientBuilder {
    api_key: Option<String>,
    base_url: Option<String>,
    timeout: Option<Duration>,
    max_retries: Option<u32>,
    http: Option<reqwest::Client>,
}

impl std::fmt::Debug for ClientBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientBuilder")
            .field("api_key", &self.api_key.as_ref().map(|_| "[redacted]"))
            .field("base_url", &self.base_url)
            .field("timeout", &self.timeout)
            .field("max_retries", &self.max_retries)
            .finish_non_exhaustive()
    }
}

impl ClientBuilder {
    /// The API key. Without one, [`build`](Self::build) reads `OUSHODH_API_KEY`.
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());

        self
    }

    /// The API's address. The default is `https://api.oushodh.dev`.
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());

        self
    }

    /// How long one request may take. The default is 30 seconds.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);

        self
    }

    /// How many times a request is tried again after a `429 rate_limited`, a `5xx` or a network
    /// error. The default is 2.
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.max_retries = Some(retries);

        self
    }

    /// The HTTP client to send requests with, for a proxy, custom TLS or tests.
    pub fn http_client(mut self, http: reqwest::Client) -> Self {
        self.http = Some(http);

        self
    }

    /// Builds the client.
    ///
    /// Fails with [`Error::MissingKey`] when no key was passed and `OUSHODH_API_KEY` is not set.
    pub fn build(self) -> Result<Client> {
        let key = self
            .api_key
            .or_else(|| std::env::var(API_KEY_ENV).ok())
            .filter(|k| !k.trim().is_empty())
            .ok_or(Error::MissingKey)?;
        let mut key = HeaderValue::from_str(&format!("Bearer {}", key.trim()))
            .map_err(|_| Error::InvalidKey)?;

        key.set_sensitive(true);

        let raw = self
            .base_url
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
        let base = Url::parse(&raw).map_err(|_| Error::InvalidBaseUrl(raw.clone()))?;

        if base.cannot_be_a_base() || !matches!(base.scheme(), "http" | "https") {
            return Err(Error::InvalidBaseUrl(raw));
        }

        Ok(Client {
            inner: Arc::new(Inner {
                http: self.http.unwrap_or_default(),
                base,
                key,
                timeout: self.timeout.unwrap_or(DEFAULT_TIMEOUT),
                max_retries: self.max_retries.unwrap_or(DEFAULT_MAX_RETRIES),
            }),
        })
    }
}

impl Client {
    /// A client with this key and the default options.
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        Self::builder().api_key(api_key).build()
    }

    /// A client with the key in `OUSHODH_API_KEY` and the default options.
    pub fn from_env() -> Result<Self> {
        Self::builder().build()
    }

    /// A builder for a client with options.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    /// The account the key belongs to. Use it to check that a key works.
    pub async fn meta(&self) -> Result<Meta> {
        self.get_json(self.url(&["v1", "meta"], &[])).await
    }

    /// One page of medicines, in the order of their permanent number, or oldest change first with
    /// `updated_since`.
    pub async fn list_medicines(&self, params: &ListParams) -> Result<Page> {
        let limit = params.limit.map(|l| l.to_string());
        let since = params.updated_since.map(time::format);
        let query = [
            ("limit", limit.as_deref()),
            ("cursor", params.cursor.as_deref()),
            ("updated_since", since.as_deref()),
            ("q", params.q.as_deref()),
        ];

        self.get_json(self.url(&["v1", "medicines"], &query)).await
    }

    /// Every medicine that matches `params`, page after page to the end.
    ///
    /// Each page is one request. The stream ends after the first error. It is boxed, so it can be
    /// polled with `try_next` without pinning it first.
    ///
    /// ```no_run
    /// use futures_util::TryStreamExt;
    ///
    /// # async fn run() -> oushodh::Result<()> {
    /// let client = oushodh::Client::from_env()?;
    /// let all: Vec<_> = client.medicines(Default::default()).try_collect().await?;
    /// # let _ = all;
    /// # Ok(())
    /// # }
    /// ```
    pub fn medicines(&self, params: ListParams) -> BoxStream<'static, Result<Medicine>> {
        let client = self.clone();

        stream::try_unfold(Some(params), move |state| {
            let client = client.clone();

            async move {
                let Some(params) = state else {
                    return Ok::<_, Error>(None);
                };
                let page = client.list_medicines(&params).await?;
                let next = page.next.map(|cursor| ListParams {
                    cursor: Some(cursor),
                    ..params
                });

                Ok(Some((page.items, next)))
            }
        })
        .map_ok(|items| stream::iter(items.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }

    /// The best matches for a person typing, best match first. `limit` is 1 to 50; the API's
    /// default is 20.
    pub async fn search(&self, q: &str, limit: Option<u32>) -> Result<SearchResult> {
        let limit = limit.map(|l| l.to_string());
        let query = [("q", Some(q)), ("limit", limit.as_deref())];

        self.get_json(self.url(&["v1", "medicines", "search"], &query))
            .await
    }

    /// One medicine with its medical text, by its ID in any profile, an old ID, or its UUID.
    pub async fn get_medicine(&self, id: &str) -> Result<MedicineDetail> {
        self.get_json(self.url(&["v1", "medicines", id], &[])).await
    }

    /// The address of a picture, such as the file name at the end of `product_image`.
    pub fn image_url(&self, file: &str) -> String {
        self.url(&["v1", "images", file], &[]).into()
    }

    /// The bytes of a JPEG picture. The request carries no key.
    pub async fn get_image(&self, file: &str) -> Result<Vec<u8>> {
        self.send(self.url(&["v1", "images", file], &[]), false)
            .await
    }

    fn url(&self, segments: &[&str], query: &[(&str, Option<&str>)]) -> Url {
        let mut url = self.inner.base.clone();

        if let Ok(mut path) = url.path_segments_mut() {
            path.pop_if_empty().extend(segments);
        }

        let pairs: Vec<_> = query
            .iter()
            .filter_map(|(k, v)| v.map(|v| (*k, v)))
            .collect();

        if !pairs.is_empty() {
            url.query_pairs_mut().extend_pairs(pairs);
        }

        url
    }

    async fn get_json<T: DeserializeOwned>(&self, url: Url) -> Result<T> {
        let body = self.send(url, true).await?;

        Ok(serde_json::from_slice(&body)?)
    }

    async fn send(&self, url: Url, auth: bool) -> Result<Vec<u8>> {
        let mut attempt = 0;

        loop {
            let result = self.send_once(url.clone(), auth).await;
            let wait = match &result {
                Err(Error::RateLimited { error, retry_after })
                    if error.code == Some(ErrorCode::RateLimited) =>
                {
                    Some(retry_after.map_or_else(|| backoff(attempt), Duration::from_secs))
                }
                Err(Error::Server(_) | Error::Network(_) | Error::Timeout(_)) => {
                    Some(backoff(attempt))
                }
                _ => None,
            };

            match wait {
                Some(wait) if attempt < self.inner.max_retries => {
                    tokio::time::sleep(wait).await;
                    attempt += 1;
                }
                _ => return result,
            }
        }
    }

    async fn send_once(&self, url: Url, auth: bool) -> Result<Vec<u8>> {
        let mut request = self
            .inner
            .http
            .get(url)
            .header(USER_AGENT, UA)
            .timeout(self.inner.timeout);

        if auth {
            request = request.header(AUTHORIZATION, self.inner.key.clone());
        }

        let response = request.send().await.map_err(Error::from_reqwest)?;
        let status = response.status().as_u16();
        let retry_after = response
            .headers()
            .get(RETRY_AFTER)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.trim().parse().ok());
        let body = response.bytes().await.map_err(Error::from_reqwest)?;

        if !(200..300).contains(&status) {
            return Err(Error::from_answer(status, &body, retry_after));
        }

        Ok(body.to_vec())
    }
}

fn backoff(attempt: u32) -> Duration {
    let ceiling = 500u64 << attempt.min(6);
    let jitter = RandomState::new().hash_one(attempt) % (ceiling / 2);

    Duration::from_millis(ceiling / 2 + jitter)
}
