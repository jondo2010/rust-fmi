#![doc=include_str!( "../README.md")]
#![deny(clippy::all)]

use proc_macro::TokenStream;
use proc_macro_error2::{Diagnostic, proc_macro_error};
use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use syn::{DeriveInput, parse_macro_input};

mod codegen;
mod model;
mod model_structure;
mod model_variables;
mod util;

#[cfg(test)]
mod tests;

use codegen::CodeGenerator;
use model::Model;

//TODO: move this into `fmi` crate?
const RUST_FMI_NAMESPACE: uuid::Uuid = uuid::uuid!("6ba7b810-9dad-11d1-80b4-00c04fd430c8");

/// Main derive macro for FMU models
///
/// # Example
///
/// ```rust,ignore
/// use fmi_export::FmuModel;
///
/// /// Simple bouncing ball model
/// #[derive(FmuModel, Default)]
/// #[model(model_exchange())]
/// struct BouncingBall {
///     /// Height above ground (state output)
///     #[variable(causality = Output, state, start = 1.0)]
///     h: f64,
///
///     /// Velocity of the ball
///     #[variable(causality = Output, state = true, start = 0.0)]
///     #[alias(name="der(h)", causality = Local, derivative = h)]
///     v: f64,
/// }
/// ```
#[proc_macro_derive(FmuModel, attributes(model, variable, alias))]
#[proc_macro_error]
pub fn derive_fmu_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match fmu_model_impl(&input) {
        Ok(expanded) => expanded.into(),
        Err(diagnostics) => {
            // Emit all diagnostics
            for diagnostic in diagnostics {
                diagnostic.emit();
            }
            // Return empty token stream to avoid further compilation errors
            TokenStream::new()
        }
    }
}

/// Implementation of the derive macro
fn fmu_model_impl(input: &DeriveInput) -> Result<TokenStream2, Vec<Diagnostic>> {
    // Check that we have a struct
    let struct_item = match &input.data {
        syn::Data::Struct(struct_data) => {
            // Convert DeriveInput to ItemStruct for our new Model parsing
            let item_struct = syn::ItemStruct {
                attrs: input.attrs.clone(),
                vis: input.vis.clone(),
                struct_token: syn::token::Struct::default(),
                ident: input.ident.clone(),
                generics: input.generics.clone(),
                fields: struct_data.fields.clone(),
                semi_token: struct_data.semi_token,
            };
            item_struct
        }
        _ => {
            return Err(vec![Diagnostic::spanned(
                input.ident.span(),
                proc_macro_error2::Level::Error,
                "FmuModel can only be derived for structs".to_string(),
            )]);
        }
    };

    // Parse the input into the new Model structure
    let model = Model::from(struct_item);

    // Generate the code using the new front-end
    let code_generator = CodeGenerator::new(model);
    let mut tokens = TokenStream2::new();
    code_generator.to_tokens(&mut tokens);

    Ok(tokens)
}
