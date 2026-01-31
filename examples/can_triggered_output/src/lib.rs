#![deny(clippy::all)]

use std::borrow::Cow;

use fmi::fmi3::{Fmi3Error, Fmi3Res};
use fmi_export::{
    fmi3::{CSDoStepResult, Context, DefaultLoggingCategory, UserModel},
    FmuModel,
};
use fmi_ls_bus::can::{CanBus, LsBusCanArbitrationLostBehavior, LsBusCanOp};

const TRANSMIT_INTERVAL: f64 = 0.3;
const CAN_ID: u32 = 0x1;

#[derive(FmuModel, Debug)]
#[model(co_simulation = true, model_exchange = false, user_model = false)]
struct CanTriggeredOutput {
    #[child(prefix = "CanChannel")]
    can: CanBus,

    #[variable(skip)]
    simulation_time: f64,

    #[variable(skip)]
    next_transmit_time: f64,

    #[variable(
        name = "org.fmi_standard.fmi_ls_bus.Can_BusNotifications",
        causality = StructuralParameter,
        variability = Fixed,
        initial = Exact,
        start = false
    )]
    can_bus_notifications: bool,
}

impl Default for CanTriggeredOutput {
    fn default() -> Self {
        Self {
            can: CanBus::default(),
            simulation_time: 0.0,
            next_transmit_time: TRANSMIT_INTERVAL,
            can_bus_notifications: false,
        }
    }
}

impl UserModel for CanTriggeredOutput {
    type LoggingCategory = DefaultLoggingCategory;

    fn configurate(&mut self, _context: &dyn Context<Self>) -> Result<(), Fmi3Error> {
        self.can.reset_buffers();

        // Create bus configuration operations
        self.can
            .tx_send_batch(|bus| {
                bus.write_operation(LsBusCanOp::ConfigBaudrate(100_000))?;
                bus.write_operation(LsBusCanOp::ConfigArbitrationLost(
                    LsBusCanArbitrationLostBehavior::BufferAndRetransmit,
                ))?;
                Ok(())
            })
            .map_err(|_| Fmi3Error::Error)?;
        self.simulation_time = 0.0;
        self.next_transmit_time = TRANSMIT_INTERVAL;
        Ok(())
    }

    fn calculate_values(&mut self, _context: &dyn Context<Self>) -> Result<Fmi3Res, Fmi3Error> {
        Ok(Fmi3Res::OK)
    }

    fn event_update(
        &mut self,
        context: &dyn Context<Self>,
        event_flags: &mut fmi::EventFlags,
    ) -> Result<Fmi3Res, Fmi3Error> {
        // We only process bus operations when the RX clock is set
        self.can
            .process_rx(|op| match op {
                LsBusCanOp::Transmit { id, data, .. } => {
                    context.log(
                        Fmi3Res::OK.into(),
                        Self::LoggingCategory::default(),
                        format_args!(
                            "Received CAN frame with ID {id} and length {}",
                            data.len()
                        ),
                    );
                }
                LsBusCanOp::ConfigBaudrate(_)
                | LsBusCanOp::Status(_)
                | LsBusCanOp::Confirm(_)
                | LsBusCanOp::BusError { .. }
                | LsBusCanOp::ArbitrationLost { .. } => {
                    // ignore for now
                }
                _ => {
                    context.log(
                        Fmi3Res::Warning.into(),
                        Self::LoggingCategory::default(),
                        format_args!("Received unexpected CAN operation: {:?}", op),
                    );
                }
            })
            .map_err(|_| Fmi3Error::Error)?;

        // Deactivate clocks and clear buffers since all operations should have been processed
        self.can.clear_after_event();

        event_flags.reset();

        Ok(Fmi3Res::OK)
    }

    fn do_step(
        &mut self,
        context: &mut dyn Context<Self>,
        current_communication_point: f64,
        communication_step_size: f64,
        _no_set_fmu_state_prior_to_current_point: bool,
    ) -> Result<CSDoStepResult, Fmi3Error> {
        context.set_time(current_communication_point);
        self.simulation_time = current_communication_point;

        let target_time = current_communication_point + communication_step_size;
        let event_needed = self
            .can
            .tx_send_batch(|bus| {
                while self.next_transmit_time <= target_time {
                    self.simulation_time = self.next_transmit_time;

                    bus.write_operation(LsBusCanOp::Transmit {
                        id: CAN_ID,
                        ide: 0,
                        rtr: 0,
                        data: Cow::Owned(vec![1, 2, 3, 4]),
                    })?;

                    context.log(
                        Fmi3Res::OK.into(),
                        Self::LoggingCategory::default(),
                        format_args!(
                            "Transmitting CAN frame with ID {CAN_ID} at internal time {:.3}",
                            self.next_transmit_time
                        ),
                    );

                    self.next_transmit_time += TRANSMIT_INTERVAL;
                }

                Ok(())
            })
            .map_err(|_| Fmi3Error::Error)?;

        self.simulation_time = target_time;
        context.set_time(target_time);

        Ok(CSDoStepResult {
            event_handling_needed: event_needed,
            terminate_simulation: false,
            early_return: false,
            last_successful_time: target_time,
        })
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
    use fmi_export::fmi3::{BasicContext, Model, ModelInstance};

    #[test]
    fn test_metadata() {
        let (vars, structure) = CanTriggeredOutput::build_toplevel_metadata();

        // Debug print to see actual VRs
        let binaries = vars.binary();
        let rx_clock = vars
            .find_by_name("CanChannel.Rx_Clock")
            .expect("rx clock variable");
        let tx_clock = vars
            .find_by_name("CanChannel.Tx_Clock")
            .expect("tx clock variable");

        println!(
            "rx_data: VR={}, clocks={:?}",
            binaries[0].value_reference(),
            binaries[0].clocks()
        );
        println!(
            "tx_data: VR={}, clocks={:?}",
            binaries[1].value_reference(),
            binaries[1].clocks()
        );
        println!("rx_clock: VR={}", rx_clock.value_reference());
        println!("tx_clock: VR={}", tx_clock.value_reference());

        // Check the generated variables and their VRs
        assert_eq!(binaries.len(), 2); // rx_data and tx_data
        assert_eq!(binaries[0].name(), "CanChannel.Rx_Data");
        assert_eq!(binaries[0].value_reference(), 1); // rx_data has VR 1
        assert_eq!(binaries[0].clocks(), Some(&[3u32][..])); // rx_clock has VR 3
        assert_eq!(
            binaries[0].mime_type,
            Some("application/org.fmi-standard.fmi-ls-bus.can; version=\"1.0.0-beta.1\"".to_string())
        );
        assert_eq!(binaries[1].name(), "CanChannel.Tx_Data");
        assert_eq!(binaries[1].value_reference(), 2); // tx_data has VR 2
        assert_eq!(binaries[1].clocks(), Some(&[4u32][..])); // tx_clock has VR 4
        assert_eq!(
            binaries[1].mime_type,
            Some("application/org.fmi-standard.fmi-ls-bus.can; version=\"1.0.0-beta.1\"".to_string())
        );

        assert_eq!(rx_clock.name(), "CanChannel.Rx_Clock");
        assert_eq!(rx_clock.value_reference(), 3); // rx_clock has VR 3
        assert_eq!(tx_clock.name(), "CanChannel.Tx_Clock");
        assert_eq!(tx_clock.value_reference(), 4); // tx_clock has VR 4

        let booleans = vars.boolean();
        assert_eq!(booleans.len(), 1); // can_bus_notifications
        assert_eq!(
            booleans[0].name(),
            "org.fmi_standard.fmi_ls_bus.Can_BusNotifications"
        );
        assert_eq!(booleans[0].value_reference(), 5); // can_bus_notifications has VR 5

        // Check max_size for binary variables (currently not supported by derive macro)
        // TODO: Enable these tests once the derive macro properly handles max_size attributes
        // assert_eq!(vars.binary[0].max_size, Some(2048));
        // assert_eq!(vars.binary[1].max_size, Some(2048));

        // Check the model structure
        let outputs: Vec<_> = structure.outputs().collect();
        assert_eq!(outputs.len(), 2);
        assert_eq!(outputs[0].value_reference, 2); // tx_data (VR=2)
        assert_eq!(outputs[1].value_reference, 4); // tx_clock (VR=4)

        let xml = fmi::schema::serialize(&structure, true).unwrap();
        println!("{xml}");
    }

    #[test]
    fn test_model_get_set() {
        let (vars, _) = CanTriggeredOutput::build_toplevel_metadata();
        let rx_clock_vr = vars
            .find_by_name("CanChannel.Rx_Clock")
            .expect("rx clock variable")
            .value_reference();
        let tx_clock_vr = vars
            .find_by_name("CanChannel.Tx_Clock")
            .expect("tx clock variable")
            .value_reference();

        println!(
            "CanBus FIELD_COUNT={}, CanTriggeredOutput FIELD_COUNT={}",
            <CanBus as fmi_export::fmi3::ModelGetSet<CanTriggeredOutput>>::FIELD_COUNT,
            <CanTriggeredOutput as fmi_export::fmi3::ModelGetSet<CanTriggeredOutput>>::FIELD_COUNT
        );

        let context = BasicContext::new(true, Box::new(|_, _, _| {}), PathBuf::new(), false, None);
        let mut model = ModelInstance::<CanTriggeredOutput, _>::new(
            "CanTriggeredOutput".to_string(),
            CanTriggeredOutput::INSTANTIATION_TOKEN,
            context,
            fmi::InterfaceType::CoSimulation,
        )
        .unwrap();

        // Test clock operations. It should only be possible to set rx_clock (input clock), and to get tx_clock (output clock)
        let mut clock_val = [false; 1];
        model.set_clock(&[rx_clock_vr], &[true]).unwrap(); // Set rx_clock (VR=3)
        model.get_clock(&[tx_clock_vr], &mut clock_val).unwrap(); // Get tx_clock (VR=4)
        assert_eq!(clock_val, [false]); // Initial clock states

        // Test setting rx_clock
        model.set_clock(&[rx_clock_vr], &[true]).unwrap(); // Set rx_clock

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
        assert_eq!(model.can.rx_data.0, Vec::<u8>::new()); // rx_data should be empty

        // Test that tx_data has default value (no start value specified)
        assert_eq!(model.can.tx_data.0, Vec::<u8>::new()); // tx_data should be empty (default)

        println!("Binary start values initialized correctly:");
        println!("  rx_data: {:?}", model.can.rx_data.0);
        println!("  tx_data: {:?}", model.can.tx_data.0);
    }

    #[test]
    fn test_can_data_processing() {
        let (vars, _) = CanTriggeredOutput::build_toplevel_metadata();
        let rx_data_vr = vars
            .find_by_name("CanChannel.Rx_Data")
            .expect("rx data variable")
            .value_reference();
        let rx_clock_vr = vars
            .find_by_name("CanChannel.Rx_Clock")
            .expect("rx clock variable")
            .value_reference();
        let tx_clock_vr = vars
            .find_by_name("CanChannel.Tx_Clock")
            .expect("tx clock variable")
            .value_reference();

        let context = BasicContext::new(true, Box::new(|_, _, _| {}), PathBuf::new(), false, None);
        let mut model = ModelInstance::<CanTriggeredOutput, _>::new(
            "CanTriggeredOutput".to_string(),
            CanTriggeredOutput::INSTANTIATION_TOKEN,
            context,
            fmi::InterfaceType::CoSimulation,
        )
        .unwrap();

        // Set some test CAN data
        let test_data = vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF];
        let rx_buffers = vec![test_data.as_slice()];
        model.set_binary(&[rx_data_vr], &rx_buffers).unwrap(); // rx_data

        // Trigger rx_clock
        model.set_clock(&[rx_clock_vr], &[true]).unwrap(); // rx_clock

        // Check that tx_clock is triggered after processing (this would happen in the FMI simulation)
        let mut clock_vals = [false; 1];
        model.get_clock(&[tx_clock_vr], &mut clock_vals).unwrap(); // tx_clock
                                                         // Note: In a real simulation, calculate_values would be called by the simulation engine
    }
}
