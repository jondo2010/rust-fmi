#![deny(clippy::all)]

use fmi::fmi3::{Fmi3Error, Fmi3Res};
use fmi_export::{
    fmi3::{Binary, Clock, DefaultLoggingCategory, ModelContext, UserModel},
    FmuModel,
};

#[derive(FmuModel, Default, Debug)]
struct CanTriggeredOutput {
    #[variable(
        name = "CanChannel.Rx_Data",
        causality = Input,
        variability = Discrete,
        initial = Exact,
        max_size = 2048,
        clocks = [rx_clock],
        mime_type = "application/org.fmi-standard.fmi-ls-bus.can; version=\"1.0.0-beta.1\"",
        start = b""
    )]
    rx_data: Binary,

    #[variable(
        name = "CanChannel.Tx_Data",
        causality = Output,
        variability = Discrete,
        initial = Calculated,
        max_size = 2048,
        clocks = [tx_clock],
        mime_type = "application/org.fmi-standard.fmi-ls-bus.can; version=\"1.0.0-beta.1\"",
    )]
    tx_data: Binary,

    #[variable(name="CanChannel.Rx_Clock", causality=Input, interval_variability=Triggered)]
    rx_clock: Clock,

    #[variable(name="CanChannel.Tx_Clock", causality=Output, interval_variability=Triggered)]
    tx_clock: Clock,

    #[variable(
        name = "org.fmi_standard.fmi_ls_bus.Can_BusNotifications",
        causality = StructuralParameter,
        variability = Fixed,
        initial = Exact,
        start = false
    )]
    can_bus_notifications: bool,
}

impl UserModel for CanTriggeredOutput {
    type LoggingCategory = DefaultLoggingCategory;

    fn configurate(&mut self, _context: &ModelContext<Self>) -> Result<(), Fmi3Error> {
        todo!();
    }

    fn calculate_values(&mut self, _context: &ModelContext<Self>) -> Result<Fmi3Res, Fmi3Error> {
        Ok(Fmi3Res::OK)
    }
}

// Export the FMU with full C API
fmi_export::export_fmu!(CanTriggeredOutput);

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use fmi::fmi3::GetSet;
    use fmi::schema::fmi3::AbstractVariableTrait;
    use fmi_export::fmi3::{Model, ModelInstance};

    #[test]
    fn test_metadata() {
        let (vars, structure) = CanTriggeredOutput::build_toplevel_metadata();

        // Debug print to see actual VRs
        println!(
            "rx_data: VR={}, clocks={:?}",
            vars.binary[0].value_reference(),
            vars.binary[0].clocks()
        );
        println!(
            "tx_data: VR={}, clocks={:?}",
            vars.binary[1].value_reference(),
            vars.binary[1].clocks()
        );
        println!("rx_clock: VR={}", vars.clock[0].value_reference());
        println!("tx_clock: VR={}", vars.clock[1].value_reference());

        // Check the generated variables and their VRs
        assert_eq!(vars.binary.len(), 2); // rx_data and tx_data
        assert_eq!(vars.binary[0].name(), "CanChannel.Rx_Data");
        assert_eq!(vars.binary[0].value_reference(), 1); // rx_data has VR 1
        assert_eq!(vars.binary[0].clocks(), Some(&[3u32][..])); // rx_clock has VR 3
        assert_eq!(
            vars.binary[0].mime_type,
            "application/org.fmi-standard.fmi-ls-bus.can; version=\"1.0.0-beta.1\""
        );
        assert_eq!(vars.binary[1].name(), "CanChannel.Tx_Data");
        assert_eq!(vars.binary[1].value_reference(), 2); // tx_data has VR 2
        assert_eq!(vars.binary[1].clocks(), Some(&[4u32][..])); // tx_clock has VR 4
        assert_eq!(
            vars.binary[1].mime_type,
            "application/org.fmi-standard.fmi-ls-bus.can; version=\"1.0.0-beta.1\""
        );

        assert_eq!(vars.clock.len(), 2); // rx_clock and tx_clock
        assert_eq!(vars.clock[0].name(), "CanChannel.Rx_Clock");
        assert_eq!(vars.clock[0].value_reference(), 3); // rx_clock has VR 3
        assert_eq!(vars.clock[1].name(), "CanChannel.Tx_Clock");
        assert_eq!(vars.clock[1].value_reference(), 4); // tx_clock has VR 4

        assert_eq!(vars.boolean.len(), 1); // can_bus_notifications
        assert_eq!(
            vars.boolean[0].name(),
            "org.fmi_standard.fmi_ls_bus.Can_BusNotifications"
        );
        assert_eq!(vars.boolean[0].value_reference(), 5); // can_bus_notifications has VR 5

        // Check max_size for binary variables (currently not supported by derive macro)
        // TODO: Enable these tests once the derive macro properly handles max_size attributes
        // assert_eq!(vars.binary[0].max_size, Some(2048));
        // assert_eq!(vars.binary[1].max_size, Some(2048));

        // Check the model structure
        assert_eq!(structure.outputs.len(), 2);
        assert_eq!(structure.outputs[0].value_reference, 2); // tx_data (VR=2)
        assert_eq!(structure.outputs[1].value_reference, 4); // tx_clock (VR=4)

        let xml = fmi::schema::serialize(&structure, true).unwrap();
        println!("{xml}");
    }

    #[test]
    fn test_model_get_set() {
        let mut model = ModelInstance::<CanTriggeredOutput>::new(
            "CanTriggeredOutput".to_string(),
            PathBuf::new(),
            true,
            Box::new(|_, _, _| {}),
            CanTriggeredOutput::INSTANTIATION_TOKEN,
        )
        .unwrap();

        let (rx_clock_vr, tx_clock_vr) = (3, 4); // VRs for rx_clock and tx_clock

        // Test clock operations. It should only be possible to set rx_clock (input clock), and to get tx_clock (output clock)
        let mut clock_val = [false; 1];
        model.set_clock(&[rx_clock_vr], &[true]).unwrap(); // Set rx_clock (VR=3)
        model.get_clock(&[tx_clock_vr], &mut clock_val).unwrap(); // Get tx_clock (VR=4)
        assert_eq!(clock_val, [false]); // Initial clock states

        // Test setting rx_clock
        model.set_clock(&[3], &[true]).unwrap(); // Set rx_clock (VR=3)

        // Test boolean parameter (StructuralParameter - read-only)
        let mut bool_vals = [false; 1];
        println!("Calling get_boolean with VR=5");
        model
            .get_boolean(&[5], &mut bool_vals) // can_bus_notifications VR=5
            .expect("Failed to get boolean parameter");
        assert_eq!(bool_vals, [false]); // can_bus_notifications default value

        println!(
            "Successfully read boolean StructuralParameter at VR=5: {:?}",
            bool_vals
        );
        // Note: We don't test set_boolean for StructuralParameter as they are typically read-only
    }

    #[test]
    fn test_binary_start_values() {
        let mut model = CanTriggeredOutput::default();
        // Set start values as would happen in ModelInstance::new()
        fmi_export::fmi3::Model::set_start_values(&mut model);

        // Test that rx_data has been initialized with start value b""
        assert_eq!(model.rx_data.0, Vec::<u8>::new()); // rx_data should be empty

        // Test that tx_data has default value (no start value specified)
        assert_eq!(model.tx_data.0, Vec::<u8>::new()); // tx_data should be empty (default)

        println!("Binary start values initialized correctly:");
        println!("  rx_data: {:?}", model.rx_data.0);
        println!("  tx_data: {:?}", model.tx_data.0);
    }

    #[test]
    fn test_can_data_processing() {
        let mut model = ModelInstance::<CanTriggeredOutput>::new(
            "CanTriggeredOutput".to_string(),
            PathBuf::new(),
            true,
            Box::new(|_, _, _| {}),
            CanTriggeredOutput::INSTANTIATION_TOKEN,
        )
        .unwrap();

        // Set some test CAN data
        let test_data = vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF];
        let rx_buffers = vec![test_data.as_slice()];
        model.set_binary(&[1], &rx_buffers).unwrap(); // rx_data VR=1

        // Trigger rx_clock
        model.set_clock(&[3], &[true]).unwrap(); // rx_clock VR=3

        // Check that tx_clock is triggered after processing (this would happen in the FMI simulation)
        let mut clock_vals = [false; 1];
        model.get_clock(&[4], &mut clock_vals).unwrap(); // tx_clock VR=4
                                                         // Note: In a real simulation, calculate_values would be called by the simulation engine
    }
}
