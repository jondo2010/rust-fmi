use fmi::fmi3::schema;

pub trait FmiVariableBuilder {
    type Var: schema::AbstractVariableTrait;
    type Start;
    fn build(
        name: &str,
        value_reference: u32,
        description: &str,
        causality: schema::Causality,
        variability: schema::Variability,
        start: impl Into<Self::Start>,
        initial: Option<schema::Initial>,
    ) -> Self::Var;
}

macro_rules! impl_fmi_variable_builder {
    ($primitive_type:ty, $fmi_type:ty) => {
        impl FmiVariableBuilder for $primitive_type {
            type Var = $fmi_type;
            type Start = Vec<Self>;
            fn build(
                name: &str,
                value_reference: u32,
                description: &str,
                causality: schema::Causality,
                variability: schema::Variability,
                start: impl Into<Self::Start>,
                initial: Option<schema::Initial>,
            ) -> Self::Var {
                <$fmi_type>::new(
                    name.to_owned(),
                    value_reference,
                    Some(description.to_owned()),
                    causality,
                    variability,
                    Some(start.into()),
                    initial,
                )
            }
        }
    };
}

impl_fmi_variable_builder!(i8, schema::FmiInt8);
impl_fmi_variable_builder!(u8, schema::FmiUInt8);
impl_fmi_variable_builder!(i16, schema::FmiInt16);
impl_fmi_variable_builder!(u16, schema::FmiUInt16);
impl_fmi_variable_builder!(i32, schema::FmiInt32);
impl_fmi_variable_builder!(u32, schema::FmiUInt32);
impl_fmi_variable_builder!(f32, schema::FmiFloat32);
impl_fmi_variable_builder!(f64, schema::FmiFloat64);
impl_fmi_variable_builder!(bool, schema::FmiBoolean);
impl_fmi_variable_builder!(String, schema::FmiString);

impl<const N: usize, T> FmiVariableBuilder for [T; N]
where
    T: FmiVariableBuilder,
    T::Var: schema::ArrayableVariableTrait,
{
    type Var = T::Var;
    type Start = T::Start;
    fn build(
        name: &str,
        value_reference: u32,
        description: &str,
        causality: schema::Causality,
        variability: schema::Variability,
        start: impl Into<Self::Start>,
        initial: Option<schema::Initial>,
    ) -> Self::Var {
        let mut var = <T as FmiVariableBuilder>::build(
            name,
            value_reference,
            description,
            causality,
            variability,
            start,
            initial,
        );
        schema::ArrayableVariableTrait::add_dimensions(
            &mut var,
            &[schema::Dimension::Fixed(N as _)],
        );
        var
    }
}

impl<T> FmiVariableBuilder for Vec<T>
where
    T: FmiVariableBuilder,
    T::Var: schema::ArrayableVariableTrait,
{
    type Var = T::Var;
    type Start = T::Start;
    fn build(
        name: &str,
        value_reference: u32,
        description: &str,
        causality: schema::Causality,
        variability: schema::Variability,
        start: impl Into<Self::Start>,
        initial: Option<schema::Initial>,
    ) -> Self::Var {
        let mut var = <T as FmiVariableBuilder>::build(
            name,
            value_reference,
            description,
            causality,
            variability,
            start,
            initial,
        );
        schema::ArrayableVariableTrait::add_dimensions(
            &mut var,
            &[schema::Dimension::Variable(todo!())],
        );
        var
    }
}

#[cfg(test)]
mod tests {
    use fmi::schema::fmi3::ArrayableVariableTrait;

    use super::*;

    #[test]
    fn test_scalar() {
        let var_f1 = <f64 as FmiVariableBuilder>::build(
            "f1",
            0,
            "Description for f1",
            schema::Causality::Parameter,
            schema::Variability::Tunable,
            vec![0.0],
            None,
        );
        assert_eq!(var_f1.dimensions(), &[]);
    }

    #[test]
    fn test_array1() {
        let var_f2 = <[u16; 2] as FmiVariableBuilder>::build(
            "f2",
            0,
            "Description for f2",
            schema::Causality::Parameter,
            schema::Variability::Tunable,
            vec![0u16, 1],
            None,
        );
        assert_eq!(var_f2.dimensions(), &[schema::Dimension::Fixed(2)]);
    }

    #[test]
    fn test_array2() {
        let var_f3 = <[[f64; 3]; 2] as FmiVariableBuilder>::build(
            "f3",
            0,
            "Description for f3",
            schema::Causality::Parameter,
            schema::Variability::Tunable,
            vec![0.0, 1.0, 2.0, 4.0, 5.0, 6.0],
            None,
        );
        assert_eq!(
            var_f3.dimensions(),
            &[schema::Dimension::Fixed(3), schema::Dimension::Fixed(2)]
        );
    }
}
