//! UTC times in the API's form, `2026-11-08T09:30:00Z`, from and to `SystemTime`.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

const DAY: u64 = 86_400;

/// Whole seconds in UTC. A time before 1970 is written as 1970-01-01T00:00:00Z.
pub(crate) fn format(time: SystemTime) -> String {
    let secs = time
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let (y, m, d) = civil_from_days((secs / DAY) as i64);
    let rest = secs % DAY;

    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rest / 3600,
        rest / 60 % 60,
        rest % 60
    )
}

/// Reads exactly `YYYY-MM-DDTHH:MM:SSZ`, at or after 1970.
pub(crate) fn parse(text: &str) -> Option<SystemTime> {
    let b = text.as_bytes();

    if b.len() != 20
        || b[4] != b'-'
        || b[7] != b'-'
        || b[10] != b'T'
        || b[13] != b':'
        || b[16] != b':'
        || b[19] != b'Z'
    {
        return None;
    }

    let num = |from: usize, to: usize| -> Option<u64> {
        let part = &text[from..to];

        if !part.bytes().all(|c| c.is_ascii_digit()) {
            return None;
        }

        part.parse().ok()
    };
    let (y, m, d) = (num(0, 4)?, num(5, 7)?, num(8, 10)?);
    let (h, min, s) = (num(11, 13)?, num(14, 16)?, num(17, 19)?);

    if y < 1970 || !(1..=12).contains(&m) || !(1..=31).contains(&d) || h > 23 || min > 59 || s > 59
    {
        return None;
    }

    let days = days_from_civil(y as i64, m as i64, d as i64);

    if civil_from_days(days) != (y as i64, m, d) {
        return None;
    }

    Some(UNIX_EPOCH + Duration::from_secs(days as u64 * DAY + h * 3600 + min * 60 + s))
}

// Howard Hinnant's civil-date algorithms.
fn civil_from_days(z: i64) -> (i64, u64, u64) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u64;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u64;

    (yoe + era * 400 + i64::from(m <= 2), m, d)
}

fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;

    era * 146_097 + doe - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_and_parses_whole_seconds() {
        let cases = [
            ("1970-01-01T00:00:00Z", 0),
            ("2000-02-29T12:00:00Z", 951_825_600),
            ("2026-11-08T09:30:00Z", 1_794_130_200),
            ("2026-12-31T23:59:59Z", 1_798_761_599),
        ];

        for (text, secs) in cases {
            let time = UNIX_EPOCH + Duration::from_secs(secs);

            assert_eq!(format(time), text);
            assert_eq!(parse(text), Some(time));
        }

        assert_eq!(
            format(UNIX_EPOCH + Duration::from_millis(1_500)),
            "1970-01-01T00:00:01Z"
        );
    }

    #[test]
    fn rejects_other_forms() {
        for text in [
            "2026-11-08T09:30:00",
            "2026-11-08 09:30:00Z",
            "2026-11-08T09:30:00.5Z",
            "2026-02-30T00:00:00Z",
            "2026-13-01T00:00:00Z",
            "1969-12-31T23:59:59Z",
            "+026-11-08T09:30:00Z",
        ] {
            assert_eq!(parse(text), None, "{text}");
        }
    }
}
