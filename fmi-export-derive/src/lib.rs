//! Derive macro for FMU model implementation
//!
//! This crate provides the `FmuModel` derive macro for automatically generating
//! FMI 3.0 implementations from annotated Rust structs.

use proc_macro::TokenStream;
use proc_macro_error2::{Diagnostic, proc_macro_error};
use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use syn::{DeriveInput, parse_macro_input};

mod codegen;
mod model;
mod parsing;
mod schema_gen;
mod validation;

use codegen::CodeGenerator;
use model::ExtendedModelInfo;
use parsing::parse_model_input;
use validation::validate_model;

/// Main derive macro for FMU models
///
/// # Example
///
/// ```rust,ignore
/// use fmi_export::FmuModel;
///
/// /// Simple bouncing ball model
/// #[derive(FmuModel, Default)]
/// #[model(ModelExchange)]
/// struct BouncingBall {
///     /// Height above ground (state output)
///     #[variable(causality = output, state = true, start = 1.0)]
///     h: f64,
///
///     /// Velocity of the ball
///     #[variable(causality = output, state = true, start = 0.0)]
///     #[alias(name="der(h)", causality = local, derivative="h")]
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
    // Parse the input into structured model information
    let model_info = parse_model_input(input).map_err(|e| vec![e])?;

    // Create extended model info (with derivatives, etc.)
    let extended_model = ExtendedModelInfo::from_model_info(model_info);

    // Validate the model according to FMI specification
    validate_model(&extended_model)?;

    // Generate the code
    let code_generator = CodeGenerator::new(extended_model);
    let mut tokens = TokenStream2::new();
    code_generator.to_tokens(&mut tokens);

    Ok(tokens)
}
