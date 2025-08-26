//! Validation logic for FMI models according to the specification

use proc_macro_error2::{Diagnostic, Level};
use proc_macro2::Span;

use crate::model::{ExtendedModelInfo, VariableInfo};
use crate::parsing::{is_float32_type, is_float64_type};

/// Validate the model according to FMI specification
pub fn validate_model(model: &ExtendedModelInfo) -> Result<(), Vec<Diagnostic>> {
    let mut errors = Vec::new();

    // Validate each variable
    for var in &model.all_variables {
        if let Err(mut var_errors) = validate_variable(var) {
            errors.append(&mut var_errors);
        }
    }

    // Validate model-level constraints
    if let Err(mut model_errors) = validate_model_constraints(model) {
        errors.append(&mut model_errors);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Validate a single variable according to FMI specification
fn validate_variable(var: &VariableInfo) -> Result<(), Vec<Diagnostic>> {
    let mut errors = Vec::new();

    // FMI Spec: "A continuous-time state or an event indicator must have causality = local or output"
    if var.is_state {
        match var.causality.as_deref() {
            Some("local") | Some("output") | None => {
                // Valid: explicit local/output causality, or None (defaults to local)
            }
            Some(invalid_causality) => {
                errors.push(Diagnostic::spanned(
                    Span::call_site(),
                    Level::Error,
                    format!(
                        "State variable '{}' has causality '{}' but FMI specification requires state variables to have causality 'local' or 'output'",
                        var.name, invalid_causality
                    ),
                ));
            }
        }
    }

    // Validate causality values
    if let Some(causality) = &var.causality {
        match causality.as_str() {
            "parameter" | "input" | "output" | "local" | "independent" 
            | "calculatedParameter" | "structuralParameter" => {
                // Valid causality
            }
            _ => {
                errors.push(Diagnostic::spanned(
                    Span::call_site(),
                    Level::Error,
                    format!("Variable '{}' has invalid causality '{}'", var.name, causality),
                ));
            }
        }
    }

    // Validate variability values
    if let Some(variability) = &var.variability {
        match variability.as_str() {
            "constant" | "fixed" | "tunable" | "discrete" | "continuous" => {
                // Valid variability
            }
            _ => {
                errors.push(Diagnostic::spanned(
                    Span::call_site(),
                    Level::Error,
                    format!("Variable '{}' has invalid variability '{}'", var.name, variability),
                ));
            }
        }
    }

    // Validate initial values
    if let Some(initial) = &var.initial {
        match initial.as_str() {
            "exact" | "approx" | "calculated" => {
                // Valid initial value
            }
            _ => {
                errors.push(Diagnostic::spanned(
                    Span::call_site(),
                    Level::Error,
                    format!("Variable '{}' has invalid initial '{}'", var.name, initial),
                ));
            }
        }
    }

    // Validate start values for numeric types
    if let Some(start) = &var.start {
        if is_float64_type(&var.field_type) {
            if start.parse::<f64>().is_err() {
                errors.push(Diagnostic::spanned(
                    Span::call_site(),
                    Level::Error,
                    format!("Variable '{}' has invalid f64 start value '{}'", var.name, start),
                ));
            }
        } else if is_float32_type(&var.field_type) {
            if start.parse::<f32>().is_err() {
                errors.push(Diagnostic::spanned(
                    Span::call_site(),
                    Level::Error,
                    format!("Variable '{}' has invalid f32 start value '{}'", var.name, start),
                ));
            }
        }
    }

    // Validate parameter constraints
    if var.causality.as_deref() == Some("parameter") {
        match var.variability.as_deref() {
            Some("fixed") | Some("tunable") | None => {
                // Valid parameter variability
            }
            Some(invalid_variability) => {
                errors.push(Diagnostic::spanned(
                    Span::call_site(),
                    Level::Error,
                    format!(
                        "Parameter variable '{}' has invalid variability '{}'. Parameters must have variability 'fixed' or 'tunable'",
                        var.name, invalid_variability
                    ),
                ));
            }
        }
    }

    // Validate input constraints
    if var.causality.as_deref() == Some("input") {
        if var.variability.as_deref() == Some("constant") {
            errors.push(Diagnostic::spanned(
                Span::call_site(),
                Level::Error,
                format!(
                    "Input variable '{}' cannot have variability 'constant'",
                    var.name
                ),
            ));
        }
    }

    // Validate derivative constraints for aliases
    for alias in &var.aliases {
        if let Some(_derivative_of) = &alias.derivative {
            // Check that the derivative_of variable exists
            // This would need access to all variables, so we'll do it at model level
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Validate model-level constraints
fn validate_model_constraints(model: &ExtendedModelInfo) -> Result<(), Vec<Diagnostic>> {
    let mut errors = Vec::new();

    // Check for duplicate variable names (including aliases)
    let mut all_names = std::collections::HashSet::new();
    
    for var in &model.all_variables {
        if !all_names.insert(&var.name) {
            errors.push(Diagnostic::spanned(
                Span::call_site(),
                Level::Error,
                format!("Duplicate variable name '{}'", var.name),
            ));
        }

        // Check alias names
        for alias in &var.aliases {
            if !all_names.insert(&alias.name) {
                errors.push(Diagnostic::spanned(
                    Span::call_site(),
                    Level::Error,
                    format!("Duplicate variable name '{}' (alias)", alias.name),
                ));
            }
        }
    }

    // Validate derivative relationships
    for var in &model.all_variables {
        for alias in &var.aliases {
            if let Some(derivative_of) = &alias.derivative {
                // Check that the target variable exists
                let target_exists = model.all_variables.iter().any(|v| v.name == *derivative_of);
                if !target_exists {
                    errors.push(Diagnostic::spanned(
                        Span::call_site(),
                        Level::Error,
                        format!(
                            "Derivative alias '{}' references non-existent variable '{}'",
                            alias.name, derivative_of
                        ),
                    ));
                }
            }
        }
    }

    // Validate that we have at least one output variable
    let has_output = model.all_variables.iter().any(|var| {
        var.causality.as_deref() == Some("output")
            || var.aliases.iter().any(|alias| alias.causality.as_deref() == Some("output"))
    });

    if !has_output {
        errors.push(Diagnostic::spanned(
            Span::call_site(),
            Level::Warning,
            "Model has no output variables. Consider adding at least one output.".to_string(),
        ));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
