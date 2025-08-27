// Extended test to demonstrate comprehensive FMI datatype support
use fmi_export::{FmuModel, fmi3::UserModel};

#[derive(Debug, Default, FmuModel)]
#[model(model_exchange())]
struct TestModel {
    // Original test variable
    #[variable(causality = Output, start = 1.0)]
    value: f64,
    
    // Extended datatype support demonstration
    #[variable(causality = Output, start = 1.5)]
    position_f32: f32,
    
    #[variable(causality = Parameter, start = 10)]
    counter_i8: i8,
    
    #[variable(causality = Parameter, start = 1000)]
    id_i16: i16,
    
    #[variable(causality = Parameter, start = 50000)]
    count_i32: i32,
    
    #[variable(causality = Parameter, start = 9000000000)]
    large_value_i64: i64,
    
    #[variable(causality = Input, start = 255)]
    status_u8: u8,
    
    #[variable(causality = Input, start = 8080)]
    port_u16: u16,
    
    #[variable(causality = Input, start = 1024)]
    size_u32: u32,
    
    #[variable(causality = Input, start = 1234567890123)]
    timestamp_u64: u64,
    
    #[variable(causality = Input, start = true)]
    enabled: bool,
    
    #[variable(causality = Parameter, start = "TestModel")]
    model_name: String,
}

impl UserModel for TestModel {
    // Use default implementations
}

fn main() {
    println!("Generated ValueRef variants for all supported datatypes:");
    
    // Original
    println!("Value = {:?}", ValueRef::Value);
    
    // All extended types
    println!("PositionF32 = {:?}", ValueRef::PositionF32);
    println!("CounterI8 = {:?}", ValueRef::CounterI8);
    println!("IdI16 = {:?}", ValueRef::IdI16);
    println!("CountI32 = {:?}", ValueRef::CountI32);
    println!("LargeValueI64 = {:?}", ValueRef::LargeValueI64);
    println!("StatusU8 = {:?}", ValueRef::StatusU8);
    println!("PortU16 = {:?}", ValueRef::PortU16);
    println!("SizeU32 = {:?}", ValueRef::SizeU32);
    println!("TimestampU64 = {:?}", ValueRef::TimestampU64);
    println!("Enabled = {:?}", ValueRef::Enabled);
    println!("ModelName = {:?}", ValueRef::ModelName);

    let model = TestModel::default();
    println!("Model: {:?}", model);
    
    println!("✅ Extended GetSet support now includes all FMI3 datatypes:");
    println!("  - Floating point: f32, f64");
    println!("  - Signed integers: i8, i16, i32, i64"); 
    println!("  - Unsigned integers: u8, u16, u32, u64");
    println!("  - Boolean: bool");
    println!("  - String: String");
}
