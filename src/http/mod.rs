//! Sends each request with the key, the User-Agent and the timeout, and retries it.

mod retry;

use std::time::Duration;

use reqwest::Url;
use reqwest::header::{AUTHORIZATION, HeaderValue, RETRY_AFTER, USER_AGENT};

use crate::error::{Error, Result};

const UA: &str = concat!("oushodh-rust/", env!("CARGO_PKG_VERSION"));

#[derive(Debug)]
pub(crate) struct Transport {
    pub(crate) http: reqwest::Client,
    pub(crate) key: HeaderValue,
    pub(crate) timeout: Duration,
    pub(crate) max_retries: u32,
}

impl Transport {
    /// Sends a `GET` to `url` until it succeeds or a retry is not allowed.
    pub(crate) async fn get(&self, url: Url, auth: bool) -> Result<Vec<u8>> {
        let mut attempt = 0;

        loop {
            let result = self.send_once(url.clone(), auth).await;

            match retry::wait(&result, attempt) {
                Some(wait) if attempt < self.max_retries => {
                    tokio::time::sleep(wait).await;
                    attempt += 1;
                }
                _ => return result,
            }
        }
    }

    async fn send_once(&self, url: Url, auth: bool) -> Result<Vec<u8>> {
        let mut request = self
            .http
            .get(url)
            .header(USER_AGENT, UA)
            .timeout(self.timeout);

        if auth {
            request = request.header(AUTHORIZATION, self.key.clone());
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
