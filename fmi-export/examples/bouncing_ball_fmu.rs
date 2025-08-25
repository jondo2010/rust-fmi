use fmi::fmi3::Fmi3Res;
/// Example showing how to export a BouncingBall FMU with full C API support
///
/// This example demonstrates the complete FMI 3.0 Model Exchange API including:
/// - fmi3EnterContinuousTimeMode
/// - fmi3SetTime
/// - fmi3SetContinuousStates / fmi3GetContinuousStates
/// - fmi3GetContinuousStateDerivatives
/// - fmi3GetEventIndicators
/// - fmi3CompletedIntegratorStep
///
/// All these methods are thoroughly tested in the test suite.
/// To use this example:
/// 1. Build as a dynamic library: cargo build --example bouncing_ball_fmu
/// 2. The resulting library can be used as an FMU by simulation tools
use fmi_export::{
    FmuModel,
    fmi3::{Model, UserModel},
};

/// BouncingBall FMU model that can be exported as a complete FMU
#[derive(FmuModel, Default, Debug)]
#[model(ModelExchange)]
struct BouncingBallFmu {
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

impl UserModel for BouncingBallFmu {
    fn calculate_values(&mut self) -> Fmi3Status {
        // Derivatives are handled by aliases: der(h) = v, der(v) = g
        Fmi3Res::OK.into()
    }

    fn event_update(&mut self) -> Result<Fmi3Res, fmi::fmi3::Fmi3Error> {
        // Handle ball bouncing off the ground
        if self.h <= 0.0 && self.v < 0.0 {
            self.h = f64::MIN_POSITIVE; // Slightly above ground
            self.v = -self.v * self.e; // Reverse velocity with energy loss

            // Stop bouncing if velocity becomes too small
            if self.v < self.v_min {
                self.v = 0.0;
                self.g = 0.0; // Disable gravity when stopped
            }
        }
        Ok(Fmi3Res::OK)
    }

    fn get_event_indicators(
        &mut self,
        indicators: &mut [f64],
    ) -> Result<Fmi3Res, fmi::fmi3::Fmi3Error> {
        if !indicators.is_empty() {
            // Event indicator for ground contact
            indicators[0] = if self.h == 0.0 && self.v == 0.0 {
                1.0 // Special case: stopped ball
            } else {
                self.h // Height as event indicator
            };
        }
        Ok(Fmi3Res::OK)
    }
}

// Export the FMU with full C API
fmi_export::export_fmu!(BouncingBallFmu);

fn main() {
    println!("BouncingBall FMU example");
    println!("Model name: {}", BouncingBallFmu::MODEL_NAME);
    println!(
        "Instantiation token: {}",
        BouncingBallFmu::INSTANTIATION_TOKEN
    );
    println!(
        "Number of continuous states: {}",
        BouncingBallFmu::get_number_of_continuous_states()
    );
    println!(
        "Number of event indicators: {}",
        BouncingBallFmu::get_number_of_event_indicators()
    );

    // Test the model functionality
    let mut model = BouncingBallFmu::default();
    model.h = 2.0;
    model.v = -1.0;

    println!("\nInitial state:");
    println!("  Height: {} m", model.h);
    println!("  Velocity: {} m/s", model.v);

    // Simulate bounce event
    model.h = -0.001; // Ball hits ground
    model.v = -3.0; // High downward velocity

    println!("\nBefore bounce:");
    println!("  Height: {} m", model.h);
    println!("  Velocity: {} m/s", model.v);

    model.event_update().expect("Event update failed");

    println!("\nAfter bounce:");
    println!("  Height: {} m", model.h);
    println!("  Velocity: {} m/s", model.v);

    // Test event indicators
    let mut indicators = [0.0];
    model
        .get_event_indicators(&mut indicators)
        .expect("Failed to get event indicators");
    println!("  Event indicator: {}", indicators[0]);

    // Test completed integrator step functionality
    println!("\nTesting completed integrator step functionality:");
    test_completed_integrator_step(&mut model);
}

/// Demonstrates the completed_integrator_step functionality
fn test_completed_integrator_step(_model: &mut BouncingBallFmu) {
    use fmi::fmi3::{Common, ModelExchange};
    use fmi_export::fmi3::ModelInstance;

    let instantiation_token = BouncingBallFmu::INSTANTIATION_TOKEN;

    let mut instance = ModelInstance::<BouncingBallFmu>::new(
        "TestCompletedIntegratorStep".to_string(),
        std::path::PathBuf::from("/tmp"),
        false,
        None,
        instantiation_token,
    )
    .expect("Failed to create model instance for integrator test");

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

    // Simulate a few integrator steps
    let test_states = [
        [2.0, -1.0], // h=2.0m, v=-1.0m/s
        [1.8, -1.5], // h=1.8m, v=-1.5m/s (falling)
        [1.5, -2.0], // h=1.5m, v=-2.0m/s (falling faster)
        [1.0, -2.5], // h=1.0m, v=-2.5m/s (approaching ground)
    ];

    for (step, states) in test_states.iter().enumerate() {
        println!(
            "  Step {}: Setting states h={:.1}m, v={:.1}m/s",
            step + 1,
            states[0],
            states[1]
        );

        instance
            .set_continuous_states(states)
            .expect("Failed to set states");

        let mut enter_event_mode = false;
        let mut terminate_simulation = false;

        instance
            .completed_integrator_step(false, &mut enter_event_mode, &mut terminate_simulation)
            .expect("Failed to complete integrator step");

        println!(
            "    → Enter event mode: {}, Terminate: {}",
            enter_event_mode, terminate_simulation
        );

        // Get and display event indicators
        let mut indicators = [0.0];
        instance
            .get_event_indicators(&mut indicators)
            .expect("Failed to get event indicators");
        println!("    → Event indicator: {:.3}", indicators[0]);

        // Get derivatives to show the physics
        let mut derivatives = [0.0, 0.0];
        instance
            .get_continuous_state_derivatives(&mut derivatives)
            .expect("Failed to get derivatives");
        println!(
            "    → Derivatives: dh/dt={:.1}m/s, dv/dt={:.1}m/s²",
            derivatives[0], derivatives[1]
        );
    }

    println!("  Completed integrator step test finished successfully!");
}
