# Oushodh Rust SDK

The official Rust client for the Oushodh API.

[![CI](https://github.com/oushodh/oushodh-rust/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/oushodh/oushodh-rust/actions/workflows/ci.yml)
[![Licence: MIT](https://img.shields.io/github/license/oushodh/oushodh-rust)](LICENSE)
[![crates.io](https://img.shields.io/crates/v/oushodh)](https://crates.io/crates/oushodh)
[![docs.rs](https://img.shields.io/docsrs/oushodh)](https://docs.rs/oushodh)
[![Rust 1.85+](https://img.shields.io/badge/rust-1.85%2B-ce422b)](https://www.rust-lang.org/)

## What Oushodh is

Oushodh serves Bangladesh medicine data over a REST API: brands, generics, prices and makers. Make a key in the member
portal at [app.oushodh.dev](https://app.oushodh.dev), and read the docs at [oushodh.dev/docs](https://oushodh.dev/docs).

The SDK is an async client on [reqwest](https://crates.io/crates/reqwest) with rustls. It needs Rust 1.85 or newer.

## Install

Install it from crates.io. Add it to `Cargo.toml`, with Tokio to run it and `futures-util` to read the paging stream:

```toml
[dependencies]
oushodh = "0.1"
futures-util = "0.3"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Quickstart

Make a key on the Keys page of the member portal. A key starts with `oush_` and is shown once. Keep it on your
server: never put it in a web page or a mobile app.

```sh
export OUSHODH_API_KEY=oush_...
```

```rust,no_run
use oushodh::Client;

#[tokio::main]
async fn main() -> oushodh::Result<()> {
    let client = Client::from_env()?; // reads OUSHODH_API_KEY

    let result = client.search("paracetamol", Some(5)).await?;

    for medicine in &result.items {
        println!("{} {}", medicine.id, medicine.product_name);
    }

    let medicine = client.get_medicine("MED-012933").await?;
    let unit = medicine.prices.unit.as_ref().map(|u| u.amount);

    println!("{}: {unit:?} BDT a unit", medicine.product_name);

    Ok(())
}
```

`Client::new("oush_...")` takes the key directly. With no key and no `OUSHODH_API_KEY`, both return
`Error::MissingKey`.

| Method                    | Route                          | Returns                                   |
| ------------------------- | ------------------------------ | ----------------------------------------- |
| `meta()`                  | `GET /v1/meta`                 | `Result<Meta>`                            |
| `list_medicines(&params)` | `GET /v1/medicines`            | `Result<Page>`                            |
| `medicines(params)`       | `GET /v1/medicines`, each page | a stream of `Result<Medicine>`            |
| `search(q, limit)`        | `GET /v1/medicines/search`     | `Result<SearchResult>`                    |
| `get_medicine(ref)`       | `GET /v1/medicines/{ref}`      | `Result<MedicineDetail>`                  |
| `image_url(file)`         | none                           | the picture's address                     |
| `get_image(file)`         | `GET /v1/images/{file}`        | `Result<Vec<u8>>`, the bytes              |

`meta()` names your account and counts the medicines your plan reads. Call it to check that a key works.

`get_medicine()` takes an ID in any profile, an old ID or a UUID. The answer always carries the medicine's current
ID, and its `monograph`.

Fields keep their API names, such as `product_name`. A value no source gives is `None`. `/v1` only gains fields, and
fields this version does not know are kept in each object's `extra` map.

## Paging

`list_medicines()` returns one page: `items`, `next` and `total`. Pass `next` back as `cursor` for the next page.
`next` is `None` on the last page. `limit` is 1 to 500, and 100 when you leave it out.

```rust,no_run
use oushodh::ListParams;

# async fn run(client: oushodh::Client) -> oushodh::Result<()> {
let mut params = ListParams { limit: Some(500), ..Default::default() };

loop {
    let page = client.list_medicines(&params).await?;

    for medicine in &page.items {
        println!("{} {}", medicine.id, medicine.product_name);
    }

    match page.next {
        Some(next) => params.cursor = Some(next),
        None => break,
    }
}
# Ok(())
# }
```

`medicines()` does the same for you. It is a stream that yields every medicine and follows `next` to the end:

```rust,no_run
use futures_util::TryStreamExt;
use oushodh::ListParams;

# async fn run(client: oushodh::Client) -> oushodh::Result<()> {
let mut all = client.medicines(ListParams { limit: Some(500), ..Default::default() });

while let Some(medicine) = all.try_next().await? {
    println!("{} {}", medicine.id, medicine.product_name);
}
# Ok(())
# }
```

Search has no cursor. To page through every match, use `medicines()` with `q`.

## Keeping a copy in sync

Copy every medicine once, then fetch only what changed.

1. The first sync pages through every medicine and keeps the newest `last_updated`.
2. Each later sync passes that time as `updated_since` and pages to the end. It gets the medicines changed at or after
   that time, oldest change first.

```rust,no_run
use std::time::SystemTime;

use futures_util::TryStreamExt;
use oushodh::ListParams;

# async fn run(client: oushodh::Client, saved: Option<SystemTime>) -> oushodh::Result<()> {
let mut newest = saved; // None on the first sync
let mut changed = client.medicines(ListParams {
    limit: Some(500),
    updated_since: saved,
    ..Default::default()
});

while let Some(medicine) = changed.try_next().await? {
    // Insert or update by medicine.uuid here.
    newest = newest.max(medicine.last_updated_at());
}
// Save `newest` for the next sync.
# Ok(())
# }
```

- `updated_since` is a `SystemTime`. The client sends it in UTC, to the whole second, such as `2026-11-08T09:30:00Z`.
  `last_updated_at()` reads a medicine's `last_updated` as a `SystemTime`.
- "At or after" means a change in the same second is never missed. The medicine that set the time comes back once
  more, so save by `uuid`.
- Store `uuid` as the key: it never changes. Show `id` to people.
- Medicines are never deleted and IDs are never reused, so a sync never has to remove one.

## Images

`product_image` is the address of the medicine's picture, or `None`. `images` lists every picture, `product_image`
first. An address under `/v1/images/` is a copy Oushodh keeps. It needs no key, so an `<img>` tag can show it, and it
never changes. Any other address belongs to the source, and it can move or be removed.

`get_image()` takes the file name, not the whole address, and sends no key:

```rust,no_run
# async fn run(client: oushodh::Client, medicine: oushodh::MedicineDetail) -> oushodh::Result<()> {
if let Some(url) = &medicine.product_image {
    if let Some(file) = url.strip_prefix(&client.image_url("")) {
        let jpeg: Vec<u8> = client.get_image(file).await?;
        # let _ = jpeg;
    }
}
# Ok(())
# }
```

`image_url(file)` builds the address of a file:

```rust
let client = oushodh::Client::new("oush_test_123")?;
let file = "50d858e0985ecc7f60418aaf0cc5ab587f42c2570a884095a9e8ccacd0f6545c.jpg";

assert_eq!(client.image_url(file), format!("https://api.oushodh.dev/v1/images/{file}"));
# assert_eq!(client.image_url(""), "https://api.oushodh.dev/v1/images/");
# Ok::<(), oushodh::Error>(())
```

## Errors

Every call returns `oushodh::Result<T>`. An error answer from the API carries its HTTP status, a `code` and a
`message`. Match on the variant or on `code()`, never on the message: the message is for people and may change.

| Variant               | Status | `code()`                           |
| --------------------- | ------ | ---------------------------------- |
| `Error::BadRequest`   | 400    | `BadRequest`                       |
| `Error::Unauthorized` | 401    | `Unauthorized`                     |
| `Error::Forbidden`    | 403    | `PlanExpired` or `PlanLimit`       |
| `Error::NotFound`     | 404    | `NotFound`                         |
| `Error::RateLimited`  | 429    | `RateLimited` or `QuotaExceeded`   |
| `Error::Server`       | 5xx    | `Internal`                         |
| `Error::Api`          | other  | any                                |
| `Error::Timeout`      | none   | none: the request took too long    |
| `Error::Network`      | none   | none: the request could not be sent |
| `Error::Decode`       | none   | none: the answer was not the JSON expected |

Building a client can also fail with `Error::MissingKey`, `Error::InvalidKey` or `Error::InvalidBaseUrl`. When an
error body is not JSON, `code()` is `None` and the message is `HTTP <status>`. No error, and no `Debug` output of the
client, ever holds the key.

```rust,no_run
use oushodh::{Error, ErrorCode};

# async fn run(client: oushodh::Client) -> oushodh::Result<()> {
match client.get_medicine("MED-999999").await {
    Ok(medicine) => println!("{}", medicine.product_name),
    Err(Error::NotFound(_)) => println!("No medicine has that ID."),
    Err(e) if e.code() == Some(ErrorCode::PlanLimit) => println!("The medicine is beyond your plan."),
    Err(e) => return Err(e),
}
# Ok(())
# }
```

## Retries and rate limits

Your plan sets how many requests each key may make in a minute (60 a minute on Developer). It also sets how many
requests your account may make in a month. See [Rate limits](https://oushodh.dev/docs/rate-limits).

- Over the minute's limit, the API answers `429 rate_limited`. The `Retry-After` header says how many seconds to wait.
  The client waits that long and tries again, up to `max_retries` times. After that it returns
  `Error::RateLimited`, with the seconds in `retry_after()`.
- When the month's requests are used up, the API answers `429 quota_exceeded` until the next month. The client never
  retries it. It returns `Error::RateLimited` at once, and `retry_after()` counts the seconds to the next month.
- After a `5xx`, a network error or a timeout, the client backs off and tries again, up to `max_retries` times. Each
  wait is about twice the last, with a random part.
- The client never retries a `400`, `401`, `403` or `404`: the same request gets the same answer.

Every route is a `GET`, so a retry is always safe.

## Plan limits

Developer reads the first 5,000 medicines and Team the first 20,000, in the order of their permanent number. Agency
and Custom read all of them.

Lists and search leave out the medicines beyond your plan's number. A lookup by ID of such a medicine answers
`403 plan_limit`: the client returns `Error::Forbidden` with `code()` `Some(ErrorCode::PlanLimit)`. An account with no
active plan gets `403 plan_expired` on every call.

## Configuration

`Client::builder()` sets options:

```rust,no_run
use std::time::Duration;

let client = oushodh::Client::builder()
    .api_key("oush_...")
    .timeout(Duration::from_secs(10))
    .max_retries(3)
    .build()?;
# Ok::<(), oushodh::Error>(())
```

| Option        | Default                   | What it does                                                         |
| ------------- | ------------------------- | -------------------------------------------------------------------- |
| `api_key`     | `OUSHODH_API_KEY`         | Your API key.                                                        |
| `base_url`    | `https://api.oushodh.dev` | Where the API is.                                                    |
| `timeout`     | 30 seconds                | How long one request may take.                                       |
| `max_retries` | `2`                       | Retries after a `429 rate_limited`, a `5xx`, a timeout or a network error. |
| `http_client` | a new `reqwest::Client`   | Your own `reqwest::Client`.                                          |

The client is cheap to clone. Clones share one connection pool. In your own tests, point `base_url` at a local mock
server.

## Development

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

`cargo test` also compiles every code sample in this README. The tests use a local mock server. They never call the
API.

## Licence

MIT. Copyright Oushodh. See [LICENSE](LICENSE).
