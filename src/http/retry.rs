//! When to try a request again, and how long to wait first.

use std::hash::{BuildHasher, RandomState};
use std::time::Duration;

use crate::error::{Error, ErrorCode, Result};

/// The wait before the next try, or `None` when the result must not be tried again. Only
/// `rate_limited` is retried: `quota_exceeded` lasts until next month.
pub(super) fn wait(result: &Result<Vec<u8>>, attempt: u32) -> Option<Duration> {
    match result {
        Err(Error::RateLimited { error, retry_after })
            if error.code == Some(ErrorCode::RateLimited) =>
        {
            Some(retry_after.map_or_else(|| backoff(attempt), Duration::from_secs))
        }
        Err(Error::Server(_) | Error::Network(_) | Error::Timeout(_)) => Some(backoff(attempt)),
        _ => None,
    }
}

fn backoff(attempt: u32) -> Duration {
    let ceiling = 500u64 << attempt.min(6);
    let jitter = RandomState::new().hash_one(attempt) % (ceiling / 2);

    Duration::from_millis(ceiling / 2 + jitter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ApiError;

    fn answer(status: u16, code: Option<ErrorCode>) -> ApiError {
        ApiError {
            status,
            code,
            message: String::new(),
        }
    }

    #[test]
    fn rate_limited_waits_retry_after() {
        let result = Err(Error::RateLimited {
            error: answer(429, Some(ErrorCode::RateLimited)),
            retry_after: Some(7),
        });

        assert_eq!(wait(&result, 0), Some(Duration::from_secs(7)));
    }

    #[test]
    fn quota_exceeded_and_404_are_not_retried() {
        let quota = Err(Error::RateLimited {
            error: answer(429, Some(ErrorCode::QuotaExceeded)),
            retry_after: Some(3600),
        });
        let missing = Err(Error::NotFound(answer(404, Some(ErrorCode::NotFound))));

        assert_eq!(wait(&quota, 0), None);
        assert_eq!(wait(&missing, 0), None);
        assert_eq!(wait(&Ok(Vec::new()), 0), None);
    }

    #[test]
    fn server_error_backs_off() {
        let result = Err(Error::Server(answer(503, None)));

        for attempt in 0..4 {
            let ceiling = Duration::from_millis(500 << attempt);
            let wait = wait(&result, attempt).unwrap();

            assert!(wait >= ceiling / 2 && wait < ceiling);
        }
    }
}
