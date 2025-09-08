//! Utility functions for serde deserialization

#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer};

/// Custom deserializer for Optional<f64> that can handle string inputs from JSON
#[cfg(feature = "serde")]
pub fn deserialize_optional_f64_from_string<'de, D>(
    deserializer: D,
) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrF64 {
        String(String),
        F64(f64),
    }

    let value = Option::<StringOrF64>::deserialize(deserializer)?;

    match value {
        Some(StringOrF64::String(s)) => s
            .parse::<f64>()
            .map(Some)
            .map_err(|_| D::Error::custom(format!("Invalid number format: '{}'", s))),
        Some(StringOrF64::F64(f)) => Ok(Some(f)),
        None => Ok(None),
    }
}
