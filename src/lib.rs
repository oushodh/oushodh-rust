//! The Rust SDK for the [Oushodh API](https://oushodh.dev/docs): medicine data for Bangladesh,
//! checked against the DGDA register.
//!
//! Every call is an HTTPS `GET` that answers JSON. Make a key on the Keys page of the member portal
//! at <https://app.oushodh.dev>, and keep it on your server.
//!
//! ```no_run
//! use futures_util::TryStreamExt;
//! use oushodh::{Client, ListParams};
//!
//! # async fn run() -> oushodh::Result<()> {
//! // Reads the key from OUSHODH_API_KEY.
//! let client = Client::from_env()?;
//!
//! let meta = client.meta().await?;
//! println!("{} reads {} medicines", meta.client, meta.medicines);
//!
//! let napa = client.get_medicine("MED-012933").await?;
//! println!("{} costs {:?} BDT", napa.product_name, napa.prices.unit.as_ref().map(|u| u.amount));
//!
//! let hits = client.search("napa", Some(5)).await?;
//! println!("{} matches", hits.total);
//!
//! let mut all = client.medicines(ListParams { limit: Some(500), ..Default::default() });
//!
//! while let Some(medicine) = all.try_next().await? {
//!     println!("{} {}", medicine.uuid, medicine.product_name);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Errors
//!
//! Every call returns [`Result`]. An error answer from the API carries its HTTP status, a
//! [`ErrorCode`] to match on, and a message for people. See [`Error`].
//!
//! # Retries
//!
//! A `429 rate_limited` waits the seconds in `Retry-After`, then tries again. A `5xx`, a timeout or
//! a network error tries again after an exponential backoff with jitter. Other errors are never
//! tried again. A `429 quota_exceeded` returns at once, with [`Error::retry_after`] set. The default is 2 retries; see
//! [`ClientBuilder::max_retries`].

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]

mod client;
mod error;
pub mod models;
mod time;

pub use client::{API_KEY_ENV, Client, ClientBuilder, DEFAULT_BASE_URL, ListParams};
pub use error::{ApiError, Error, ErrorCode, Result};
pub use models::{Medicine, MedicineDetail, MedicineType, Meta, Monograph, Page, SearchResult};

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;
