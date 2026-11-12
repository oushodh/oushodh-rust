# Changelog

All notable changes to this crate are written here. The crate follows
[Semantic Versioning](https://semver.org/).

## 0.1.0 (unreleased)

- First version of the Rust SDK for the Oushodh API `/v1`.
- `Client` with an API key, or `OUSHODH_API_KEY`, and options for the base URL, timeout, retries
  and the HTTP client.
- `meta`, `list_medicines`, `medicines` (a stream to the last page), `search`, `get_medicine`,
  `image_url` and `get_image`.
- Typed models for every object in the OpenAPI spec. Fields this version does not know are kept in
  `extra`.
- One `Error` enum with a variant per error family. `429 rate_limited`, `5xx` and network errors are
  tried again. `429 quota_exceeded` returns at once with `retry_after`.
- `updated_since` takes a `SystemTime` and is sent as `YYYY-MM-DDTHH:MM:SSZ`.
