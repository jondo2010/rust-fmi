/// Trait for parsing start values from syn::Expr
pub trait ParseStartValue: Sized {
    fn parse_literal(lit: &syn::Lit) -> Option<Self>;
}

/// Macro to implement ParseStartValue for numeric types using num-traits
macro_rules! impl_parse_for_numeric {
    (float: $($t:ty),*) => {
        $(
            impl ParseStartValue for $t {
                fn parse_literal(lit: &syn::Lit) -> Option<Self> {
                    match lit {
                        syn::Lit::Float(lit_float) => lit_float.base10_parse().ok(),
                        syn::Lit::Int(lit_int) => lit_int.base10_parse().ok(),
                        _ => None,
                    }
                }
            }
        )*
    };
    (int: $($t:ty),*) => {
        $(
            impl ParseStartValue for $t {
                fn parse_literal(lit: &syn::Lit) -> Option<Self> {
                    match lit {
                        syn::Lit::Int(lit_int) => lit_int.base10_parse().ok(),
                        _ => None,
                    }
                }
            }
        )*
    };
}

// Use the macro to implement for all numeric types
impl_parse_for_numeric!(float: f32, f64);
impl_parse_for_numeric!(int: i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);

/// Specific implementation for bool
impl ParseStartValue for bool {
    fn parse_literal(lit: &syn::Lit) -> Option<Self> {
        match lit {
            syn::Lit::Bool(lit_bool) => Some(lit_bool.value),
            _ => None,
        }
    }
}

/// Specific implementation for String
impl ParseStartValue for String {
    fn parse_literal(lit: &syn::Lit) -> Option<Self> {
        match lit {
            syn::Lit::Str(lit_str) => Some(lit_str.value()),
            _ => None,
        }
    }
}

/// Unified function to parse start values from syn::Expr
pub fn parse_start_value<T: ParseStartValue>(expr: &syn::Expr) -> Vec<T> {
    match expr {
        // Single literal
        syn::Expr::Lit(syn::ExprLit { lit, .. }) => T::parse_literal(lit)
            .map(|v| vec![v])
            .unwrap_or_else(Vec::new),
        // Array literal
        syn::Expr::Array(syn::ExprArray { elems, .. }) => elems
            .iter()
            .filter_map(|elem| {
                if let syn::Expr::Lit(syn::ExprLit { lit, .. }) = elem {
                    T::parse_literal(lit)
                } else {
                    None
                }
            })
            .collect(),
        _ => vec![], // Only support literals and arrays of literals
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_parse_start_values() {
        // Test single literals with different numeric types
        let input: syn::Expr = parse_quote!(42.5);
        let result = parse_start_value::<f64>(&input);
        assert_eq!(result, vec![42.5]);

        let input: syn::Expr = parse_quote!(3.14);
        let result = parse_start_value::<f32>(&input);
        assert_eq!(result, vec![3.14_f32]);

        let input: syn::Expr = parse_quote!(123);
        let result = parse_start_value::<i32>(&input);
        assert_eq!(result, vec![123]);

        let input: syn::Expr = parse_quote!(255);
        let result = parse_start_value::<u8>(&input);
        assert_eq!(result, vec![255_u8]);

        let input: syn::Expr = parse_quote!(9223372036854775807);
        let result = parse_start_value::<i64>(&input);
        assert_eq!(result, vec![9223372036854775807_i64]);

        let input: syn::Expr = parse_quote!(true);
        let result = parse_start_value::<bool>(&input);
        assert_eq!(result, vec![true]);

        let input: syn::Expr = parse_quote!("hello");
        let result = parse_start_value::<String>(&input);
        assert_eq!(result, vec!["hello".to_string()]);

        // Test arrays
        let input: syn::Expr = parse_quote!([0.0, 0.1, 1.0, 1.1, 2.0, 2.1]);
        let result = parse_start_value::<f64>(&input);
        assert_eq!(result, vec![0.0, 0.1, 1.0, 1.1, 2.0, 2.1]);

        let input: syn::Expr = parse_quote!([true, false, true]);
        let result = parse_start_value::<bool>(&input);
        assert_eq!(result, vec![true, false, true]);

        let input: syn::Expr = parse_quote!([1, 2, 3, 4]);
        let result = parse_start_value::<i32>(&input);
        assert_eq!(result, vec![1, 2, 3, 4]);

        let input: syn::Expr = parse_quote!([1, 2, 3, 4]);
        let result = parse_start_value::<u16>(&input);
        assert_eq!(result, vec![1_u16, 2_u16, 3_u16, 4_u16]);

        let input: syn::Expr = parse_quote!(["foo", "bar", "baz"]);
        let result = parse_start_value::<String>(&input);
        assert_eq!(
            result,
            vec!["foo".to_string(), "bar".to_string(), "baz".to_string()]
        );
    }

    #[test]
    fn test_invalid_types() {
        // Test type mismatches return empty vectors
        let input: syn::Expr = parse_quote!("not_a_number");
        let result = parse_start_value::<f64>(&input);
        assert_eq!(result, Vec::<f64>::new());

        let input: syn::Expr = parse_quote!(42);
        let result = parse_start_value::<bool>(&input);
        assert_eq!(result, Vec::<bool>::new());

        let input: syn::Expr = parse_quote!(true);
        let result = parse_start_value::<String>(&input);
        assert_eq!(result, Vec::<String>::new());
    }
}
