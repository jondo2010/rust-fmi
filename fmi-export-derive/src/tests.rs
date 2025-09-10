use crate::{model::Model, model_variables::build_model_variables};
use fmi::{fmi3::schema, schema::fmi3::InitializableVariableTrait};
use schema::AbstractVariableTrait;
use syn::parse_quote;

#[test]
fn test_comprehensive_datatype_support() {
    // Test that our expanded FMI datatype support works correctly
    let input: syn::DeriveInput = parse_quote! {
        /// A comprehensive FMI model demonstrating all supported datatypes
        #[model()]
        struct ComprehensiveModel {
            // Float types
            /// 32-bit float position
            #[variable(causality = Output, start = 1.5)]
            position_f32: f32,

            /// 64-bit float velocity
            #[variable(causality = Output, start = 2.7)]
            velocity_f64: f64,

            // Signed integer types
            /// 8-bit signed counter
            #[variable(causality = Parameter, start = 10)]
            counter_i8: i8,

            /// 16-bit signed ID
            #[variable(causality = Parameter, start = 1000)]
            id_i16: i16,

            /// 32-bit signed count
            #[variable(causality = Parameter, start = 50000)]
            count_i32: i32,

            /// 64-bit signed large value
            #[variable(causality = Parameter, start = 9000000000)]
            large_value_i64: i64,

            // Unsigned integer types
            /// 8-bit unsigned status
            #[variable(causality = Input, start = 255)]
            status_u8: u8,

            /// 16-bit unsigned port
            #[variable(causality = Input, start = 8080)]
            port_u16: u16,

            /// 32-bit unsigned size
            #[variable(causality = Input, start = 1024)]
            size_u32: u32,

            /// 64-bit unsigned timestamp
            #[variable(causality = Input, start = 1234567890123)]
            timestamp_u64: u64,

            // Boolean type
            /// Enable flag
            #[variable(causality = Input, start = true)]
            enabled: bool,

            // String type
            /// Model name
            #[variable(causality = Parameter, start = "ComprehensiveModel")]
            model_name: String,

            // Test alias functionality with different types
            /// Velocity alias (float)
            #[variable(causality = Output, start = 2.7)]
            #[alias(name = "vel_alias", description = "Velocity alias", causality = Output)]
            velocity_alias: f64,

            /// Counter alias (integer)
            #[variable(causality = Parameter, start = 42)]
            #[alias(name = "count_alias", description = "Counter alias", causality = Parameter)]
            counter_alias: i32,
        }
    };

    let model = Model::from(input);
    let model_variables = build_model_variables(&model.fields);

    // Test variable counts - each alias creates an additional variable (plus automatic time variable)
    assert_eq!(model_variables.float32.len(), 1);
    assert_eq!(model_variables.float64.len(), 4); // velocity_f64 + velocity_alias + vel_alias + time variable
    assert_eq!(model_variables.int8.len(), 1);
    assert_eq!(model_variables.int16.len(), 1);
    assert_eq!(model_variables.int32.len(), 3); // count_i32 + counter_alias + count_alias
    assert_eq!(model_variables.int64.len(), 1);
    assert_eq!(model_variables.uint8.len(), 1);
    assert_eq!(model_variables.uint16.len(), 1);
    assert_eq!(model_variables.uint32.len(), 1);
    assert_eq!(model_variables.uint64.len(), 1);
    assert_eq!(model_variables.boolean.len(), 1);
    assert_eq!(model_variables.string.len(), 1);

    // Total should be 17 variables (12 base + 2 aliases + 2 additional variables with aliases + 1 time variable = 17)
    assert_eq!(model_variables.len(), 17);

    // Test specific variable properties
    // Float types use Vec<T> for start values (skip time variable at index 0)
    assert_eq!(model_variables.float32[0].start, Some(vec![1.5]));
    assert_eq!(model_variables.float64[1].start, Some(vec![2.7])); // First user float64 variable at index 1

    // Integer types use Option<T> for start values
    assert_eq!(model_variables.int8[0].start, Some(vec![10]));
    assert_eq!(model_variables.int16[0].start, Some(vec![1000]));
    assert_eq!(model_variables.int32[0].start, Some(vec![50000]));
    assert_eq!(model_variables.int64[0].start, Some(vec![9000000000]));
    assert_eq!(model_variables.uint8[0].start, Some(vec![255]));
    assert_eq!(model_variables.uint16[0].start, Some(vec![8080]));
    assert_eq!(model_variables.uint32[0].start, Some(vec![1024]));
    assert_eq!(model_variables.uint64[0].start, Some(vec![1234567890123]));

    // Boolean uses Vec<bool>
    assert_eq!(model_variables.boolean[0].start, Some(vec![true]));

    // String uses Vec<StringStart>
    let string_values: Vec<&str> = model_variables.string[0]
        .start()
        .unwrap()
        .iter()
        .map(|s| s.value.as_str())
        .collect();
    assert_eq!(string_values, vec!["ComprehensiveModel"]);

    // Test causalities are preserved
    assert_eq!(
        model_variables.float32[0].causality(),
        schema::Causality::Output
    );
    assert_eq!(
        model_variables.int32[0].causality(),
        schema::Causality::Parameter
    );
    assert_eq!(
        model_variables.uint8[0].causality(),
        schema::Causality::Input
    );
    assert_eq!(
        model_variables.boolean[0].causality(),
        schema::Causality::Input
    );

    // Test variability assignments
    assert_eq!(
        model_variables.float32[0].variability(),
        schema::Variability::Continuous
    );
    assert_eq!(
        model_variables.int32[0].variability(),
        schema::Variability::Discrete
    );
    assert_eq!(
        model_variables.boolean[0].variability(),
        schema::Variability::Discrete
    );
}
