use crate::model::Model;
use fmi::fmi3::schema;
use schema::AbstractVariableTrait;
use syn::parse_quote;

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
    assert_eq!(
        fmi_description.description,
        Some("A comprehensive FMI model demonstrating all supported datatypes".to_string())
    );

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
    assert_eq!(
        fmi_description.model_variables.int64[0].start,
        Some(9000000000)
    );
    assert_eq!(fmi_description.model_variables.uint8[0].start, Some(255));
    assert_eq!(fmi_description.model_variables.uint16[0].start, Some(8080));
    assert_eq!(fmi_description.model_variables.uint32[0].start, Some(1024));
    assert_eq!(
        fmi_description.model_variables.uint64[0].start,
        Some(1234567890123)
    );

    // Boolean uses Vec<bool>
    assert_eq!(fmi_description.model_variables.boolean[0].start, vec![true]);

    // String uses Vec<StringStart>
    let string_values: Vec<&str> = fmi_description.model_variables.string[0].start().collect();
    assert_eq!(string_values, vec!["ComprehensiveModel"]);

    // Test causalities are preserved
    assert_eq!(
        fmi_description.model_variables.float32[0].causality(),
        schema::Causality::Output
    );
    assert_eq!(
        fmi_description.model_variables.int32[0].causality(),
        schema::Causality::Parameter
    );
    assert_eq!(
        fmi_description.model_variables.uint8[0].causality(),
        schema::Causality::Input
    );
    assert_eq!(
        fmi_description.model_variables.boolean[0].causality(),
        schema::Causality::Input
    );

    // Test variability assignments
    assert_eq!(
        fmi_description.model_variables.float32[0].variability(),
        schema::Variability::Continuous
    );
    assert_eq!(
        fmi_description.model_variables.int32[0].variability(),
        schema::Variability::Discrete
    );
    assert_eq!(
        fmi_description.model_variables.boolean[0].variability(),
        schema::Variability::Discrete
    );

    println!("✅ All comprehensive datatype tests passed!");
}

#[test]
fn test_get_set_comprehensive_datatype_support() {
    // This test verifies that our GetSet implementation generates the correct method signatures
    // and compiles properly for all supported datatypes.
    use syn::parse_quote;

    let input: syn::ItemStruct = parse_quote! {
        #[model()]
        struct GetSetTestModel {
            // All supported types for GetSet testing
            #[variable(causality = Output, start = 1.5)]
            test_f32: f32,
            #[variable(causality = Output, start = 2.7)]
            test_f64: f64,
            #[variable(causality = Parameter, start = 10)]
            test_i8: i8,
            #[variable(causality = Parameter, start = 1000)]
            test_i16: i16,
            #[variable(causality = Parameter, start = 50000)]
            test_i32: i32,
            #[variable(causality = Parameter, start = 9000000000)]
            test_i64: i64,
            #[variable(causality = Input, start = 255)]
            test_u8: u8,
            #[variable(causality = Input, start = 8080)]
            test_u16: u16,
            #[variable(causality = Input, start = 1024)]
            test_u32: u32,
            #[variable(causality = Input, start = 1234567890123)]
            test_u64: u64,
            #[variable(causality = Input, start = true)]
            test_bool: bool,
            #[variable(causality = Parameter, start = "TestModel")]
            test_string: String,
        }
    };

    let model = Model::from(input);

    // Test that model description generation works for all types
    let fmi_description = schema::Fmi3ModelDescription::try_from(model.clone()).unwrap();

    // Verify all variable types are present
    assert_eq!(fmi_description.model_variables.float32.len(), 1);
    assert_eq!(fmi_description.model_variables.float64.len(), 1);
    assert_eq!(fmi_description.model_variables.int8.len(), 1);
    assert_eq!(fmi_description.model_variables.int16.len(), 1);
    assert_eq!(fmi_description.model_variables.int32.len(), 1);
    assert_eq!(fmi_description.model_variables.int64.len(), 1);
    assert_eq!(fmi_description.model_variables.uint8.len(), 1);
    assert_eq!(fmi_description.model_variables.uint16.len(), 1);
    assert_eq!(fmi_description.model_variables.uint32.len(), 1);
    assert_eq!(fmi_description.model_variables.uint64.len(), 1);
    assert_eq!(fmi_description.model_variables.boolean.len(), 1);
    assert_eq!(fmi_description.model_variables.string.len(), 1);

    // Total should be 12 variables (one of each type)
    assert_eq!(fmi_description.model_variables.len(), 12);

    // Test that GetSet implementation compiles and generates proper code
    // by verifying the token stream contains all required method implementations
    use crate::codegen::GetSetImpl;
    use quote::ToTokens;

    let get_set_impl = GetSetImpl::new(&model.ident, &model);
    let mut tokens = proc_macro2::TokenStream::new();
    get_set_impl.to_tokens(&mut tokens);

    let code = tokens.to_string();

    // Verify all GetSet method implementations are generated
    assert!(
        code.contains("fn get_boolean"),
        "Missing get_boolean implementation"
    );
    assert!(
        code.contains("fn set_boolean"),
        "Missing set_boolean implementation"
    );
    assert!(
        code.contains("fn get_float32"),
        "Missing get_float32 implementation"
    );
    assert!(
        code.contains("fn set_float32"),
        "Missing set_float32 implementation"
    );
    assert!(
        code.contains("fn get_float64"),
        "Missing get_float64 implementation"
    );
    assert!(
        code.contains("fn set_float64"),
        "Missing set_float64 implementation"
    );
    assert!(
        code.contains("fn get_int8"),
        "Missing get_int8 implementation"
    );
    assert!(
        code.contains("fn set_int8"),
        "Missing set_int8 implementation"
    );
    assert!(
        code.contains("fn get_int16"),
        "Missing get_int16 implementation"
    );
    assert!(
        code.contains("fn set_int16"),
        "Missing set_int16 implementation"
    );
    assert!(
        code.contains("fn get_int32"),
        "Missing get_int32 implementation"
    );
    assert!(
        code.contains("fn set_int32"),
        "Missing set_int32 implementation"
    );
    assert!(
        code.contains("fn get_int64"),
        "Missing get_int64 implementation"
    );
    assert!(
        code.contains("fn set_int64"),
        "Missing set_int64 implementation"
    );
    assert!(
        code.contains("fn get_uint8"),
        "Missing get_uint8 implementation"
    );
    assert!(
        code.contains("fn set_uint8"),
        "Missing set_uint8 implementation"
    );
    assert!(
        code.contains("fn get_uint16"),
        "Missing get_uint16 implementation"
    );
    assert!(
        code.contains("fn set_uint16"),
        "Missing set_uint16 implementation"
    );
    assert!(
        code.contains("fn get_uint32"),
        "Missing get_uint32 implementation"
    );
    assert!(
        code.contains("fn set_uint32"),
        "Missing set_uint32 implementation"
    );
    assert!(
        code.contains("fn get_uint64"),
        "Missing get_uint64 implementation"
    );
    assert!(
        code.contains("fn set_uint64"),
        "Missing set_uint64 implementation"
    );
    assert!(
        code.contains("fn get_string"),
        "Missing get_string implementation"
    );
    assert!(
        code.contains("fn set_string"),
        "Missing set_string implementation"
    );

    // Verify that variable-specific match cases are generated for each field
    assert!(
        code.contains("ValueRef::TestF32"),
        "Missing TestF32 value reference"
    );
    assert!(
        code.contains("ValueRef::TestF64"),
        "Missing TestF64 value reference"
    );
    assert!(
        code.contains("ValueRef::TestI8"),
        "Missing TestI8 value reference"
    );
    assert!(
        code.contains("ValueRef::TestI16"),
        "Missing TestI16 value reference"
    );
    assert!(
        code.contains("ValueRef::TestI32"),
        "Missing TestI32 value reference"
    );
    assert!(
        code.contains("ValueRef::TestI64"),
        "Missing TestI64 value reference"
    );
    assert!(
        code.contains("ValueRef::TestU8"),
        "Missing TestU8 value reference"
    );
    assert!(
        code.contains("ValueRef::TestU16"),
        "Missing TestU16 value reference"
    );
    assert!(
        code.contains("ValueRef::TestU32"),
        "Missing TestU32 value reference"
    );
    assert!(
        code.contains("ValueRef::TestU64"),
        "Missing TestU64 value reference"
    );
    assert!(
        code.contains("ValueRef::TestBool"),
        "Missing TestBool value reference"
    );
    assert!(
        code.contains("ValueRef::TestString"),
        "Missing TestString value reference"
    );

    // Verify field access patterns
    assert!(
        code.contains("self.test_f32"),
        "Missing test_f32 field access"
    );
    assert!(
        code.contains("self.test_f64"),
        "Missing test_f64 field access"
    );
    assert!(
        code.contains("self.test_i8"),
        "Missing test_i8 field access"
    );
    assert!(
        code.contains("self.test_i16"),
        "Missing test_i16 field access"
    );
    assert!(
        code.contains("self.test_i32"),
        "Missing test_i32 field access"
    );
    assert!(
        code.contains("self.test_i64"),
        "Missing test_i64 field access"
    );
    assert!(
        code.contains("self.test_u8"),
        "Missing test_u8 field access"
    );
    assert!(
        code.contains("self.test_u16"),
        "Missing test_u16 field access"
    );
    assert!(
        code.contains("self.test_u32"),
        "Missing test_u32 field access"
    );
    assert!(
        code.contains("self.test_u64"),
        "Missing test_u64 field access"
    );
    assert!(
        code.contains("self.test_bool"),
        "Missing test_bool field access"
    );
    assert!(
        code.contains("self.test_string"),
        "Missing test_string field access"
    );

    println!("✅ All GetSet comprehensive datatype tests passed!");
}
