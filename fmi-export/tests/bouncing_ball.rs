use fmi::fmi3::{Common, Fmi3Res, ModelExchange};
use fmi_export::{
    FmuModel,
    fmi3::{Model, UserModel},
};

/// Bouncing Ball model
///
/// This model simulates a ball bouncing on the ground with realistic physics including:
/// - Gravitational acceleration
/// - Energy loss on impact (coefficient of restitution)
/// - Event detection for ground contact
/// - Stopping condition when velocity becomes too small
///
/// Mathematical equations:
/// - dh/dt = v (velocity is derivative of height)
/// - dv/dt = g (gravity is derivative of velocity)
///
/// Event handling:
/// - When ball hits ground (h <= 0 and v < 0):
///   - Reverse velocity and apply energy loss: v := -v * e
///   - If velocity becomes too small (v < v_min), stop bouncing
///
/// where:
/// - h: height above ground (state)
/// - v: vertical velocity (state)
/// - g: gravitational acceleration (parameter, typically -9.81 m/s²)
/// - e: coefficient of restitution (parameter, 0 < e < 1)
/// - v_min: minimum velocity threshold (constant)
#[derive(FmuModel, Default, Debug)]
#[model(model_exchange())]
struct BouncingBall {
    /// Height above ground (state output)
    #[variable(causality = Output, state, start = 1.0)]
    h: f64,

    /// Velocity of the ball
    #[variable(causality = Output, state, start = 0.0)]
    #[alias(name="der(h)", causality = Local, derivative=h)]
    v: f64,

    /// Gravitational acceleration
    #[variable(causality = Parameter, variability = Fixed, start = -9.81)]
    #[alias(name = "der(v)", causality = Local, derivative = v)]
    g: f64,

    /// Coefficient of restitution (parameter)
    #[variable(causality = Parameter, variability = Tunable, start = 0.7)]
    e: f64,

    /// Minimum velocity threshold (constant)
    #[variable(causality = Local, start = 0.1)]
    v_min: f64,
}

impl UserModel for BouncingBall {
    /// Calculate derivatives for the bouncing ball model
    /// With aliases: der(h) = v and der(v) = g
    /// No explicit synchronization needed since aliases are handled by the generated code
    fn calculate_values(&mut self) -> fmi::fmi3::Fmi3Status {
        // No calculation needed - der(h) is aliased to v, der(v) is aliased to g
        // The generated getter code will automatically return the correct values
        Fmi3Res::OK.into()
    }

    /// Handle discrete events (ball hitting ground)
    fn event_update(&mut self) -> Result<fmi::fmi3::Fmi3Res, fmi::fmi3::Fmi3Error> {
        // Check if ball hits ground (h <= 0) and is moving downward (v < 0)
        if self.h <= 0.0 && self.v < 0.0 {
            // Set height slightly above ground to avoid numerical issues
            self.h = f64::MIN_POSITIVE;

            // Reverse velocity and apply energy loss due to impact
            self.v = -self.v * self.e;

            // If velocity becomes too small, stop bouncing
            if self.v < self.v_min {
                self.v = 0.0;
                self.g = 0.0; // No more gravity when ball stops
            }
        }

        Ok(Fmi3Res::OK)
    }

    /// Get event indicators for zero-crossing detection
    fn get_event_indicators(
        &mut self,
        indicators: &mut [f64],
    ) -> Result<fmi::fmi3::Fmi3Res, fmi::fmi3::Fmi3Error> {
        if !indicators.is_empty() {
            // Event indicator: positive when ball is above ground, zero when touching ground
            // Special case: if both height and velocity are zero (stopped), return positive value
            indicators[0] = if self.h == 0.0 && self.v == 0.0 {
                1.0
            } else {
                self.h
            };
        }
        Ok(Fmi3Res::OK)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fmi_export::fmi3::ModelInstance;

    #[test]
    fn test_model_metadata() {
        // Test that the Model derive macro generates the expected metadata
        assert_eq!(BouncingBall::MODEL_NAME, "BouncingBall");
        assert!(!BouncingBall::MODEL_DESCRIPTION.is_empty());
        assert!(!BouncingBall::INSTANTIATION_TOKEN.is_empty());

        // Test continuous states count (h and v are states)
        let actual_state_count = BouncingBall::get_number_of_continuous_states();
        println!("Actual continuous states count: {}", actual_state_count);
        assert_eq!(
            actual_state_count, 2,
            "Expected 2 continuous states (h and v), got {}",
            actual_state_count
        );

        // Test model description content
        assert!(BouncingBall::MODEL_DESCRIPTION.contains("Bouncing Ball"));
    }

    #[test]
    fn test_derivative_attributes() {
        // Test that derivative aliases have correct derivative attributes set
        let model_vars = BouncingBall::model_variables();

        // Should have 7 Float64 variables: h, v, der(h), g, der(v), e, v_min
        assert_eq!(model_vars.float64.len(), 7, "Expected 7 Float64 variables");

        // Find der(h) and der(v) variables
        let der_h = model_vars
            .float64
            .iter()
            .find(|v| {
                v.init_var
                    .typed_arrayable_var
                    .arrayable_var
                    .abstract_var
                    .name
                    == "der(h)"
            })
            .expect("Should find der(h) variable");
        let der_v = model_vars
            .float64
            .iter()
            .find(|v| {
                v.init_var
                    .typed_arrayable_var
                    .arrayable_var
                    .abstract_var
                    .name
                    == "der(v)"
            })
            .expect("Should find der(v) variable");

        // Validate der(h) attributes
        assert_eq!(
            der_h
                .init_var
                .typed_arrayable_var
                .arrayable_var
                .abstract_var
                .value_reference,
            2
        );
        assert_eq!(
            der_h
                .init_var
                .typed_arrayable_var
                .arrayable_var
                .abstract_var
                .causality,
            fmi::fmi3::schema::Causality::Local
        );
        assert_eq!(
            der_h.init_var.initial,
            Some(fmi::fmi3::schema::Initial::Calculated)
        );
        assert_eq!(der_h.real_var_attr.derivative, Some(0)); // Points to h (VR=0)
        assert!(der_h.start.is_empty()); // No start value for derivatives

        // Validate der(v) attributes
        assert_eq!(
            der_v
                .init_var
                .typed_arrayable_var
                .arrayable_var
                .abstract_var
                .value_reference,
            4
        );
        assert_eq!(
            der_v
                .init_var
                .typed_arrayable_var
                .arrayable_var
                .abstract_var
                .causality,
            fmi::fmi3::schema::Causality::Local
        );
        assert_eq!(
            der_v.init_var.initial,
            Some(fmi::fmi3::schema::Initial::Calculated)
        );
        assert_eq!(der_v.real_var_attr.derivative, Some(1)); // Points to v (VR=1)
        assert!(der_v.start.is_empty()); // No start value for derivatives

        // Validate base variables have no derivative attribute
        let h = model_vars
            .float64
            .iter()
            .find(|v| {
                v.init_var
                    .typed_arrayable_var
                    .arrayable_var
                    .abstract_var
                    .name
                    == "h"
            })
            .expect("Should find h variable");
        let v = model_vars
            .float64
            .iter()
            .find(|v| {
                v.init_var
                    .typed_arrayable_var
                    .arrayable_var
                    .abstract_var
                    .name
                    == "v"
            })
            .expect("Should find v variable");

        assert_eq!(h.real_var_attr.derivative, None);
        assert_eq!(v.real_var_attr.derivative, None);

        println!("All derivative attributes validated successfully!");
    }

    #[test]
    fn test_bounce_event() {
        let mut model = BouncingBall::default();

        // Set conditions: ball hits ground with downward velocity
        model.h = 0.0;
        model.v = -2.0;
        model.g = -9.81;
        model.e = 0.7;
        model.v_min = 0.1;

        // Trigger bounce event
        model.event_update().expect("Event update should succeed");

        // Check that ball bounced
        assert!(
            model.h > 0.0,
            "Height should be slightly above zero after bounce"
        );
        assert_eq!(model.v, 1.4); // v = -(-2.0) * 0.7 = 1.4
        assert_eq!(model.g, -9.81); // Gravity unchanged since velocity > v_min
    }

    #[test]
    fn test_stop_bouncing() {
        let mut model = BouncingBall::default();

        // Set conditions: ball hits ground with very small velocity
        model.h = 0.0;
        model.v = -0.05; // Less than v_min (0.1)
        model.g = -9.81;
        model.e = 0.7;
        model.v_min = 0.1;

        // Trigger bounce event
        model.event_update().expect("Event update should succeed");

        // Check that ball stopped bouncing
        assert!(model.h > 0.0);
        assert_eq!(model.v, 0.0); // Velocity set to zero
        assert_eq!(model.g, 0.0); // Gravity disabled
    }

    #[test]
    fn test_event_indicators() {
        let mut model = BouncingBall::default();

        // Test normal case: ball above ground
        let mut indicators = [0.0];
        model
            .get_event_indicators(&mut indicators)
            .expect("Should get event indicators");
        assert_eq!(indicators[0], 1.0); // h = 1.0 (default start value)

        // Test stopped ball case
        let mut stopped_model = BouncingBall {
            h: 0.0,
            v: 0.0,
            ..Default::default()
        };
        stopped_model
            .get_event_indicators(&mut indicators)
            .expect("Should get event indicators");
        assert_eq!(indicators[0], 1.0); // Special case: stopped ball
    }

    #[test]
    fn test_model_exchange_interface() {
        let instantiation_token = BouncingBall::INSTANTIATION_TOKEN;

        let mut instance = ModelInstance::<BouncingBall>::new(
            "TestBouncingBall".to_string(),
            std::path::PathBuf::from("/tmp"),
            false,
            None,
            instantiation_token,
        )
        .expect("Failed to create model instance");

        // Enter initialization mode
        instance
            .enter_initialization_mode(None, 0.0, None)
            .expect("Failed to enter initialization mode");

        // Set and get continuous states
        let initial_states = [1.0, 0.0]; // h=1.0m, v=0.0m/s
        instance
            .set_continuous_states(&initial_states)
            .expect("Failed to set states");

        let mut retrieved_states = [0.0, 0.0];
        instance
            .get_continuous_states(&mut retrieved_states)
            .expect("Failed to get states");
        assert_eq!(retrieved_states, [1.0, 0.0]);

        // Exit initialization and enter continuous time mode
        instance
            .exit_initialization_mode()
            .expect("Failed to exit initialization mode");
        instance
            .enter_event_mode()
            .expect("Failed to enter event mode");
        instance
            .enter_continuous_time_mode()
            .expect("Failed to enter continuous time mode");

        // Get derivatives
        let mut derivatives = [0.0, 0.0];
        instance
            .get_continuous_state_derivatives(&mut derivatives)
            .expect("Failed to get derivatives");

        // Verify calculated derivatives
        assert_eq!(derivatives[0], 0.0); // der_h = v = 0.0
        assert_eq!(derivatives[1], -9.81); // der_v = g = -9.81

        // Test event indicators
        let mut indicators = [0.0];
        instance
            .get_event_indicators(&mut indicators)
            .expect("Failed to get event indicators");
        assert_eq!(indicators[0], 1.0); // h = 1.0
    }

    #[test]
    fn test_value_references() {
        let variables = BouncingBall::model_variables();
        println!("Float64 variables:");
        for (i, var) in variables.float64.iter().enumerate() {
            println!(
                "  {} - VR: {}, Name: {}, Causality: {:?}",
                i,
                var.init_var
                    .typed_arrayable_var
                    .arrayable_var
                    .abstract_var
                    .value_reference,
                var.init_var
                    .typed_arrayable_var
                    .arrayable_var
                    .abstract_var
                    .name,
                var.init_var
                    .typed_arrayable_var
                    .arrayable_var
                    .abstract_var
                    .causality
            );
        }
    }

    #[test]
    fn test_comprehensive_variable_get_set_behavior() {
        use fmi::fmi3::GetSet;

        let instantiation_token = BouncingBall::INSTANTIATION_TOKEN;

        // Define all variables with their expected properties (VR is looked up dynamically)
        #[derive(Debug, Clone)]
        struct VariableInfo {
            name: &'static str,
            causality: &'static str,
            variability: &'static str,
            settable_in_instantiated: bool,
            settable_in_initialization: bool,
            settable_in_event: bool,
            settable_in_continuous: bool,
            gettable: bool,
            is_derivative: bool,
        }

        let variable_definitions = [
            VariableInfo {
                name: "h",
                causality: "output",
                variability: "continuous",
                settable_in_instantiated: true,
                settable_in_initialization: true,
                settable_in_event: true,
                settable_in_continuous: true,
                gettable: true,
                is_derivative: false,
            },
            VariableInfo {
                name: "v",
                causality: "output",
                variability: "continuous",
                settable_in_instantiated: true,
                settable_in_initialization: true,
                settable_in_event: true,
                settable_in_continuous: true,
                gettable: true,
                is_derivative: false,
            },
            VariableInfo {
                name: "der(h)",
                causality: "local",
                variability: "continuous",
                settable_in_instantiated: false,
                settable_in_initialization: false,
                settable_in_event: false,
                settable_in_continuous: false,
                gettable: true,
                is_derivative: true,
            },
            VariableInfo {
                name: "g",
                causality: "parameter",
                variability: "fixed",
                settable_in_instantiated: true,
                settable_in_initialization: true,
                settable_in_event: false,
                settable_in_continuous: false,
                gettable: true,
                is_derivative: false,
            },
            VariableInfo {
                name: "der(v)",
                causality: "local",
                variability: "continuous",
                settable_in_instantiated: false,
                settable_in_initialization: false,
                settable_in_event: false,
                settable_in_continuous: false,
                gettable: true,
                is_derivative: true,
            },
            VariableInfo {
                name: "e",
                causality: "parameter",
                variability: "tunable",
                settable_in_instantiated: true,
                settable_in_initialization: true,
                settable_in_event: true,
                settable_in_continuous: false,
                gettable: true,
                is_derivative: false,
            },
            VariableInfo {
                name: "v_min",
                causality: "local",
                variability: "fixed",
                settable_in_instantiated: true,
                settable_in_initialization: true,
                settable_in_event: false,
                settable_in_continuous: false,
                gettable: true,
                is_derivative: false,
            },
        ];

        // Look up VRs dynamically from the model - don't hardcode implementation details
        let model_vars = BouncingBall::model_variables();
        let mut variable_vrs = std::collections::HashMap::new();
        for var_info in &variable_definitions {
            let var = model_vars
                .find_by_name(var_info.name)
                .expect(&format!("Variable {} should exist", var_info.name));
            variable_vrs.insert(var_info.name, var.value_reference());
        }

        println!("Testing comprehensive get/set behavior for all variables across all states...");

        // Test all states systematically
        let states = [
            ("Instantiated", 0),
            ("InitializationMode", 1),
            ("EventMode", 2),
            ("ContinuousTimeMode", 3),
        ];

        for (state_name, state_index) in &states {
            println!("\n=== Testing {} state ===", state_name);

            let mut instance = ModelInstance::<BouncingBall>::new(
                "TestBouncingBall".to_string(),
                std::path::PathBuf::from("/tmp"),
                false,
                None,
                instantiation_token,
            )
            .expect("Failed to create model instance");

            // Transition to the target state
            match state_index {
                0 => {
                    // Already in Instantiated state
                }
                1 => {
                    instance
                        .enter_initialization_mode(None, 0.0, None)
                        .expect("Failed to enter initialization mode");
                }
                2 => {
                    instance
                        .enter_initialization_mode(None, 0.0, None)
                        .expect("Failed to enter initialization mode");
                    instance
                        .exit_initialization_mode()
                        .expect("Failed to exit initialization mode");
                    instance
                        .enter_event_mode()
                        .expect("Failed to enter event mode");
                }
                3 => {
                    instance
                        .enter_initialization_mode(None, 0.0, None)
                        .expect("Failed to enter initialization mode");
                    instance
                        .exit_initialization_mode()
                        .expect("Failed to exit initialization mode");
                    instance
                        .enter_event_mode()
                        .expect("Failed to enter event mode");
                    instance
                        .enter_continuous_time_mode()
                        .expect("Failed to enter continuous time mode");
                }
                _ => unreachable!(),
            }

            // Test get operations for all variables (should always work)
            for var_info in &variable_definitions {
                if var_info.gettable {
                    let vr = variable_vrs[var_info.name];
                    let mut values = vec![f64::NAN];
                    let result = instance.get_float64(&[vr], &mut values);
                    assert!(
                        result.is_ok(),
                        "Getting {} (VR={}) should succeed in {} state",
                        var_info.name,
                        vr,
                        state_name
                    );
                }
            }

            // Test set operations based on expected behavior
            for var_info in &variable_definitions {
                let should_be_settable = match state_index {
                    0 => var_info.settable_in_instantiated,
                    1 => var_info.settable_in_initialization,
                    2 => var_info.settable_in_event,
                    3 => var_info.settable_in_continuous,
                    _ => unreachable!(),
                };

                // Only test setting if the variable has a setter case generated
                // (derivatives don't have setter cases)
                if !var_info.is_derivative {
                    let test_value = match var_info.name {
                        "h" => 2.5,
                        "v" => 1.5,
                        "g" => -9.5,
                        "e" => 0.8,
                        "v_min" => 0.05,
                        _ => 1.0,
                    };

                    let vr = variable_vrs[var_info.name];
                    let result = instance.set_float64(&[vr], &[test_value]);

                    if should_be_settable {
                        assert!(
                            result.is_ok(),
                            "Setting {} (VR={}, {}/{}) should succeed in {} state",
                            var_info.name,
                            vr,
                            var_info.causality,
                            var_info.variability,
                            state_name
                        );
                    } else {
                        assert!(
                            result.is_err(),
                            "Setting {} (VR={}, {}/{}) should fail in {} state",
                            var_info.name,
                            vr,
                            var_info.causality,
                            var_info.variability,
                            state_name
                        );
                    }
                }
            }
        }

        println!("\n=== Testing derivative variables have no setter cases ===");
        let mut instance = ModelInstance::<BouncingBall>::new(
            "TestBouncingBall".to_string(),
            std::path::PathBuf::from("/tmp"),
            false,
            None,
            instantiation_token,
        )
        .expect("Failed to create model instance");

        // Derivatives should not have setter cases, so setting them should do nothing
        // (the match case will hit the default `_ => {}` case)
        for var_info in variable_definitions.iter().filter(|v| v.is_derivative) {
            let vr = variable_vrs[var_info.name];
            let result = instance.set_float64(&[vr], &[999.0]);
            assert!(
                result.is_ok(),
                "Setting derivative {} (VR={}) should succeed but do nothing",
                var_info.name,
                vr
            );
        }

        println!("\nAll comprehensive get/set behavior tests passed!");
    }
}
