use super::*;

#[test]
fn test_int16() {
    let xml = r#"<Int16 name="Int16_input" valueReference="15" causality="input" start="0"/>"#;
    let var: FmiInt16 = yaserde::de::from_str(xml).unwrap();

    assert_eq!(var.name(), "Int16_input");
    assert_eq!(var.value_reference(), 15);
    assert_eq!(var.causality(), Causality::Input);
    assert_eq!(var.start, Some(vec![0]));
    assert_eq!(var.variability(), Variability::Discrete); // The default for non-float types should be discrete
}

#[test]
fn test_float64() {
    let xml = r#"<Float64
        name="g"
        valueReference="5"
        causality="parameter"
        variability="fixed"
        initial="exact"
        declaredType="Acceleration"
        start="-9.81"
        derivative="1"
        description="Gravity acting on the ball"
    />"#;
    let var: FmiFloat64 = yaserde::de::from_str(xml).unwrap();

    assert_eq!(var.name(), "g");
    assert_eq!(var.value_reference(), 5);
    assert_eq!(var.variability(), Variability::Fixed);
    assert_eq!(var.initial(), Some(Initial::Exact));
    assert_eq!(var.causality(), Causality::Parameter);
    assert_eq!(var.declared_type(), Some("Acceleration"));
    assert_eq!(var.start(), Some([-9.81].as_slice()));
    assert_eq!(var.derivative(), Some(1));
    assert_eq!(var.description(), Some("Gravity acting on the ball"));
    assert_eq!(var.can_handle_multiple_set_per_time_instant(), None);
    assert_eq!(var.intermediate_update(), None);
}

#[test]
fn test_dim_f64() {
    let xml = r#"<Float64
        name="A"
        valueReference="4"
        description="Matrix coefficient A"
        causality="parameter"
        variability="tunable"
        start="1 0 0 0 1 0 0 0 1">
        <Dimension valueReference="2"/>
        <Dimension valueReference="2"/>
        </Float64>"#;

    let var: FmiFloat64 = yaserde::de::from_str(xml).unwrap();
    assert_eq!(var.name(), "A");
    assert_eq!(var.value_reference(), 4);
    assert_eq!(var.variability(), Variability::Tunable);
    assert_eq!(var.causality(), Causality::Parameter);
    assert_eq!(var.description(), Some("Matrix coefficient A"));
    assert_eq!(
        var.start,
        Some(vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0])
    );
    assert_eq!(var.dimensions().len(), 2);
    assert_eq!(var.dimensions()[0].as_variable(), Some(2));
}

#[test]
fn test_string() {
    let xml = r#"<String name="String_parameter" valueReference="29" causality="parameter" variability="fixed">
        <Start value="Set me!"/>
    </String>"#;

    let var: FmiString = yaserde::de::from_str(xml).unwrap();
    assert_eq!(var.name(), "String_parameter");
    assert_eq!(var.value_reference(), 29);
    assert_eq!(var.variability(), Variability::Fixed);
    assert_eq!(var.causality(), Causality::Parameter);
    let start_values: Vec<&str> = var.start().unwrap().iter().map(|s| s.value.as_str()).collect();
    assert_eq!(start_values, vec!["Set me!"]);
}

#[test]
fn test_binary() {
    let xml = r#"
            <Binary name="Binary_input" valueReference="31" causality="input">
                <Start value="666f6f"/>
            </Binary>"#;

    let var: FmiBinary = yaserde::de::from_str(xml).unwrap();
    assert_eq!(var.name(), "Binary_input");
    assert_eq!(var.value_reference(), 31);
    assert_eq!(var.causality(), Causality::Input);
    let start0 = &var.start().unwrap()[0];
    assert_eq!(start0.value.as_str(), "666f6f");
    assert_eq!(start0.as_bytes(), Ok(vec![0x66, 0x6f, 0x6f]));
}

#[test]
fn test_float32() {
    let xml =
        r#"<Float32 name="float32_var" valueReference="10" causality="output" start="3.14"/>"#;
    let var: FmiFloat32 = yaserde::de::from_str(xml).unwrap();

    assert_eq!(var.name(), "float32_var");
    assert_eq!(var.value_reference(), 10);
    assert_eq!(var.causality(), Causality::Output);
    assert_eq!(var.start(), Some([3.14].as_slice()));
    assert_eq!(var.variability(), Variability::Continuous); // Default for float types
    assert_eq!(var.derivative(), None);
    assert_eq!(var.reinit(), None);
}

#[test]
fn test_int8() {
    let xml = r#"<Int8 name="int8_var" valueReference="20" causality="parameter" variability="fixed" start="-128"/>"#;
    let var: FmiInt8 = yaserde::de::from_str(xml).unwrap();

    assert_eq!(var.name(), "int8_var");
    assert_eq!(var.value_reference(), 20);
    assert_eq!(var.causality(), Causality::Parameter);
    assert_eq!(var.start, Some(vec![-128]));
    assert_eq!(var.variability(), Variability::Fixed);
}

#[test]
fn test_uint8() {
    let xml = r#"<UInt8 name="uint8_var" valueReference="21" causality="local" start="255"/>"#;
    let var: FmiUInt8 = yaserde::de::from_str(xml).unwrap();

    assert_eq!(var.name(), "uint8_var");
    assert_eq!(var.value_reference(), 21);
    assert_eq!(var.causality(), Causality::Local);
    assert_eq!(var.start, Some(vec![255]));
    assert_eq!(var.variability(), Variability::Discrete); // Default for integer types
}

#[test]
fn test_uint16() {
    let xml = r#"<UInt16 name="uint16_var" valueReference="22" causality="calculatedParameter" start="65535"/>"#;
    let var: FmiUInt16 = yaserde::de::from_str(xml).unwrap();

    assert_eq!(var.name(), "uint16_var");
    assert_eq!(var.value_reference(), 22);
    assert_eq!(var.causality(), Causality::CalculatedParameter);
    assert_eq!(var.start, Some(vec![65535]));
}

#[test]
fn test_int32() {
    let xml = r#"<Int32 name="int32_var" valueReference="23" causality="structuralParameter" variability="tunable" start="-2147483648"/>"#;
    let var: FmiInt32 = yaserde::de::from_str(xml).unwrap();

    assert_eq!(var.name(), "int32_var");
    assert_eq!(var.value_reference(), 23);
    assert_eq!(var.causality(), Causality::StructuralParameter);
    assert_eq!(var.start, Some(vec![-2147483648]));
    assert_eq!(var.variability(), Variability::Tunable);
}

#[test]
fn test_uint32() {
    let xml = r#"<UInt32 name="uint32_var" valueReference="24" causality="independent" start="4294967295"/>"#;
    let var: FmiUInt32 = yaserde::de::from_str(xml).unwrap();

    assert_eq!(var.name(), "uint32_var");
    assert_eq!(var.value_reference(), 24);
    assert_eq!(var.causality(), Causality::Independent);
    assert_eq!(var.start, Some(vec![4294967295]));
}

#[test]
fn test_int64() {
    let xml = r#"<Int64 name="int64_var" valueReference="25" causality="dependent" start="-9223372036854775808"/>"#;
    let var: FmiInt64 = yaserde::de::from_str(xml).unwrap();

    assert_eq!(var.name(), "int64_var");
    assert_eq!(var.value_reference(), 25);
    assert_eq!(var.causality(), Causality::Dependent);
    assert_eq!(var.start, Some(vec![-9223372036854775808]));
}

#[test]
fn test_uint64() {
    let xml = r#"<UInt64 name="uint64_var" valueReference="26" causality="input" variability="constant" start="18446744073709551615"/>"#;
    let var: FmiUInt64 = yaserde::de::from_str(xml).unwrap();

    assert_eq!(var.name(), "uint64_var");
    assert_eq!(var.value_reference(), 26);
    assert_eq!(var.causality(), Causality::Input);
    assert_eq!(var.start, Some(vec![18446744073709551615]));
    assert_eq!(var.variability(), Variability::Constant);
}

#[test]
fn test_boolean() {
    let xml = r#"<Boolean name="boolean_var" valueReference="30" causality="output" start="true false true"/>"#;
    let var: FmiBoolean = yaserde::de::from_str(xml).unwrap();

    assert_eq!(var.name(), "boolean_var");
    assert_eq!(var.value_reference(), 30);
    assert_eq!(var.causality(), Causality::Output);
    assert_eq!(var.start, Some(vec![true, false, true]));
    assert_eq!(var.variability(), Variability::Discrete); // Default for boolean
}

#[test]
fn test_variable_with_all_attributes() {
    let xml = r#"<Float64
            name="complex_var"
            valueReference="100"
            description="A complex variable with many attributes"
            causality="output"
            variability="continuous"
            canHandleMultipleSetPerTimeInstant="true"
            intermediateUpdate="false"
            previous="99"
            initial="calculated"
            declaredType="CustomType"
            start="1.0 2.0"
            derivative="101"
            reinit="true">
            <Dimension start="2"/>
        </Float64>"#;

    let var: FmiFloat64 = yaserde::de::from_str(xml).unwrap();
    assert_eq!(var.name(), "complex_var");
    assert_eq!(var.value_reference(), 100);
    assert_eq!(
        var.description(),
        Some("A complex variable with many attributes")
    );
    assert_eq!(var.causality(), Causality::Output);
    assert_eq!(var.variability(), Variability::Continuous);
    assert_eq!(var.can_handle_multiple_set_per_time_instant(), Some(true));
    assert_eq!(var.intermediate_update(), Some(false));
    assert_eq!(var.previous(), Some(99));
    assert_eq!(var.initial(), Some(Initial::Calculated));
    assert_eq!(var.declared_type(), Some("CustomType"));
    assert_eq!(var.start(), Some([1.0, 2.0].as_slice()));
    assert_eq!(var.derivative(), Some(101));
    assert_eq!(var.reinit(), Some(true));
    assert_eq!(var.dimensions().len(), 1);
    assert_eq!(var.dimensions()[0].as_fixed(), Some(2));
}

#[test]
fn test_dimension_with_value_reference() {
    let xml = r#"<Float32
            name="matrix_var"
            valueReference="200"
            causality="parameter"
            start="1.0 2.0 3.0 4.0">
            <Dimension valueReference="201"/>
            <Dimension start="2"/>
        </Float32>"#;

    let var: FmiFloat32 = yaserde::de::from_str(xml).unwrap();
    assert_eq!(var.name(), "matrix_var");
    assert_eq!(var.dimensions().len(), 2);
    assert_eq!(var.dimensions()[0].as_variable(), Some(201));
    assert_eq!(var.dimensions()[0].as_fixed(), None);
    assert_eq!(var.dimensions()[1].as_variable(), None);
    assert_eq!(var.dimensions()[1].as_fixed(), Some(2));
    assert_eq!(var.start(), Some([1.0, 2.0, 3.0, 4.0].as_slice()));
}

#[test]
fn test_string_multiple_starts() {
    let xml = r#"<String name="multi_string" valueReference="300" causality="parameter">
            <Start value="First string"/>
            <Start value="Second string"/>
            <Start value="Third string"/>
        </String>"#;

    let var: FmiString = yaserde::de::from_str(xml).unwrap();
    assert_eq!(var.name(), "multi_string");
    let start_values: Vec<&str> = var.start().unwrap().iter().map(|s| s.value.as_str()).collect();
    assert_eq!(
        start_values,
        vec!["First string", "Second string", "Third string"]
    );
}

#[test]
fn test_binary_multiple_starts_and_attributes() {
    let xml = r#"<Binary
            name="multi_binary"
            valueReference="400"
            causality="input"
            mimeType="application/custom"
            maxSize="1024">
            <Start value="48656c6c6f"/>
            <Start value="576f726c64"/>
        </Binary>"#;

    let var: FmiBinary = yaserde::de::from_str(xml).unwrap();
    assert_eq!(var.name(), "multi_binary");
    assert_eq!(var.mime_type, "application/custom");
    assert_eq!(var.max_size, Some(1024));

    let start_values: Vec<&BinaryStart> = var.start().unwrap().iter().collect();
    assert_eq!(start_values.len(), 2);
    assert_eq!(start_values[0].value, "48656c6c6f");
    assert_eq!(start_values[1].value, "576f726c64");

    // Test hex parsing
    assert_eq!(
        start_values[0].as_bytes(),
        Ok(vec![0x48, 0x65, 0x6c, 0x6c, 0x6f])
    ); // "Hello"
    assert_eq!(
        start_values[1].as_bytes(),
        Ok(vec![0x57, 0x6f, 0x72, 0x6c, 0x64])
    ); // "World"
}

#[test]
fn test_binary_hex_parsing_with_prefix() {
    let xml = r#"<Binary name="hex_binary" valueReference="500" causality="input">
            <Start value="0x48656C6C6F"/>
        </Binary>"#;

    let var: FmiBinary = yaserde::de::from_str(xml).unwrap();
    let start0 = &var.start().unwrap()[0];
    assert_eq!(start0.as_bytes(), Ok(vec![0x48, 0x65, 0x6C, 0x6C, 0x6F])); // "HeLLO"
}

#[test]
fn test_binary_hex_parsing_with_whitespace() {
    let xml = r#"<Binary name="spaced_binary" valueReference="600" causality="input">
            <Start value="48 65 6c 6c 6f 20 57 6f 72 6c 64"/>
        </Binary>"#;

    let var: FmiBinary = yaserde::de::from_str(xml).unwrap();
    let start0 = &var.start().unwrap()[0];
    assert_eq!(
        start0.as_bytes(),
        Ok(vec![
            0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x57, 0x6f, 0x72, 0x6c, 0x64
        ])
    ); // "Hello World"
}

#[test]
fn test_initial_values() {
    let xml_exact =
        r#"<Float64 name="exact_var" valueReference="700" initial="exact" start="1.0"/>"#;
    let var_exact: FmiFloat64 = yaserde::de::from_str(xml_exact).unwrap();
    assert_eq!(var_exact.initial(), Some(Initial::Exact));

    let xml_approx =
        r#"<Float64 name="approx_var" valueReference="701" initial="approx" start="1.0"/>"#;
    let var_approx: FmiFloat64 = yaserde::de::from_str(xml_approx).unwrap();
    assert_eq!(var_approx.initial(), Some(Initial::Approx));

    let xml_calculated =
        r#"<Float64 name="calc_var" valueReference="702" initial="calculated" start="1.0"/>"#;
    let var_calculated: FmiFloat64 = yaserde::de::from_str(xml_calculated).unwrap();
    assert_eq!(var_calculated.initial(), Some(Initial::Calculated));
}

#[test]
fn test_variable_annotations() {
    let xml = r#"<Int32 name="annotated_var" valueReference="800" causality="local" start="42">
            <Annotations>
                <Annotation type="info">This is an informational annotation.</Annotation>
                <Annotation type="warning">This is a warning annotation.</Annotation>
            </Annotations>
        </Int32>"#;

    let var: FmiInt32 = yaserde::de::from_str(xml).unwrap();
    assert_eq!(var.name(), "annotated_var");
    assert_eq!(var.value_reference(), 800);
    assert_eq!(var.causality(), Causality::Local);
    assert_eq!(var.start, Some(vec![42]));

    let annotations = var.annotations().unwrap();
    assert_eq!(annotations.annotations.len(), 2);
    assert_eq!(annotations.annotations[0].r#type, "info".to_string());
    assert_eq!(
        annotations.annotations[0].content,
        "This is an informational annotation."
    );
    assert_eq!(annotations.annotations[1].r#type, "warning".to_string());
    assert_eq!(
        annotations.annotations[1].content,
        "This is a warning annotation."
    );
}

#[test]
fn test_data_type_enum() {
    let float32_var: FmiFloat32 = Default::default();
    assert_eq!(float32_var.data_type(), VariableType::FmiFloat32);

    let float64_var: FmiFloat64 = Default::default();
    assert_eq!(float64_var.data_type(), VariableType::FmiFloat64);

    let int8_var: FmiInt8 = Default::default();
    assert_eq!(int8_var.data_type(), VariableType::FmiInt8);

    let uint8_var: FmiUInt8 = Default::default();
    assert_eq!(uint8_var.data_type(), VariableType::FmiUInt8);

    let int16_var: FmiInt16 = Default::default();
    assert_eq!(int16_var.data_type(), VariableType::FmiInt16);

    let uint16_var: FmiUInt16 = Default::default();
    assert_eq!(uint16_var.data_type(), VariableType::FmiUInt16);

    let int32_var: FmiInt32 = Default::default();
    assert_eq!(int32_var.data_type(), VariableType::FmiInt32);

    let uint32_var: FmiUInt32 = Default::default();
    assert_eq!(uint32_var.data_type(), VariableType::FmiUInt32);

    let int64_var: FmiInt64 = Default::default();
    assert_eq!(int64_var.data_type(), VariableType::FmiInt64);

    let uint64_var: FmiUInt64 = Default::default();
    assert_eq!(uint64_var.data_type(), VariableType::FmiUInt64);

    let boolean_var: FmiBoolean = Default::default();
    assert_eq!(boolean_var.data_type(), VariableType::FmiBoolean);

    let string_var: FmiString = Default::default();
    assert_eq!(string_var.data_type(), VariableType::FmiString);

    let binary_var: FmiBinary = Default::default();
    assert_eq!(binary_var.data_type(), VariableType::FmiBinary);
}

#[cfg(feature = "arrow")]
#[test]
fn test_arrow_data_type_conversion() {
    use arrow::datatypes::DataType;

    assert_eq!(DataType::from(VariableType::FmiFloat32), DataType::Float32);
    assert_eq!(DataType::from(VariableType::FmiFloat64), DataType::Float64);
    assert_eq!(DataType::from(VariableType::FmiInt8), DataType::Int8);
    assert_eq!(DataType::from(VariableType::FmiUInt8), DataType::UInt8);
    assert_eq!(DataType::from(VariableType::FmiInt16), DataType::Int16);
    assert_eq!(DataType::from(VariableType::FmiUInt16), DataType::UInt16);
    assert_eq!(DataType::from(VariableType::FmiInt32), DataType::Int32);
    assert_eq!(DataType::from(VariableType::FmiUInt32), DataType::UInt32);
    assert_eq!(DataType::from(VariableType::FmiInt64), DataType::Int64);
    assert_eq!(DataType::from(VariableType::FmiUInt64), DataType::UInt64);
    assert_eq!(DataType::from(VariableType::FmiBoolean), DataType::Boolean);
    assert_eq!(DataType::from(VariableType::FmiString), DataType::Utf8);
    assert_eq!(DataType::from(VariableType::FmiBinary), DataType::Binary);
}
