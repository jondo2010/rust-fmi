use syn::parse_quote;
use crate::model_new::Model;
use fmi::fmi3::schema;
use fmi_schema::fmi3::{AbstractVariableTrait, Causality, Variability};

#[test]
fn test_comprehensive_datatype_support() {
    // Test that our expanded FMI datatype support works correctly
    let input: syn::ItemStruct = parse_quote! {
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
            #[alias(name = "vel_alias", description = "Velocity alias")]
            velocity_alias: f64,

            /// Counter alias (integer)
            #[variable(causality = Parameter, start = 42)]
            #[alias(name = "count_alias", description = "Counter alias")]  
            counter_alias: i32,
        }
    };

    let model = Model::from(input);
    let fmi_description = schema::Fmi3ModelDescription::try_from(model).unwrap();

    // Test model-level attributes
    assert_eq!(fmi_description.model_name, "ComprehensiveModel");
    assert_eq!(fmi_description.description, Some("A comprehensive FMI model demonstrating all supported datatypes".to_string()));

    // Test variable counts - each alias creates an additional variable
    assert_eq!(fmi_description.model_variables.float32.len(), 1);
    assert_eq!(fmi_description.model_variables.float64.len(), 3); // velocity_f64 + velocity_alias + vel_alias 
    assert_eq!(fmi_description.model_variables.int8.len(), 1);
    assert_eq!(fmi_description.model_variables.int16.len(), 1);
    assert_eq!(fmi_description.model_variables.int32.len(), 3); // count_i32 + counter_alias + count_alias
    assert_eq!(fmi_description.model_variables.int64.len(), 1);
    assert_eq!(fmi_description.model_variables.uint8.len(), 1);
    assert_eq!(fmi_description.model_variables.uint16.len(), 1);
    assert_eq!(fmi_description.model_variables.uint32.len(), 1);
    assert_eq!(fmi_description.model_variables.uint64.len(), 1);
    assert_eq!(fmi_description.model_variables.boolean.len(), 1);
    assert_eq!(fmi_description.model_variables.string.len(), 1);

    // Total should be 16 variables (12 base + 2 aliases + 2 additional variables with aliases = 16)
    assert_eq!(fmi_description.model_variables.len(), 16);

    // Test specific variable properties
    // Float types use Vec<T> for start values
    assert_eq!(fmi_description.model_variables.float32[0].start, vec![1.5]);
    assert_eq!(fmi_description.model_variables.float64[0].start, vec![2.7]);

    // Integer types use Option<T> for start values  
    assert_eq!(fmi_description.model_variables.int8[0].start, Some(10));
    assert_eq!(fmi_description.model_variables.int16[0].start, Some(1000));
    assert_eq!(fmi_description.model_variables.int32[0].start, Some(50000));
    assert_eq!(fmi_description.model_variables.int64[0].start, Some(9000000000));
    assert_eq!(fmi_description.model_variables.uint8[0].start, Some(255));
    assert_eq!(fmi_description.model_variables.uint16[0].start, Some(8080));
    assert_eq!(fmi_description.model_variables.uint32[0].start, Some(1024));
    assert_eq!(fmi_description.model_variables.uint64[0].start, Some(1234567890123));

    // Boolean uses Vec<bool>
    assert_eq!(fmi_description.model_variables.boolean[0].start, vec![true]);

    // String uses Vec<StringStart>
    let string_values: Vec<&str> = fmi_description.model_variables.string[0].start().collect();
    assert_eq!(string_values, vec!["ComprehensiveModel"]);

    // Test causalities are preserved
    assert_eq!(fmi_description.model_variables.float32[0].causality(), Causality::Output);
    assert_eq!(fmi_description.model_variables.int32[0].causality(), Causality::Parameter);
    assert_eq!(fmi_description.model_variables.uint8[0].causality(), Causality::Input);
    assert_eq!(fmi_description.model_variables.boolean[0].causality(), Causality::Input);

    // Test variability assignments
    assert_eq!(fmi_description.model_variables.float32[0].variability(), Variability::Continuous);
    assert_eq!(fmi_description.model_variables.int32[0].variability(), Variability::Discrete);
    assert_eq!(fmi_description.model_variables.boolean[0].variability(), Variability::Discrete);

    println!("✅ All comprehensive datatype tests passed!");
}