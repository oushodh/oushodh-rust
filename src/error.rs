use serde::Deserialize;

/// The `code` of an error answer. Match on it, never on the message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ErrorCode {
    /// A bad parameter, such as `limit=0`, a time in another form, or a missing `q`.
    BadRequest,
    /// No key, an unknown key or a revoked key. The answer never says which.
    Unauthorized,
    /// The key's account has no active plan.
    PlanExpired,
    /// The medicine is beyond the medicines your plan reads.
    PlanLimit,
    /// No medicine or picture has that name.
    NotFound,
    /// More requests in a minute than the key's plan allows.
    RateLimited,
    /// The plan's requests for this month are used up.
    QuotaExceeded,
    /// A fault on the server.
    Internal,
    /// A code this version does not know.
    #[serde(other)]
    Unknown,
}

/// An error answer from the API: its HTTP status, `code` and `message`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ApiError {
    /// The HTTP status.
    pub status: u16,
    /// The error code. Match on it. `None` when the answer's body is not JSON.
    pub code: Option<ErrorCode>,
    /// For people. Do not match on it; it may change.
    pub message: String,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.code {
            Some(code) => write!(f, "{} {code:?}: {}", self.status, self.message),
            None => write!(f, "{}: {}", self.status, self.message),
        }
    }
}

/// Everything that can go wrong. No variant ever holds the API key.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// No key was passed and `OUSHODH_API_KEY` is not set.
    #[error("no API key: pass one or set OUSHODH_API_KEY")]
    MissingKey,
    /// The key holds characters an HTTP header cannot carry.
    #[error("the API key holds characters an HTTP header cannot carry")]
    InvalidKey,
    /// The base URL cannot be parsed.
    #[error("invalid base URL: {0}")]
    InvalidBaseUrl(String),
    /// `400 bad_request`.
    #[error("bad request: {0}")]
    BadRequest(ApiError),
    /// `401 unauthorized`.
    #[error("unauthorized: {0}")]
    Unauthorized(ApiError),
    /// `403 plan_expired` or `403 plan_limit`.
    #[error("forbidden: {0}")]
    Forbidden(ApiError),
    /// `404 not_found`.
    #[error("not found: {0}")]
    NotFound(ApiError),
    /// `429 rate_limited` or `429 quota_exceeded`.
    #[error("too many requests: {error}")]
    RateLimited {
        /// The error answer.
        error: ApiError,
        /// Seconds to wait, from the `Retry-After` header.
        retry_after: Option<u64>,
    },
    /// A `5xx` answer.
    #[error("server error: {0}")]
    Server(ApiError),
    /// An error answer with another status.
    #[error("unexpected answer: {0}")]
    Api(ApiError),
    /// The request timed out.
    #[error("request timed out")]
    Timeout(#[source] reqwest::Error),
    /// The request could not be sent or the answer could not be read.
    #[error("network error: {0}")]
    Network(#[source] reqwest::Error),
    /// A success answer whose JSON does not match the expected object.
    #[error("cannot read the answer: {0}")]
    Decode(#[from] serde_json::Error),
}

impl Error {
    /// The error answer, when the API sent one.
    pub fn api_error(&self) -> Option<&ApiError> {
        match self {
            Error::BadRequest(e)
            | Error::Unauthorized(e)
            | Error::Forbidden(e)
            | Error::NotFound(e)
            | Error::Server(e)
            | Error::Api(e)
            | Error::RateLimited { error: e, .. } => Some(e),
            _ => None,
        }
    }

    /// The HTTP status, when the API answered.
    pub fn status(&self) -> Option<u16> {
        self.api_error().map(|e| e.status)
    }

    /// The error code, when the API answered with an error body.
    pub fn code(&self) -> Option<ErrorCode> {
        self.api_error().and_then(|e| e.code)
    }

    /// Seconds to wait before trying again, from the `Retry-After` header of a `429`.
    pub fn retry_after(&self) -> Option<u64> {
        match self {
            Error::RateLimited { retry_after, .. } => *retry_after,
            _ => None,
        }
    }

    pub(crate) fn from_reqwest(error: reqwest::Error) -> Self {
        let error = error.without_url();

        if error.is_timeout() {
            return Error::Timeout(error);
        }

        Error::Network(error)
    }

    pub(crate) fn from_answer(status: u16, body: &[u8], retry_after: Option<u64>) -> Self {
        #[derive(Deserialize)]
        struct Body {
            error: Inner,
        }

        #[derive(Deserialize)]
        struct Inner {
            code: ErrorCode,
            message: String,
        }

        let (code, message) = match serde_json::from_slice::<Body>(body) {
            Ok(b) => (Some(b.error.code), b.error.message),
            Err(_) => (None, format!("HTTP {status}")),
        };
        let error = ApiError {
            status,
            code,
            message,
        };

        match status {
            400 => Error::BadRequest(error),
            401 => Error::Unauthorized(error),
            403 => Error::Forbidden(error),
            404 => Error::NotFound(error),
            429 => Error::RateLimited { error, retry_after },
            500..=599 => Error::Server(error),
            _ => Error::Api(error),
        }
    }
}

/// The result of every call.
pub type Result<T> = std::result::Result<T, Error>;
