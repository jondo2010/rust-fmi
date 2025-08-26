//! Model representation and data structures

use syn::Type;

/// Information about the overall model
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub name: String,
    pub interface_type: String,
    pub description: Option<String>,
    pub variables: Vec<VariableInfo>,
}

/// Information about a variable
#[derive(Debug, Clone)]
pub struct VariableInfo {
    pub name: String,
    pub field_type: Type,
    pub causality: Option<String>,
    pub variability: Option<String>,
    pub initial: Option<String>,
    pub start: Option<String>,
    pub is_state: bool,
    pub description: Option<String>,
    /// If this variable is an alias for another variable, store the target name
    pub alias_of: Option<String>,
    /// If this variable is a derivative of another variable, store the target name
    pub derivative_of: Option<String>,
    /// Aliases defined for this field (additional variable references)
    pub aliases: Vec<AliasInfo>,
}

/// Information about an alias for a field
#[derive(Debug, Clone)]
pub struct AliasInfo {
    pub name: String,
    pub causality: Option<String>,
    pub variability: Option<String>,
    pub initial: Option<String>,
    pub start: Option<String>,
    pub derivative: Option<String>,
    pub description: Option<String>,
}

/// Extended variable information including derived variables
#[derive(Debug, Clone)]
pub struct ExtendedModelInfo {
    /// Original model information
    pub model: ModelInfo,
    /// All variables combined (user + derivatives)
    pub all_variables: Vec<VariableInfo>,
}

impl ExtendedModelInfo {
    pub fn from_model_info(model: ModelInfo) -> Self {
        // For now, just use the user variables as-is
        // Derivative fields are explicitly declared by the user in their struct
        let all_variables = model.variables.clone();

        Self {
            model,
            all_variables,
        }
    }
}
