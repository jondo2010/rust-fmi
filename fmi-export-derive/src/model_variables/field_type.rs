//! Inner parsing and handling for user struct fields types.
//!
//! We handle:
//! - all scalar Rust types that map to FMI types.
//! - fixed-size arrays of those types, e.g. `[f64; 3]`
//! - nested fixed-size arrays, e.g. `[[f64; 2]; 2]`
//! - TODO: variable-length arrays, `Vec<u16>`
//! - FUTURE: support for 3rd party matrix types from `ndarray` and `faer`

use fmi::fmi3::schema;

#[derive(Debug, PartialEq, Clone)]
pub struct FieldType {
    /// The schema VariableType
    pub r#type: schema::VariableType,
    pub dimensions: Vec<schema::Dimension>,
}

impl TryFrom<syn::Type> for FieldType {
    type Error = String;
    fn try_from(ty: syn::Type) -> Result<Self, Self::Error> {
        match ty {
            syn::Type::Array(type_array) => {
                // Handle array types like [f64; 3] or [[f32; 2]; 3]
                let element_type = &*type_array.elem;
                let array_len = match &type_array.len {
                    syn::Expr::Lit(expr_lit) => match &expr_lit.lit {
                        syn::Lit::Int(lit_int) => lit_int
                            .base10_parse::<u64>()
                            .map_err(|e| format!("Failed to parse array length: {}", e))?,
                        _ => return Err("Array length must be an integer literal".into()),
                    },
                    _ => return Err("Array length must be a literal".into()),
                };

                // Recursively process the element type to get the base type and any inner dimensions
                let mut inner_field_type = FieldType::try_from(element_type.clone())?;

                // Add this array's dimension at the end (innermost dimensions first)
                inner_field_type
                    .dimensions
                    .push(schema::Dimension::Fixed(array_len));

                Ok(inner_field_type)
            }
            syn::Type::Path(type_path) => {
                // Handle scalar types like u8, i32, f64, etc.
                if let Some(segment) = type_path.path.segments.last() {
                    let type_name = segment.ident.to_string();
                    let variable_type = match type_name.as_str() {
                        "f32" => schema::VariableType::FmiFloat32,
                        "f64" => schema::VariableType::FmiFloat64,
                        "i8" => schema::VariableType::FmiInt8,
                        "u8" => schema::VariableType::FmiUInt8,
                        "i16" => schema::VariableType::FmiInt16,
                        "u16" => schema::VariableType::FmiUInt16,
                        "i32" => schema::VariableType::FmiInt32,
                        "u32" => schema::VariableType::FmiUInt32,
                        "i64" => schema::VariableType::FmiInt64,
                        "u64" => schema::VariableType::FmiUInt64,
                        "bool" => schema::VariableType::FmiBoolean,
                        "String" => schema::VariableType::FmiString,
                        "Bytes" => schema::VariableType::FmiBinary,
                        _ => return Err(format!("Unsupported scalar type: {}", type_name)),
                    };

                    Ok(FieldType {
                        r#type: variable_type,
                        dimensions: Vec::new(),
                    })
                } else {
                    Err("Invalid type path".into())
                }
            }
            _ => Err("Unsupported type".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    #[test]
    fn test_scalars() {
        let input: syn::Type = parse_quote! { u8 };
        let field_type = FieldType::try_from(input).unwrap();
        assert_eq!(field_type.r#type, schema::VariableType::FmiUInt8);
        assert_eq!(field_type.dimensions.len(), 0);

        let input: syn::Type = parse_quote! { i32 };
        let field_type = FieldType::try_from(input).unwrap();
        assert_eq!(field_type.r#type, schema::VariableType::FmiInt32);
        assert_eq!(field_type.dimensions.len(), 0);
    }

    #[test]
    fn test_array1() {
        let input: syn::Type = parse_quote! {
            [f64; 3]
        };
        let field_type = FieldType::try_from(input).unwrap();
        assert_eq!(field_type.r#type, schema::VariableType::FmiFloat64);
        assert_eq!(field_type.dimensions.len(), 1);
        assert_eq!(field_type.dimensions[0].as_fixed(), Some(3));
    }

    #[test]
    fn test_array2() {
        let input: syn::Type = parse_quote! {
            [[f32; 2]; 3]
        };
        let field_type = FieldType::try_from(input).unwrap();
        assert_eq!(field_type.r#type, schema::VariableType::FmiFloat32);
        assert_eq!(field_type.dimensions.len(), 2);
        assert_eq!(field_type.dimensions[0].as_fixed(), Some(2));
        assert_eq!(field_type.dimensions[1].as_fixed(), Some(3));
    }
}
