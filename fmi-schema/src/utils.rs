//! Utility functions for serde deserialization

use std::{fmt::Display, ops::Deref, str::FromStr};

/// Custom deserializer for `Optional<f64>` that can handle string inputs from JSON
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::AttrList;

    #[test]
    fn attr_list_parses_whitespace_and_formats_canonically() {
        let values = AttrList::<i32>::from_str("  -2\t0\n  17 ").unwrap();

        assert_eq!(&*values, &[-2, 0, 17]);
        assert_eq!(values.to_string(), "-2 0 17");
    }

    #[test]
    fn attr_list_supports_empty_input_and_reports_item_errors() {
        let empty = AttrList::<u32>::from_str(" \t\n ").unwrap();

        assert!(empty.is_empty());
        assert_eq!(empty.to_string(), "");
        assert!(AttrList::<u32>::from_str("1 not-a-number 3").is_err());
    }

    #[cfg(feature = "serde")]
    mod serde_tests {
        use serde::de::value::{Error, F64Deserializer, StrDeserializer};
        use serde::de::{Deserializer, Visitor};

        use super::super::deserialize_optional_f64_from_string;

        enum OptionalDeserializer<D> {
            Some(D),
            None,
        }

        impl<'de, D> Deserializer<'de> for OptionalDeserializer<D>
        where
            D: Deserializer<'de>,
        {
            type Error = D::Error;

            fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where
                V: Visitor<'de>,
            {
                match self {
                    Self::Some(value) => visitor.visit_some(value),
                    Self::None => visitor.visit_none(),
                }
            }

            fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where
                V: Visitor<'de>,
            {
                self.deserialize_any(visitor)
            }

            serde::forward_to_deserialize_any! {
                bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
                bytes byte_buf unit unit_struct newtype_struct seq tuple tuple_struct map
                struct enum identifier ignored_any
            }
        }

        #[test]
        fn optional_f64_accepts_strings_numbers_and_null() {
            let from_string = deserialize_optional_f64_from_string(OptionalDeserializer::Some(
                StrDeserializer::<Error>::new("-12.5e2"),
            ))
            .unwrap();
            let from_number = deserialize_optional_f64_from_string(OptionalDeserializer::Some(
                F64Deserializer::<Error>::new(3.25),
            ))
            .unwrap();
            let from_null = deserialize_optional_f64_from_string(
                OptionalDeserializer::<StrDeserializer<Error>>::None,
            )
            .unwrap();

            assert_eq!(from_string, Some(-1250.0));
            assert_eq!(from_number, Some(3.25));
            assert_eq!(from_null, None);
        }

        #[test]
        fn optional_f64_reports_the_invalid_string() {
            let error = deserialize_optional_f64_from_string(OptionalDeserializer::Some(
                StrDeserializer::<Error>::new("not-a-number"),
            ))
            .unwrap_err();

            assert_eq!(error.to_string(), "Invalid number format: 'not-a-number'");
        }
    }
}
