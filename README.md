# oushodh

The Rust SDK for the Oushodh API: medicine data for Bangladesh, checked against the DGDA register.

It is an async client on [reqwest](https://crates.io/crates/reqwest) with rustls. Every call is an
HTTPS `GET` that answers JSON. The full API reference is at <https://oushodh.dev/docs>.

## Install

Install it from GitHub for now:

```toml
[dependencies]
oushodh = { git = "https://github.com/oushodh/oushodh-rust", tag = "v0.1.0" }
futures-util = "0.3"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Until the `v0.1.0` tag exists, use `branch = "main"` in place of `tag`.

Coming soon: `cargo add oushodh` from crates.io.

The SDK needs Rust 1.85 or newer.

## Get a key

1. Sign up at [app.oushodh.dev](https://app.oushodh.dev) and choose a plan.
2. On the Keys page, make a key. It starts with `oush_` and is shown only once.
3. Keep it on your server, never in a web page or a mobile app.

The client reads the key from the `OUSHODH_API_KEY` environment variable, or you pass it in.

## Quickstart

```rust,no_run
use oushodh::Client;

#[tokio::main]
async fn main() -> oushodh::Result<()> {
    let client = Client::from_env()?;

    let meta = client.meta().await?;
    println!("{} reads {} medicines", meta.client, meta.medicines);

    let napa = client.get_medicine("MED-012933").await?;
    println!("{} ({})", napa.product_name, napa.uuid);

    let hits = client.search("napa", Some(5)).await?;
    for medicine in &hits.items {
        println!("{} {}", medicine.id, medicine.product_name);
    }

    Ok(())
}
```

`Client::new("oush_…")` takes the key directly. `Client::builder()` sets options:

```rust,no_run
use std::time::Duration;

let client = oushodh::Client::builder()
    .api_key("oush_…")
    .timeout(Duration::from_secs(10))
    .max_retries(3)
    .build()?;
# Ok::<(), oushodh::Error>(())
```

| Option        | Default                   |
| ------------- | ------------------------- |
| `base_url`    | `https://api.oushodh.dev` |
| `timeout`     | 30 seconds a request      |
| `max_retries` | 2                         |
| `http_client` | a new `reqwest::Client`   |

The client is cheap to clone. Clones share one connection pool.

## Medicines

`get_medicine` takes a medicine's ID in any profile, an old ID, or its UUID. The answer always
carries the medicine's current ID in your profile, and its `monograph`: the medical text, or `None`.

A value no source gives is `None`, never a blank, a zero or an estimate. Store `uuid` as the key that
never changes, and show `id` to people.

Read prices from `prices`. It holds every MRP the sources state: per unit, per strip and per pack.
The older `price`, `price_per` and `unit_prices` fields are kept for older clients.

`/v1` only gains fields. Fields this version of the SDK does not know are kept in each object's
`extra` map.

## Paging

`list_medicines` answers one page. Pass its `next` back as `cursor` for the next page; `next` is
`None` on the last page.

```rust,no_run
use oushodh::ListParams;

# async fn run(client: oushodh::Client) -> oushodh::Result<()> {
let page = client
    .list_medicines(&ListParams { limit: Some(500), ..Default::default() })
    .await?;
println!("{} of {}", page.items.len(), page.total);
# Ok(())
# }
```

`medicines` is a stream that follows `next` to the last page, one request a page:

```rust,no_run
use futures_util::TryStreamExt;
use oushodh::ListParams;

# async fn run(client: oushodh::Client) -> oushodh::Result<()> {
let mut all = client.medicines(ListParams { limit: Some(500), ..Default::default() });

while let Some(medicine) = all.try_next().await? {
    println!("{} {}", medicine.uuid, medicine.product_name);
}
# Ok(())
# }
```

`q` keeps only medicines whose brand, generic or manufacturer starts with the words, or whose ID
matches. `search` gives the best matches for a person typing, best match first, up to 50, with no
cursor.

## Sync

Medicines are never deleted and IDs are never reused, so a copy only ever adds and updates.

1. The first sync pages through every medicine and keeps the newest `last_updated`.
2. Each later sync passes that time as `updated_since` and pages to the end. The answer holds only
   the medicines changed at or after that time, oldest change first.

```rust,no_run
use std::time::SystemTime;

use futures_util::TryStreamExt;
use oushodh::ListParams;

# async fn run(client: oushodh::Client, saved: Option<SystemTime>) -> oushodh::Result<()> {
let mut newest = saved;
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

`updated_since` is a `SystemTime`. The client sends it in UTC to the whole second, like
`2026-11-08T09:30:00Z`. `last_updated_at()` reads a medicine's `last_updated` as a `SystemTime`.

"At or after" means a change in the same second is never missed: the medicine that set the time
comes back once more.

## Errors

Every call returns `oushodh::Result<T>`. An error answer from the API carries its HTTP status, a
`code` and a `message`. Match on the variant or on `code()`, never on the message: it is for people
and may change. When the body of an error answer is not JSON, `code` is `None` and the message is
`HTTP <status>`.

| Variant                       | Status | Codes                             |
| ----------------------------- | ------ | --------------------------------- |
| `Error::BadRequest`           | 400    | `bad_request`                     |
| `Error::Unauthorized`         | 401    | `unauthorized`                    |
| `Error::Forbidden`            | 403    | `plan_expired`, `plan_limit`      |
| `Error::NotFound`             | 404    | `not_found`                       |
| `Error::RateLimited`          | 429    | `rate_limited`, `quota_exceeded`  |
| `Error::Server`               | 5xx    | `internal`                        |
| `Error::Api`                  | other  | any                               |
| `Error::Timeout`              | none   | the request took too long         |
| `Error::Network`              | none   | the request could not be sent     |
| `Error::Decode`               | none   | the answer was not the JSON asked |
| `Error::MissingKey`           | none   | no key and no `OUSHODH_API_KEY`   |

```rust,no_run
use oushodh::{Error, ErrorCode};

# async fn run(client: oushodh::Client) -> oushodh::Result<()> {
match client.get_medicine("MED-999999").await {
    Ok(medicine) => println!("{}", medicine.product_name),
    Err(Error::NotFound(_)) => println!("no such medicine"),
    Err(e) if e.code() == Some(ErrorCode::PlanLimit) => println!("upgrade to read it"),
    Err(e) => return Err(e),
}
# Ok(())
# }
```

No error, and no `Debug` output of the client, ever holds the key.

## Rate limits

Your plan sets how many requests each key may make in a minute (60 a minute on Developer). Over
it, the API answers `429 rate_limited`, and the `Retry-After` header says how many seconds to wait.

The client waits those seconds and tries again, up to `max_retries` times. After a `5xx`, a timeout
or a network error it tries again after an exponential backoff with jitter. It never tries a
`400`, `401`, `403` or `404` again: the same request gets the same answer.

Your plan also sets the requests a month. When they are used up, the API answers
`429 quota_exceeded` until the next month, and `Retry-After` counts the seconds to it. The client
never waits for that and never tries it again. It returns `Error::RateLimited` at once, with
`retry_after()` set to those seconds.

## Plan limits

Some plans read only the first medicines, in the order of their permanent number. Lists and search
leave out the medicines beyond your plan's number, and `meta().medicines` counts the ones you read.
A lookup by ID of a medicine beyond it answers `403 plan_limit`. A `403 plan_expired` means the
account has no active plan.

## Images

`product_image` is the address of a medicine's picture, and `images` lists every picture. Pictures
under `/v1/images` need no key, so a web page can show them directly.

```rust,no_run
# async fn run(client: oushodh::Client) -> oushodh::Result<()> {
let file = "50d858e0985ecc7f60418aaf0cc5ab587f42c2570a884095a9e8ccacd0f6545c.jpg";
let url = client.image_url(file);
let jpeg: Vec<u8> = client.get_image(file).await?;
# let _ = (url, jpeg);
# Ok(())
# }
```

`get_image` sends no key. The file never changes, so it can be cached for a year.

## Docs

- API reference: <https://oushodh.dev/docs>
- OpenAPI spec: <https://oushodh.dev/docs/openapi.json>
- Crate docs: `cargo doc --open`

## License

MIT. Copyright Oushodh.
