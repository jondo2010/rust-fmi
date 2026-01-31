use fmi::fmi3::{binding, schema};

use crate::fmi3::types::{Binary, Clock};

/// Wrapper for start values that can be either scalar or vector
pub enum StartValue<T> {
    Scalar(T),
    Vector(Vec<T>),
}

impl<T> From<T> for StartValue<T> {
    fn from(value: T) -> Self {
        StartValue::Scalar(value)
    }
}

impl<T> From<Vec<T>> for StartValue<T> {
    fn from(value: Vec<T>) -> Self {
        StartValue::Vector(value)
    }
}

impl<T, const N: usize> From<[T; N]> for StartValue<T> {
    fn from(value: [T; N]) -> Self {
        StartValue::Vector(value.into())
    }
}

impl<T: Clone> From<StartValue<T>> for Vec<T> {
    fn from(value: StartValue<T>) -> Self {
        match value {
            StartValue::Scalar(v) => vec![v],
            StartValue::Vector(v) => v,
        }
    }
}

// Special implementation for byte arrays to support Binary start values
impl<const N: usize> From<&[u8; N]> for StartValue<Vec<u8>> {
    fn from(value: &[u8; N]) -> Self {
        StartValue::Scalar(value.to_vec())
    }
}

impl From<&[u8]> for StartValue<Vec<u8>> {
    fn from(value: &[u8]) -> Self {
        StartValue::Scalar(value.to_vec())
    }
}

/// A builder for creating FMI variables with a fluent interface.
///
/// This builder holds all possible FMI variable attributes. The generic type `T`
/// determines which concrete FMI variable type will be created by `finish()`.
/// Type-specific `finish()` implementations will extract only the relevant fields.
pub struct VariableBuilder<T>
where
    T: FmiVariableBuilder,
{
    // Common fields for all variable types
    name: String,
    value_reference: binding::fmi3ValueReference,
    description: Option<String>,
    causality: Option<schema::Causality>,
    variability: Option<schema::Variability>,
    can_handle_multiple_set_per_time_instant: Option<bool>,
    intermediate_update: Option<bool>,
    previous: Option<u32>,
    declared_type: Option<String>,
    initial: Option<schema::Initial>,

    // Type-specific start value
    start: Option<T::Start>,

    // Float-specific fields (for continuous variables)
    derivative: Option<u32>, // Value reference of the state variable this is a derivative of
    reinit: Option<bool>,

    // Float attributes
    min: Option<f64>,
    max: Option<f64>,
    nominal: Option<f64>,

    // Integer attributes
    quantity: Option<String>,

    // Binary attributes
    max_size: Option<usize>,
    mime_type: Option<String>,

    // Clock attributes
    clocks: Option<Vec<u32>>,

    // Dimensions for array variables
    dimensions: Vec<schema::Dimension>,

    _phantom: std::marker::PhantomData<T>,
}

impl<T> VariableBuilder<T>
where
    T: FmiVariableBuilder,
{
    /// Create a new variable builder with the given name.
    pub fn new(name: impl Into<String>, value_reference: binding::fmi3ValueReference) -> Self {
        Self {
            name: name.into(),
            value_reference,
            description: None,
            causality: None,
            variability: None,
            can_handle_multiple_set_per_time_instant: None,
            intermediate_update: None,
            previous: None,
            declared_type: None,
            initial: None,
            start: None,
            derivative: None,
            reinit: None,
            min: None,
            max: None,
            nominal: None,
            quantity: None,
            max_size: None,
            mime_type: None,
            clocks: None,
            dimensions: Vec::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    // Common attribute setters

    /// Set the description for the variable.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the causality for the variable.
    pub fn with_causality(mut self, causality: schema::Causality) -> Self {
        self.causality = Some(causality);
        self
    }

    /// Set the variability for the variable.
    pub fn with_variability(mut self, variability: schema::Variability) -> Self {
        self.variability = Some(variability);
        self
    }

    /// Set whether the variable can handle multiple set operations per time instant.
    pub fn with_can_handle_multiple_set_per_time_instant(mut self, value: bool) -> Self {
        self.can_handle_multiple_set_per_time_instant = Some(value);
        self
    }

    /// Set whether the variable can be updated during intermediate update mode.
    pub fn with_intermediate_update(mut self, value: bool) -> Self {
        self.intermediate_update = Some(value);
        self
    }

    /// Set the value reference of the variable that provides the previous value.
    pub fn with_previous(mut self, value_reference: u32) -> Self {
        self.previous = Some(value_reference);
        self
    }

    /// Set the declared type name from TypeDefinitions.
    pub fn with_declared_type(mut self, type_name: impl Into<String>) -> Self {
        self.declared_type = Some(type_name.into());
        self
    }

    /// Set the initial attribute for the variable.
    pub fn with_initial(mut self, initial: schema::Initial) -> Self {
        self.initial = Some(initial);
        self
    }

    /// Set the start value for the variable.
    pub fn with_start(mut self, start: impl Into<T::Start>) -> Self {
        self.start = Some(start.into());
        self
    }

    // Float-specific attribute setters

    /// Set the derivative relationship for the variable (value reference of state variable).
    pub fn with_derivative(mut self, derivative_vr: u32) -> Self {
        self.derivative = Some(derivative_vr);
        self
    }

    /// Set whether the variable can be reinitialized during event mode.
    pub fn with_reinit(mut self, reinit: bool) -> Self {
        self.reinit = Some(reinit);
        self
    }

    /// Set the minimum value for the variable (float types).
    pub fn with_min(mut self, min: f64) -> Self {
        self.min = Some(min);
        self
    }

    /// Set the maximum value for the variable (float types).
    pub fn with_max(mut self, max: f64) -> Self {
        self.max = Some(max);
        self
    }

    /// Set the nominal value for the variable (float types).
    pub fn with_nominal(mut self, nominal: f64) -> Self {
        self.nominal = Some(nominal);
        self
    }

    // Integer-specific attribute setters

    /// Set the quantity for the variable (integer types).
    pub fn with_quantity(mut self, quantity: impl Into<String>) -> Self {
        self.quantity = Some(quantity.into());
        self
    }

    // Array attribute setters

    /// Add dimensions to the variable (for array types).
    pub fn with_dimensions(mut self, dimensions: Vec<schema::Dimension>) -> Self {
        self.dimensions = dimensions;
        self
    }

    // Binary-specific attribute setters

    /// Set the maximum size for binary variables.
    pub fn with_max_size(mut self, max_size: usize) -> Self {
        self.max_size = Some(max_size);
        self
    }

    /// Set the MIME type for binary variables.
    pub fn with_mime_type(mut self, mime_type: impl Into<String>) -> Self {
        self.mime_type = Some(mime_type.into());
        self
    }

    // Clock-related attribute setters

    /// Set the clocks that this variable belongs to.
    pub fn with_clocks(mut self, clocks: Vec<u32>) -> Self {
        self.clocks = Some(clocks);
        self
    }

    /// Build the final FMI variable.
    ///
    /// This delegates to the type-specific `finish` implementation which will
    /// extract only the relevant fields for the concrete variable type.
    pub fn finish(self) -> T::Var {
        T::finish(self)
    }
}

/// Trait for types that can be used to build FMI variables.
///
/// Implementors define:
/// - `Var`: The concrete FMI variable type to create (e.g., `FmiFloat64`, `FmiBoolean`)
/// - `Start`: The type for start values (typically `StartValue<T>`)
/// - `variable()`: Creates a new builder (default implementation provided)
/// - `finish()`: Type-specific method to extract relevant fields and create the concrete variable
pub trait FmiVariableBuilder: Sized {
    type Var: schema::AbstractVariableTrait;
    type Start;

    /// Create a new variable builder with the given name.
    fn variable(name: impl Into<String>, value_reference: u32) -> VariableBuilder<Self> {
        VariableBuilder::new(name, value_reference)
    }

    /// Type-specific finish method that creates the concrete variable type.
    ///
    /// This method receives the complete builder and extracts only the fields
    /// relevant to the specific variable type being created.
    fn finish(builder: VariableBuilder<Self>) -> Self::Var;
}

// Macro for implementing FmiVariableBuilder for float types (f32, f64)
macro_rules! impl_fmi_variable_builder_float {
    ($primitive_type:ty, $fmi_type:ty, $default_variability:expr) => {
        impl FmiVariableBuilder for $primitive_type {
            type Var = $fmi_type;
            type Start = StartValue<Self>;

            fn finish(builder: VariableBuilder<Self>) -> Self::Var {
                let mut var = <$fmi_type>::new(
                    builder.name,
                    builder.value_reference,
                    if builder.description.as_ref().map_or(true, |d| d.is_empty()) {
                        None
                    } else {
                        builder.description
                    },
                    builder.causality.unwrap_or(schema::Causality::Local),
                    builder.variability.unwrap_or($default_variability),
                    builder.start.map(Into::into),
                    builder.initial,
                );

                // Set float-specific attributes if present
                if let Some(derivative) = builder.derivative {
                    var.derivative = Some(derivative);
                }
                if let Some(reinit) = builder.reinit {
                    var.reinit = Some(reinit);
                }

                // Apply dimensions if any
                if !builder.dimensions.is_empty() {
                    var.dimensions = builder.dimensions;
                }

                var
            }
        }
    };
}

// Macro for implementing FmiVariableBuilder for integer types
macro_rules! impl_fmi_variable_builder_int {
    ($primitive_type:ty, $fmi_type:ty) => {
        impl FmiVariableBuilder for $primitive_type {
            type Var = $fmi_type;
            type Start = StartValue<Self>;

            fn finish(builder: VariableBuilder<Self>) -> Self::Var {
                let mut var = <$fmi_type>::new(
                    builder.name,
                    builder.value_reference,
                    if builder.description.as_ref().map_or(true, |d| d.is_empty()) {
                        None
                    } else {
                        builder.description
                    },
                    builder.causality.unwrap_or(schema::Causality::Local),
                    builder.variability.unwrap_or(schema::Variability::Discrete),
                    builder.start.map(Into::into),
                    builder.initial,
                );

                // Apply dimensions if any
                if !builder.dimensions.is_empty() {
                    var.dimensions = builder.dimensions;
                }

                var
            }
        }
    };
}

impl_fmi_variable_builder_float!(f32, schema::FmiFloat32, schema::Variability::Continuous);
impl_fmi_variable_builder_float!(f64, schema::FmiFloat64, schema::Variability::Continuous);

impl_fmi_variable_builder_int!(i8, schema::FmiInt8);
impl_fmi_variable_builder_int!(u8, schema::FmiUInt8);
impl_fmi_variable_builder_int!(i16, schema::FmiInt16);
impl_fmi_variable_builder_int!(u16, schema::FmiUInt16);
impl_fmi_variable_builder_int!(i32, schema::FmiInt32);
impl_fmi_variable_builder_int!(u32, schema::FmiUInt32);

// Boolean type
impl FmiVariableBuilder for bool {
    type Var = schema::FmiBoolean;
    type Start = StartValue<Self>;

    fn finish(builder: VariableBuilder<Self>) -> Self::Var {
        let mut var = schema::FmiBoolean::new(
            builder.name,
            builder.value_reference,
            if builder.description.as_ref().map_or(true, |d| d.is_empty()) {
                None
            } else {
                builder.description
            },
            builder.causality.unwrap_or(schema::Causality::Local),
            builder.variability.unwrap_or(schema::Variability::Discrete),
            builder.start.map(Into::into),
            builder.initial,
        );

        // Apply dimensions if any
        if !builder.dimensions.is_empty() {
            var.dimensions = builder.dimensions;
        }

        var
    }
}

// String type
impl FmiVariableBuilder for String {
    type Var = schema::FmiString;
    type Start = StartValue<Self>;

    fn finish(builder: VariableBuilder<Self>) -> Self::Var {
        let mut var = schema::FmiString::new(
            builder.name,
            builder.value_reference,
            if builder.description.as_ref().map_or(true, |d| d.is_empty()) {
                None
            } else {
                builder.description
            },
            builder.causality.unwrap_or(schema::Causality::Local),
            builder.variability.unwrap_or(schema::Variability::Discrete),
            builder.start.map(Into::into),
            builder.initial,
        );

        // Apply dimensions if any
        if !builder.dimensions.is_empty() {
            var.dimensions = builder.dimensions;
        }

        var
    }
}

// Array type support - delegates to element type's finish() and adds dimensions
impl<const N: usize, T> FmiVariableBuilder for [T; N]
where
    T: FmiVariableBuilder,
    T::Var: schema::ArrayableVariableTrait,
    T::Start: Into<Vec<T>>,
{
    type Var = T::Var;
    type Start = T::Start;

    fn finish(mut builder: VariableBuilder<Self>) -> Self::Var {
        // Add the fixed dimension for this array
        builder.dimensions.push(schema::Dimension::Fixed(N as _));

        // Delegate to the element type's finish implementation
        // We need to transmute the builder to the element type
        let element_builder = VariableBuilder::<T> {
            name: builder.name,
            value_reference: builder.value_reference,
            description: builder.description,
            causality: builder.causality,
            variability: builder.variability,
            can_handle_multiple_set_per_time_instant: builder
                .can_handle_multiple_set_per_time_instant,
            intermediate_update: builder.intermediate_update,
            previous: builder.previous,
            declared_type: builder.declared_type,
            initial: builder.initial,
            start: builder.start,
            derivative: builder.derivative,
            reinit: builder.reinit,
            min: builder.min,
            max: builder.max,
            nominal: builder.nominal,
            quantity: builder.quantity,
            max_size: builder.max_size,
            mime_type: builder.mime_type,
            clocks: builder.clocks,
            dimensions: builder.dimensions,
            _phantom: std::marker::PhantomData,
        };

        T::finish(element_builder)
    }
}

// Vec type support - similar to array but with variable dimension
impl<T> FmiVariableBuilder for Vec<T>
where
    T: FmiVariableBuilder,
    T::Var: schema::ArrayableVariableTrait,
    T::Start: Into<Vec<T>>,
{
    type Var = T::Var;
    type Start = T::Start;

    fn finish(mut builder: VariableBuilder<Self>) -> Self::Var {
        // Add a variable dimension
        builder.dimensions.push(schema::Dimension::Variable(0));

        // Delegate to the element type's finish implementation
        let element_builder = VariableBuilder::<T> {
            name: builder.name,
            value_reference: builder.value_reference,
            description: builder.description,
            causality: builder.causality,
            variability: builder.variability,
            can_handle_multiple_set_per_time_instant: builder
                .can_handle_multiple_set_per_time_instant,
            intermediate_update: builder.intermediate_update,
            previous: builder.previous,
            declared_type: builder.declared_type,
            initial: builder.initial,
            start: builder.start,
            derivative: builder.derivative,
            reinit: builder.reinit,
            min: builder.min,
            max: builder.max,
            nominal: builder.nominal,
            quantity: builder.quantity,
            max_size: builder.max_size,
            mime_type: builder.mime_type,
            clocks: builder.clocks,
            dimensions: builder.dimensions,
            _phantom: std::marker::PhantomData,
        };

        T::finish(element_builder)
    }
}

impl FmiVariableBuilder for Clock {
    type Var = schema::FmiClock;

    type Start = ();

    fn finish(builder: VariableBuilder<Self>) -> Self::Var {
        let var = schema::FmiClock::new(
            builder.name,
            builder.value_reference,
            builder.description,
            builder.causality.unwrap_or(schema::Causality::Local),
            builder.variability.unwrap_or_default(),
        );

        var
    }
}

impl FmiVariableBuilder for Binary {
    type Var = schema::FmiBinary;
    type Start = StartValue<Vec<u8>>;

    fn finish(builder: VariableBuilder<Self>) -> Self::Var {
        let start_values = builder.start.map(|start| {
            let vec_values: Vec<Vec<u8>> = start.into();
            vec_values
                .into_iter()
                .map(|bytes| {
                    // Simple hex encoding since base64 is not available
                    bytes
                        .iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<String>()
                })
                .collect()
        });

        let mut var = schema::FmiBinary::new(
            builder.name,
            builder.value_reference,
            builder.description,
            builder.causality.unwrap_or(schema::Causality::Local),
            builder.variability.unwrap_or(schema::Variability::Discrete),
            start_values,
            builder.initial,
        );

        // Set max_size if provided
        if let Some(max_size) = builder.max_size {
            var.max_size = Some(max_size as u32);
        }

        // Set mime_type if provided
        if let Some(mime_type) = builder.mime_type {
            var.mime_type = Some(mime_type);
        }

        // Set clocks if provided
        if let Some(clocks) = builder.clocks {
            var.clocks = Some(fmi::schema::utils::AttrList(clocks));
        }

        // Apply dimensions if any
        if !builder.dimensions.is_empty() {
            var.dimensions = builder.dimensions;
        }

        var
    }
}

#[cfg(test)]
mod tests {
    use fmi::schema::fmi3::{AbstractVariableTrait, ArrayableVariableTrait};

    use super::*;

    #[test]
    fn test_scalar() {
        let var_f1 = <f64 as FmiVariableBuilder>::variable("f1", 0)
            .with_description("Description for f1")
            .with_causality(schema::Causality::Parameter)
            .with_variability(schema::Variability::Tunable)
            .with_start(0.0)
            .finish();
        assert_eq!(var_f1.dimensions(), &[]);
    }

    #[test]
    fn test_array1() {
        let var_f2 = <[u16; 2] as FmiVariableBuilder>::variable("f2", 0)
            .with_description("Description for f2")
            .with_causality(schema::Causality::Parameter)
            .with_variability(schema::Variability::Tunable)
            .with_start(vec![0u16, 1])
            .finish();
        assert_eq!(var_f2.dimensions(), &[schema::Dimension::Fixed(2)]);
    }

    #[test]
    fn test_builder_pattern() {
        // Test using the builder pattern with method chaining
        let var = <f64 as FmiVariableBuilder>::variable("test_var", 42)
            .with_description("A test variable")
            .with_causality(schema::Causality::Output)
            .with_variability(schema::Variability::Continuous)
            .with_start(1.5)
            .with_initial(schema::Initial::Exact)
            .finish();

        assert_eq!(var.name(), "test_var");
        assert_eq!(var.value_reference(), 42);
        assert_eq!(var.description(), Some("A test variable"));
        assert_eq!(var.causality(), schema::Causality::Output);
        assert_eq!(var.variability(), schema::Variability::Continuous);
    }

    #[test]
    fn test_builder_with_defaults() {
        // Test that builder provides sensible defaults
        let var = <f64 as FmiVariableBuilder>::variable("minimal", 0)
            .with_start(0.0)
            .finish();

        assert_eq!(var.name(), "minimal");
        assert_eq!(var.value_reference(), 0);
        assert_eq!(var.causality(), schema::Causality::Local);
        assert_eq!(var.variability(), schema::Variability::Continuous);
    }

    #[test]
    fn test_float_specific_attributes() {
        // Test float-specific attributes like derivative and reinit
        let var = <f64 as FmiVariableBuilder>::variable("state_derivative", 10)
            .with_causality(schema::Causality::Local)
            .with_variability(schema::Variability::Continuous)
            .with_derivative(5)  // Value reference of the state variable
            .with_reinit(true)
            .with_min(-100.0)
            .with_max(100.0)
            .with_nominal(10.0)
            .with_start(0.0)
            .with_initial(schema::Initial::Calculated)
            .finish();

        assert_eq!(var.name(), "state_derivative");
        assert_eq!(var.value_reference(), 10);
        assert_eq!(var.derivative(), Some(5));
        assert_eq!(var.reinit(), Some(true));
    }

    #[test]
    fn test_bool_type_creates_fmi_boolean() {
        // Test that bool type creates FmiBoolean
        let var = <bool as FmiVariableBuilder>::variable("flag", 20)
            .with_causality(schema::Causality::Output)
            .with_variability(schema::Variability::Discrete)
            .with_start(false)
            .finish();

        assert_eq!(var.name(), "flag");
        assert_eq!(var.value_reference(), 20);
        assert_eq!(var.causality(), schema::Causality::Output);
        assert_eq!(var.variability(), schema::Variability::Discrete);
    }

    #[test]
    fn test_comprehensive_builder() {
        // Test builder with many optional fields
        let var = <f64 as FmiVariableBuilder>::variable("comprehensive", 100)
            .with_description("A comprehensive test variable")
            .with_causality(schema::Causality::Parameter)
            .with_variability(schema::Variability::Tunable)
            .with_can_handle_multiple_set_per_time_instant(true)
            .with_intermediate_update(false)
            .with_declared_type("CustomFloat64Type")
            .with_start(42.0)
            .with_initial(schema::Initial::Exact)
            .with_min(0.0)
            .with_max(1000.0)
            .with_nominal(100.0)
            .finish();

        assert_eq!(var.name(), "comprehensive");
        assert_eq!(var.value_reference(), 100);
        assert_eq!(var.description(), Some("A comprehensive test variable"));
        assert_eq!(var.causality(), schema::Causality::Parameter);
        assert_eq!(var.variability(), schema::Variability::Tunable);
    }
}
