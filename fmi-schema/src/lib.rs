use thiserror::Error;

pub mod date_time;
#[cfg(feature = "fmi2")]
pub mod fmi2;
#[cfg(feature = "fmi3")]
pub mod fmi3;
pub mod minimal;

#[derive(Debug, Error)]
pub enum Error {
    #[error("ScalarVariable at index {} not found in Model '{}'.", .1, .0)]
    VariableAtIndexNotFound(String, usize),

    #[error("ScalarVariable '{}' not found in Model '{}'.", name, model)]
    VariableNotFound { model: String, name: String },

    //#[error("Mismatched variable type: expected {0:?} but found {1:?}")]
    //VariableTypeMismatch(ScalarVariableElementBase, ScalarVariableElementBase),
    #[error("ScalarVariable '{}' does not define a derivative.", .0)]
    VariableDerivativeMissing(String),

    #[error(transparent)]
    Semver(#[from] lenient_semver::parser::OwnedError),

    #[error("Error parsing XML: {0}")]
    XmlParse(String),
}
