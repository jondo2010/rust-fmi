use fmi_export::{FmuModel, fmi3::Model};

#[derive(Debug, Default, FmuModel)]
#[model(ModelExchange)]
struct SimpleModel {
    /// A simple output variable
    #[variable(causality = output, start = 42.0)]
    output_value: f64,
    /// A parameter
    #[variable(causality = parameter, start = 1.0)]
    parameter: f64,
}

#[cfg(test)]
mod tests {
    use fmi::fmi3::{Fmi3Res, GetSet};

    use super::*;

    #[test]
    fn test_simple_model_generation() {
        let mut model = SimpleModel::default();
        model.set_start_values();

        // Test initial values
        assert_eq!(model.output_value, 42.0);
        assert_eq!(model.parameter, 1.0);

        // Test calculation
        model.calculate_values();

        dbg!(SimpleModel::model_description());
    }

    #[test]
    fn test_value_ref_enum() {
        // Test that the ValueRef enum was generated correctly
        assert_eq!(ValueRef::OutputValue as u32, 0);
        assert_eq!(ValueRef::Parameter as u32, 1);

        // Test conversions
        let vr: fmi::fmi3::binding::fmi3ValueReference = 0;
        let val_ref = ValueRef::from(vr);
        assert_eq!(val_ref, ValueRef::OutputValue);

        let back_to_vr: fmi::fmi3::binding::fmi3ValueReference = val_ref.into();
        assert_eq!(back_to_vr, 0);
    }

    #[test]
    fn test_getters() {
        let mut model = SimpleModel::default();
        model.set_start_values();

        // Test get_float64
        let vrs = [0u32, 1u32]; // OutputValue and Parameter
        let mut values = [0.0f64; 2];

        let res = model.get_float64(&vrs, &mut values).unwrap();
        assert_eq!(res, Fmi3Res::OK);
        assert_eq!(values[0], 42.0);
        assert_eq!(values[1], 1.0);
    }
}
