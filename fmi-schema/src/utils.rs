//! Utility functions for serde deserialization

use std::{fmt::Display, ops::Deref, str::FromStr};

/// Custom deserializer for Optional<f64> that can handle string inputs from JSON
#[cfg(feature = "serde")]
pub fn deserialize_optional_f64_from_string<'de, D>(
    deserializer: D,
) -> Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
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

/// Newtype for space-separated lists in XML attributes
#[derive(PartialEq, Debug)]
pub struct AttrList<T>(pub Vec<T>);

impl<T> Deref for AttrList<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<T: FromStr> FromStr for AttrList<T> {
    type Err = T::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let items = s
            .split_whitespace()
            .map(|item| item.parse())
            .collect::<Result<Vec<T>, T::Err>>()?;
        Ok(AttrList(items))
    }
}

impl<T: Display> Display for AttrList<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use itertools::Itertools;
        write!(f, "{}", self.0.iter().join(" "))
    }
}
