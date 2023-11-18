pub mod instance;
pub mod model;
pub mod schema;
pub mod binding {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]

    include!(concat!(env!("OUT_DIR"), "/fmi3_bindings.rs"));
}
pub mod import;

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
            format!("{}Z", s)
        };
        match chrono::DateTime::parse_from_rfc3339(&s_with_timezone) {
            Ok(cdt) => Ok(DateTime(cdt)),
            Err(err) => Err(err),
        }
    }
}
