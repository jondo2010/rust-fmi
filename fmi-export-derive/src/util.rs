use fmi::fmi3::schema;

/// Convert a syn::Type to a schema::VariableType
pub fn rust_type_to_variable_type(ty: &syn::Type) -> Result<schema::VariableType, String> {
    match ty {
        syn::Type::Path(type_path) => {
            let type_name = &type_path.path.segments.last().unwrap().ident;
            let type_str = type_name.to_string();

            match type_str.as_str() {
                "f32" => Ok(schema::VariableType::FmiFloat32),
                "f64" => Ok(schema::VariableType::FmiFloat64),
                "i8" => Ok(schema::VariableType::FmiInt8),
                "i16" => Ok(schema::VariableType::FmiInt16),
                "i32" => Ok(schema::VariableType::FmiInt32),
                "i64" => Ok(schema::VariableType::FmiInt64),
                "u8" => Ok(schema::VariableType::FmiUInt8),
                "u16" => Ok(schema::VariableType::FmiUInt16),
                "u32" => Ok(schema::VariableType::FmiUInt32),
                "u64" => Ok(schema::VariableType::FmiUInt64),
                "bool" => Ok(schema::VariableType::FmiBoolean),
                "String" => Ok(schema::VariableType::FmiString),
                _ => Err(format!(
                    "Unsupported field type '{}'. Supported types are: f32, f64, i8, i16, i32, i64, u8, u16, u32, u64, bool, String",
                    type_name
                )),
            }
        }
        _ => Err("Unsupported field type. Only path types are supported.".to_string()),
    }
}

/// Helper function to create InitializableVariable structure
/// Parse numeric start value from syn::Expr
pub fn parse_numeric_start_value<T>(expr: &syn::Expr) -> Vec<T>
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    match expr {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Float(lit_float),
            ..
        }) => {
            if let Ok(value) = lit_float.base10_parse::<T>() {
                vec![value]
            } else {
                vec![]
            }
        }
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(lit_int),
            ..
        }) => {
            if let Ok(value) = lit_int.base10_parse::<T>() {
                vec![value]
            } else {
                vec![]
            }
        }
        _ => vec![], // For now, only support numeric literals
    }
}

/// Parse boolean start value from syn::Expr
pub fn parse_bool_start_value(expr: &syn::Expr) -> Vec<bool> {
    match expr {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Bool(lit_bool),
            ..
        }) => vec![lit_bool.value],
        _ => vec![], // Only support boolean literals
    }
}

/// Parse string start value from syn::Expr
pub fn parse_string_start_value(expr: &syn::Expr) -> Vec<String> {
    match expr {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(lit_str),
            ..
        }) => vec![lit_str.value()],
        _ => vec![], // Only support string literals
    }
}
