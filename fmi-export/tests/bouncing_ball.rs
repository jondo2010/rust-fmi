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
#[model(ModelExchange)]
struct BouncingBall {
    /// Height above ground (state output)
    #[variable(causality = output, state = true, start = 1.0)]
    h: f64,

    /// Velocity of the ball
    #[variable(causality = output, state = true, start = 0.0)]
    #[alias(name="der(h)", causality = local, derivative="h")]
    v: f64,

    /// Gravitational acceleration
    #[variable(causality = parameter, start = -9.81)]
    #[alias(name = "der(v)", causality = local, derivative = "v")]
    g: f64,

    /// Coefficient of restitution (parameter)
    #[variable(causality = parameter, start = 0.7)]
    e: f64,

    /// Minimum velocity threshold (constant)
    #[variable(causality = local, start = 0.1)]
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
    fn test_calculate_values() {
        let mut model = BouncingBall::default();

        // Set initial conditions: ball at height 1m, stationary
        model.h = 1.0;
        model.v = 0.0;
        model.g = -9.81;

        // Calculate derivatives
        let status = model.calculate_values();
        let expected_status = fmi::fmi3::Fmi3Status::from(fmi::fmi3::Fmi3Res::OK);
        assert_eq!(format!("{:?}", status), format!("{:?}", expected_status));

        // Check that derivatives are available as aliases
        // der_h should be an alias for v, der_v should be an alias for g
        // Note: We can't directly access aliases in tests, but the FMI interface will handle this
        assert_eq!(model.v, 0.0); // velocity (which der_h aliases)
        assert_eq!(model.g, -9.81); // gravity (which der_v aliases)
    }

    #[test]
    fn test_falling_ball_derivatives() {
        let mut model = BouncingBall::default();

        // Set conditions: ball falling with velocity -5 m/s
        model.h = 2.0;
        model.v = -5.0;
        model.g = -9.81;

        // Calculate derivatives
        model.calculate_values();

        // Check the actual field values (aliases will be handled by the FMI interface)
        assert_eq!(model.v, -5.0); // velocity (which der_h aliases)
        assert_eq!(model.g, -9.81); // gravity (which der_v aliases)
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
    fn test_complete_bounce_simulation() {
        let mut model = BouncingBall::default();

        // Initial conditions: ball at 1m height, falling at -1 m/s
        model.h = 1.0;
        model.v = -1.0;
        model.g = -9.81;
        model.e = 0.8;
        model.v_min = 0.1;

        // Simulate ball hitting ground
        model.h = -0.001; // Slightly below ground (would be detected by integrator)
        model.v = -3.0; // Significant downward velocity

        // Trigger bounce
        model.event_update().expect("Event update should succeed");

        // Verify bounce occurred correctly (with floating point tolerance)
        assert!(model.h > 0.0, "Ball should be above ground after bounce");
        assert!((model.v - 2.4).abs() < 1e-10); // -(-3.0) * 0.8 = 2.4 (with tolerance)
        assert_eq!(model.g, -9.81); // Still subject to gravity

        // Calculate derivatives after bounce
        model.calculate_values();
        // Check the actual field values (aliases will be handled by FMI interface)
        assert!((model.v - 2.4).abs() < 1e-10); // velocity (which der_h aliases)
        assert_eq!(model.g, -9.81); // gravity (which der_v aliases)
    }

    #[test]
    fn test_set_time() {
        use fmi_export::fmi3::ModelInstance;

        let instantiation_token = BouncingBall::INSTANTIATION_TOKEN;

        let mut instance = ModelInstance::<BouncingBall>::new(
            "TestSetTime".to_string(),
            std::path::PathBuf::from("/tmp"),
            false,
            None,
            instantiation_token,
        )
        .expect("Failed to create model instance for set_time test");

        // Initialize the model
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

        // Test setting different time values
        let test_times = [0.0, 0.5, 1.0, 2.5, 10.0];

        for time in test_times {
            println!("Testing set_time with time: {}", time);

            instance.set_time(time).expect("Failed to set time");

            // Verify time was set correctly by checking internal state
            // Note: The time is stored internally but there's no direct getter
            // We can verify by ensuring the call succeeded without error
            println!("  → Successfully set time to {}", time);
        }

        println!("All set_time tests passed!");
    }
}
