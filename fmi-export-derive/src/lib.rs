#![doc=include_str!( "../README.md")]
#![deny(clippy::all)]

use proc_macro::TokenStream;
use proc_macro_error2::proc_macro_error;
use syn::{DeriveInput, parse_macro_input};

mod codegen;
mod model;
mod util;

//#[cfg(test)]
//mod tests;

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
/// #[model()]
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
    let model = Model::from(input);

    proc_macro_error2::abort_if_dirty();

    let code_generator = CodeGenerator::new(model);
    quote::quote! {
        #code_generator
    }
    .into()
}
