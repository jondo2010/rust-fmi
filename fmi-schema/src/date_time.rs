//! DateTime support for FMI schema.

/// A wrapper around `chrono::DateTime` that implements `FromStr` for `xsd:dateTime`.
#[derive(Debug, Clone, PartialEq)]
pub struct DateTime(chrono::DateTime<chrono::FixedOffset>);

impl std::fmt::Display for DateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.to_rfc3339().fmt(f)
    }
}

impl std::str::FromStr for DateTime {
    type Err = chrono::format::ParseError;

    // Note:
    // `parse_from_rfc3339` parses an RFC 3339 and ISO 8601 date and time string.
    // XSD follows ISO 8601, which allows no time zone at the end of literal.
    // Since RFC 3339 does not allow such behavior, the function tries to add
    // 'Z' (which equals "+00:00") in case there is no timezone provided.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tz_provided = s.ends_with('Z') || s.contains('+') || s.matches('-').count() == 3;
        let s_with_timezone = if tz_provided {
            s.to_string()
        } else {
            format!("{s}Z")
        };
        match chrono::DateTime::parse_from_rfc3339(&s_with_timezone) {
            Ok(cdt) => Ok(DateTime(cdt)),
            Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::DateTime;

    #[test]
    fn parses_xsd_date_time_without_timezone_as_utc() {
        let date_time = DateTime::from_str("2024-02-29T12:34:56.789").unwrap();

        assert_eq!(date_time.to_string(), "2024-02-29T12:34:56.789+00:00");
    }

    #[test]
    fn preserves_explicit_positive_and_negative_offsets() {
        let positive = DateTime::from_str("2024-02-29T12:34:56+05:30").unwrap();
        let negative = DateTime::from_str("2024-02-29T12:34:56-07:00").unwrap();

        assert_eq!(positive.to_string(), "2024-02-29T12:34:56+05:30");
        assert_eq!(negative.to_string(), "2024-02-29T12:34:56-07:00");
    }

    #[test]
    fn accepts_zulu_timezone_and_rejects_invalid_dates() {
        let zulu = DateTime::from_str("2024-02-29T12:34:56Z").unwrap();

        assert_eq!(zulu.to_string(), "2024-02-29T12:34:56+00:00");
        assert!(DateTime::from_str("2023-02-29T12:34:56Z").is_err());
    }
}
