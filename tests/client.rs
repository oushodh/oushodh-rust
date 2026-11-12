use std::net::TcpListener;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use futures_util::TryStreamExt;
use oushodh::{Client, Error, ErrorCode, ListParams, MedicineType};
use serde_json::{Value, json};
use wiremock::matchers::{header, method, path, query_param, query_param_is_missing};
use wiremock::{Mock, MockServer, ResponseTemplate};

const KEY: &str = "oush_test_123";
const PICTURE: &str = "50d858e0985ecc7f60418aaf0cc5ab587f42c2570a884095a9e8ccacd0f6545c.jpg";

fn client(server: &MockServer) -> Client {
    Client::builder()
        .api_key(KEY)
        .base_url(server.uri())
        .build()
        .unwrap()
}

fn medicine(n: u32) -> Value {
    json!({
        "id": format!("MED-{n:06}"),
        "uuid": format!("01a1ba6f-f4d6-7143-ac84-{n:012}"),
        "brand": "Napa",
        "category": null,
        "product_name": "Napa 500 mg Tablet",
        "product_image": null,
        "images": [],
        "unit": null,
        "size": null,
        "price": 1.2,
        "price_per": "unit",
        "price_source": "dgda_price",
        "unit_prices": [],
        "prices": {
            "currency": "BDT",
            "unit": { "amount": 1.2, "source": "dgda_price" },
            "sizes": [],
            "strip": { "amount": 12.0, "units": 10, "source": "dgda_price" },
            "packs": [{ "amount": 360.0, "units": null, "label": "30's pack", "source": "dgda_price" }]
        },
        "description_en": null,
        "description_bn": null,
        "dosage_form": "Tablet",
        "strength": "500 mg",
        "generic_name": "Paracetamol",
        "manufacturer": "Beximco Pharmaceuticals PLC.",
        "source_url": null,
        "last_updated": "2026-11-09T15:27:34Z",
        "dgda_listed": true,
        "medicine_type": "allopathic"
    })
}

fn nov_8() -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(1_794_130_200)
}

fn error_body(code: &str) -> Value {
    json!({ "error": { "code": code, "message": "Something for people" } })
}

#[tokio::test]
async fn meta_sends_the_key_and_user_agent() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/meta"))
        .and(header("authorization", format!("Bearer {KEY}").as_str()))
        .and(header(
            "user-agent",
            concat!("oushodh-rust/", env!("CARGO_PKG_VERSION")),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "client": "acme",
            "id_profile": "default",
            "medicines": 58937,
            "last_import": "2026-11-09T15:27:40Z",
            "plan": "team"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let meta = client(&server).meta().await.unwrap();

    assert_eq!(meta.client, "acme");
    assert_eq!(meta.medicines, 58937);
    assert_eq!(meta.last_import.as_deref(), Some("2026-11-09T15:27:40Z"));
    assert_eq!(meta.extra["plan"], "team");
}

#[tokio::test]
async fn list_medicines_sends_every_parameter() {
    let server = MockServer::start().await;
    let mut item = medicine(1);

    item["new_field"] = json!("kept");

    Mock::given(method("GET"))
        .and(path("/v1/medicines"))
        .and(query_param("limit", "2"))
        .and(query_param("cursor", "abc"))
        .and(query_param("updated_since", "2026-11-08T09:30:00Z"))
        .and(query_param("q", "napa 500"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "items": [item],
            "next": "def",
            "total": 7
        })))
        .expect(1)
        .mount(&server)
        .await;

    let params = ListParams {
        limit: Some(2),
        cursor: Some("abc".into()),
        updated_since: Some(UNIX_EPOCH + Duration::from_millis(1_794_130_200_750)),
        q: Some("napa 500".into()),
    };
    let page = client(&server).list_medicines(&params).await.unwrap();

    assert_eq!(page.total, 7);
    assert_eq!(page.next.as_deref(), Some("def"));
    assert_eq!(page.items[0].id, "MED-000001");
    assert_eq!(page.items[0].medicine_type, Some(MedicineType::Allopathic));
    assert_eq!(
        page.items[0].prices.strip.as_ref().unwrap().units,
        Some(10.0)
    );
    assert_eq!(page.items[0].prices.packs[0].units, None);
    assert_eq!(page.items[0].extra["new_field"], "kept");
    assert_eq!(
        page.items[0].last_updated_at(),
        Some(UNIX_EPOCH + Duration::from_secs(1_794_238_054))
    );
}

#[tokio::test]
async fn list_medicines_without_parameters_sends_no_query() {
    let server = MockServer::start().await;

    Mock::given(path("/v1/medicines"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "items": [], "next": null, "total": 0 })),
        )
        .mount(&server)
        .await;

    client(&server)
        .list_medicines(&ListParams::default())
        .await
        .unwrap();

    let requests = server.received_requests().await.unwrap();

    assert_eq!(requests[0].url.query(), None);
}

#[tokio::test]
async fn medicines_pages_to_the_end_and_keeps_the_filters() {
    let server = MockServer::start().await;
    let since = "2026-11-08T09:30:00Z";

    Mock::given(path("/v1/medicines"))
        .and(query_param("updated_since", since))
        .and(query_param_is_missing("cursor"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "items": [medicine(1), medicine(2)],
            "next": "c1",
            "total": 3
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(path("/v1/medicines"))
        .and(query_param("updated_since", since))
        .and(query_param("cursor", "c1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "items": [medicine(3)],
            "next": null,
            "total": 3
        })))
        .expect(1)
        .mount(&server)
        .await;

    let params = ListParams {
        updated_since: Some(nov_8()),
        ..Default::default()
    };
    let all: Vec<_> = client(&server)
        .medicines(params)
        .try_collect()
        .await
        .unwrap();
    let ids: Vec<_> = all.iter().map(|m| m.id.as_str()).collect();

    assert_eq!(ids, ["MED-000001", "MED-000002", "MED-000003"]);
}

#[tokio::test]
async fn medicines_stops_at_the_first_error() {
    let server = MockServer::start().await;

    Mock::given(path("/v1/medicines"))
        .respond_with(ResponseTemplate::new(400).set_body_json(error_body("bad_request")))
        .mount(&server)
        .await;

    let result: Result<Vec<_>, _> = client(&server)
        .medicines(ListParams::default())
        .try_collect()
        .await;

    assert!(matches!(result, Err(Error::BadRequest(_))));
}

#[tokio::test]
async fn search_sends_q_and_limit() {
    let server = MockServer::start().await;

    Mock::given(path("/v1/medicines/search"))
        .and(query_param("q", "napa"))
        .and(query_param("limit", "5"))
        .and(header("authorization", format!("Bearer {KEY}").as_str()))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "items": [medicine(1)], "total": 42 })),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = client(&server).search("napa", Some(5)).await.unwrap();

    assert_eq!(result.total, 42);
    assert_eq!(result.items.len(), 1);
}

#[tokio::test]
async fn get_medicine_reads_the_monograph_and_encodes_the_id() {
    let server = MockServer::start().await;
    let mut detail = medicine(12933);

    detail["monograph"] = json!({
        "source": "source_a",
        "indications": "Fever",
        "composition": null,
        "pharmacology": null,
        "dosage": null,
        "interactions": null,
        "contraindications": null,
        "side_effects": null,
        "pregnancy_lactation": null,
        "precautions": null,
        "overdose": null,
        "storage": null
    });

    Mock::given(path("/v1/medicines/MED-012933"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&detail))
        .expect(1)
        .mount(&server)
        .await;
    let mut bare = medicine(1);

    bare["monograph"] = Value::Null;

    Mock::given(path("/v1/medicines/a%2Fb"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&bare))
        .expect(1)
        .mount(&server)
        .await;

    let got = client(&server).get_medicine("MED-012933").await.unwrap();
    let monograph = got.monograph.as_ref().unwrap();

    assert_eq!(got.product_name, "Napa 500 mg Tablet");
    assert_eq!(monograph.source, "source_a");
    assert_eq!(monograph.indications.as_deref(), Some("Fever"));
    assert!(!got.medicine.extra.contains_key("monograph"));

    let bare = client(&server).get_medicine("a/b").await.unwrap();

    assert!(bare.monograph.is_none());
}

#[tokio::test]
async fn images_need_no_key() {
    let server = MockServer::start().await;
    let file = format!("/v1/images/{PICTURE}");

    Mock::given(path(file.as_str()))
        .respond_with(ResponseTemplate::new(200).set_body_raw(vec![0xff, 0xd8, 0xff], "image/jpeg"))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server);
    let bytes = client.get_image(PICTURE).await.unwrap();
    let requests = server.received_requests().await.unwrap();

    assert_eq!(bytes, [0xff, 0xd8, 0xff]);
    assert!(requests[0].headers.get("authorization").is_none());
    assert_eq!(client.image_url(PICTURE), format!("{}{file}", server.uri()));
}

#[tokio::test]
async fn image_url_uses_the_hosted_api_by_default() {
    let client = Client::new(KEY).unwrap();

    assert_eq!(
        client.image_url(PICTURE),
        format!("https://api.oushodh.dev/v1/images/{PICTURE}")
    );
}

#[tokio::test]
async fn a_missing_image_is_not_found() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(404).set_body_json(error_body("not_found")))
        .mount(&server)
        .await;

    let error = client(&server).get_image(PICTURE).await.unwrap_err();

    assert_eq!(error.code(), Some(ErrorCode::NotFound));
}

#[tokio::test]
async fn each_error_family_has_its_variant() {
    let cases = [
        (400, "bad_request"),
        (401, "unauthorized"),
        (403, "plan_expired"),
        (403, "plan_limit"),
        (404, "not_found"),
        (429, "rate_limited"),
        (429, "quota_exceeded"),
        (500, "internal"),
        (418, "something_new"),
    ];

    for (status, code) in cases {
        let server = MockServer::start().await;

        Mock::given(path("/v1/meta"))
            .respond_with(ResponseTemplate::new(status).set_body_json(error_body(code)))
            .mount(&server)
            .await;

        let client = Client::builder()
            .api_key(KEY)
            .base_url(server.uri())
            .max_retries(0)
            .build()
            .unwrap();
        let error = client.meta().await.unwrap_err();
        let family = match &error {
            Error::BadRequest(_) => 400,
            Error::Unauthorized(_) => 401,
            Error::Forbidden(_) => 403,
            Error::NotFound(_) => 404,
            Error::RateLimited { .. } => 429,
            Error::Server(_) => 500,
            Error::Api(_) => 418,
            other => panic!("unexpected {other:?}"),
        };
        let api = error.api_error().unwrap();

        assert_eq!(family, status, "{code}");
        assert_eq!(error.status(), Some(status));
        assert_eq!(api.message, "Something for people");
        assert!(!error.to_string().contains(KEY));
        assert!(!format!("{error:?}").contains(KEY));
    }
}

#[tokio::test]
async fn error_codes_are_read_from_the_body() {
    let server = MockServer::start().await;

    Mock::given(path("/v1/medicines/MED-999999"))
        .respond_with(ResponseTemplate::new(403).set_body_json(error_body("plan_limit")))
        .mount(&server)
        .await;
    Mock::given(path("/v1/meta"))
        .respond_with(ResponseTemplate::new(502).set_body_string("<html>bad gateway</html>"))
        .mount(&server)
        .await;

    let client = Client::builder()
        .api_key(KEY)
        .base_url(server.uri())
        .max_retries(0)
        .build()
        .unwrap();

    assert_eq!(
        client.get_medicine("MED-999999").await.unwrap_err().code(),
        Some(ErrorCode::PlanLimit)
    );
    let error = client.meta().await.unwrap_err();

    assert_eq!(error.code(), None);
    assert_eq!(error.api_error().unwrap().message, "HTTP 502");
}

#[tokio::test]
async fn rate_limited_waits_for_retry_after_then_succeeds() {
    let server = MockServer::start().await;

    Mock::given(path("/v1/meta"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "1")
                .set_body_json(error_body("rate_limited")),
        )
        .up_to_n_times(1)
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(path("/v1/meta"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "client": "acme",
            "id_profile": "default",
            "medicines": 1,
            "last_import": null
        })))
        .expect(1)
        .mount(&server)
        .await;

    let started = Instant::now();
    let meta = client(&server).meta().await.unwrap();

    assert!(started.elapsed() >= Duration::from_secs(1));
    assert_eq!(meta.last_import, None);
}

#[tokio::test]
async fn rate_limited_gives_retry_after_when_retries_run_out() {
    let server = MockServer::start().await;

    Mock::given(path("/v1/meta"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "12")
                .set_body_json(error_body("rate_limited")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = Client::builder()
        .api_key(KEY)
        .base_url(server.uri())
        .max_retries(0)
        .build()
        .unwrap();
    let error = client.meta().await.unwrap_err();

    assert_eq!(error.retry_after(), Some(12));
    assert_eq!(error.code(), Some(ErrorCode::RateLimited));
}

#[tokio::test]
async fn quota_exceeded_is_never_retried() {
    let server = MockServer::start().await;

    Mock::given(path("/v1/meta"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "1500000")
                .set_body_json(error_body("quota_exceeded")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let error = client(&server).meta().await.unwrap_err();

    assert_eq!(error.retry_after(), Some(1_500_000));
    assert_eq!(error.code(), Some(ErrorCode::QuotaExceeded));
}

#[tokio::test]
async fn a_server_error_is_retried_then_succeeds() {
    let server = MockServer::start().await;

    Mock::given(path("/v1/medicines/search"))
        .respond_with(ResponseTemplate::new(503).set_body_json(error_body("internal")))
        .up_to_n_times(1)
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(path("/v1/medicines/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "items": [], "total": 0 })))
        .expect(1)
        .mount(&server)
        .await;

    let result = client(&server).search("napa", None).await.unwrap();

    assert_eq!(result.total, 0);
}

#[tokio::test]
async fn server_errors_stop_after_max_retries() {
    let server = MockServer::start().await;

    Mock::given(path("/v1/meta"))
        .respond_with(ResponseTemplate::new(500).set_body_json(error_body("internal")))
        .expect(3)
        .mount(&server)
        .await;

    let error = client(&server).meta().await.unwrap_err();

    assert!(matches!(error, Error::Server(_)));
}

#[tokio::test]
async fn not_found_is_never_retried() {
    let server = MockServer::start().await;

    Mock::given(path("/v1/medicines/MED-999999"))
        .respond_with(ResponseTemplate::new(404).set_body_json(error_body("not_found")))
        .expect(1)
        .mount(&server)
        .await;

    let error = client(&server)
        .get_medicine("MED-999999")
        .await
        .unwrap_err();

    assert!(matches!(error, Error::NotFound(_)));
}

#[tokio::test]
async fn a_network_error_is_a_network_variant_without_the_key() {
    let port = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let client = Client::builder()
        .api_key(KEY)
        .base_url(format!("http://127.0.0.1:{port}"))
        .max_retries(1)
        .build()
        .unwrap();
    let error = client.meta().await.unwrap_err();

    assert!(matches!(error, Error::Network(_)), "{error:?}");
    assert!(!error.to_string().contains(KEY));
    assert!(!format!("{error:?}").contains(KEY));
}

#[tokio::test]
async fn a_slow_answer_is_a_timeout() {
    let server = MockServer::start().await;

    Mock::given(path("/v1/meta"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_millis(500)))
        .mount(&server)
        .await;

    let client = Client::builder()
        .api_key(KEY)
        .base_url(server.uri())
        .timeout(Duration::from_millis(50))
        .max_retries(0)
        .build()
        .unwrap();
    let error = client.meta().await.unwrap_err();

    assert!(matches!(error, Error::Timeout(_)), "{error:?}");
}

#[tokio::test]
async fn the_key_is_read_from_the_environment() {
    // Only this test touches the variable; every other test passes its key.
    unsafe { std::env::remove_var("OUSHODH_API_KEY") };

    assert!(matches!(Client::from_env(), Err(Error::MissingKey)));

    unsafe { std::env::set_var("OUSHODH_API_KEY", KEY) };

    let client = Client::from_env().unwrap();

    unsafe { std::env::remove_var("OUSHODH_API_KEY") };

    assert!(!format!("{client:?}").contains(KEY));
}

#[test]
fn bad_options_fail_at_construction() {
    assert!(matches!(Client::new(""), Err(Error::MissingKey)));
    assert!(matches!(Client::new("oush_\nabc"), Err(Error::InvalidKey)));
    assert!(matches!(
        Client::builder().api_key(KEY).base_url("not a url").build(),
        Err(Error::InvalidBaseUrl(_))
    ));
    assert!(!format!("{:?}", Client::builder().api_key(KEY)).contains(KEY));
}

#[test]
fn unknown_enum_values_are_kept_as_unknown() {
    let mut item = medicine(1);

    item["medicine_type"] = json!("siddha");

    let parsed: oushodh::Medicine = serde_json::from_value(item).unwrap();

    assert_eq!(parsed.medicine_type, Some(MedicineType::Unknown));
}
