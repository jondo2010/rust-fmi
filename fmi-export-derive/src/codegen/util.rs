use convert_case::{Case, Casing};

/// Helper function to convert a field name to PascalCase for enum variants
pub fn to_pascal_case(name: &str) -> String {
    // Handle special characters for alias names like "der(h)" -> "DerH"
    let cleaned = name
        .replace("(", "_")
        .replace(")", "")
        .replace("-", "_")
        .replace(" ", "_")
        .replace(".", "_");

    cleaned.to_case(Case::Pascal)
}

/// Generate the variant name for a ValueRef enum from a variable name
pub fn generate_variant_name(name: &str) -> syn::Ident {
    let variant_name = if name.starts_with("der(") && name.ends_with(")") {
        let inner = &name[4..name.len() - 1];
        format!("Der{}", to_pascal_case(inner))
    } else {
        to_pascal_case(name)
    };
    quote::format_ident!("{}", variant_name)
}
