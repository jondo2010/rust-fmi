#[cfg(false)] // Deaktiviert für jetzt - wird mit dem neuen Procedural Macro System ersetzt
#[test_log::test]
#[test]
fn test_van_der_pol() {
    use fmi_export::store::{Desc, ScalarType, Store, Tables, vr_pack};

    // VR enum with type safety and vr_pack encoding
    #[repr(u32)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum ValRef {
        Time = 0,  // offset 0
        X0 = 1,    // offset 1
        DerX0 = 2, // offset 2
        X1 = 3,    // offset 3
        DerX1 = 4, // offset 4
        Mu = 5,    // offset 5
    }

    impl From<binding::fmi3ValueReference> for ValRef {
        fn from(value: binding::fmi3ValueReference) -> Self {
            match value {
                0 => ValRef::Time,
                1 => ValRef::X0,
                2 => ValRef::DerX0,
                3 => ValRef::X1,
                4 => ValRef::DerX1,
                5 => ValRef::Mu,
                _ => panic!("Invalid value reference: {}", value),
            }
        }
    }

    impl From<ValRef> for binding::fmi3ValueReference {
        fn from(value: ValRef) -> Self {
            value as u32
        }
    }

    #[derive(Debug, Default)]
    struct VanDerPol {
        x0: f64,
        der_x0: f64,
        x1: f64,
        der_x1: f64,
        mu: f64,
    }

    impl GetSet for VanDerPol {
        type ValueRef = binding::fmi3ValueReference;

        fn get_float64(&mut self, vrs: &[Self::ValueRef], values: &mut [f64]) -> Fmi3Status {
            for (vr, value) in vrs.iter().zip(values.iter_mut()) {
                match ValRef::from(*vr) {
                    ValRef::Time => *value = self.x0, // Time is x0 for this model
                    ValRef::X0 => *value = self.x0,
                    ValRef::DerX0 => *value = self.der_x0,
                    ValRef::X1 => *value = self.x1,
                    ValRef::DerX1 => *value = self.der_x1,
                    ValRef::Mu => *value = self.mu,
                }
            }
            Fmi3Res::OK.into()
        }

        fn set_float64(&mut self, vrs: &[Self::ValueRef], values: &[f64]) -> Fmi3Status {
            for (vr, value) in vrs.iter().zip(values.iter()) {
                match ValRef::from(*vr) {
                    ValRef::Time => self.x0 = *value,
                    ValRef::X0 => self.x0 = *value,
                    ValRef::DerX0 => self.der_x0 = *value,
                    ValRef::X1 => self.x1 = *value,
                    ValRef::DerX1 => self.der_x1 = *value,
                    ValRef::Mu => self.mu = *value,
                }
            }
            Fmi3Res::OK.into()
        }
    }

    impl ModelData for VanDerPol {
        type ValueRef = ValRef;

        fn set_start_values(&mut self) {
            // Set Van Der Pol initial conditions
            self.x0 = 2.0;
            self.x1 = 0.0;
            self.mu = 1.0;
        }

        fn calculate_values(&mut self) -> Fmi3Status {
            // Get current values from state
            let x0 = self.x0;
            let x1 = self.x1;
            let mu = self.mu;

            // Calculate derivatives using Van Der Pol equation
            self.der_x0 = x1;
            self.der_x1 = mu * ((1.0 - x0 * x0) * x1) - x0;

            Fmi3Res::OK.into()
        }
    }

    // Test the new implementation with ModelInstance
    let mut instance: ModelInstance<VanDerPol> = ModelInstance::new(
        "test".to_string(),
        std::path::PathBuf::from("/tmp"),
        false,
        None, // No logging callback
    );
    instance.enter_initialization_mode(None, 0.0, None);
    let mut vals = [0.0f64; 3];
    instance.get_float64(
        &[ValRef::X0 as u32, ValRef::X1 as u32, ValRef::Mu as u32],
        &mut vals,
    );
    assert_eq!(vals[0], 2.0);
    assert_eq!(vals[1], 0.0);
    assert_eq!(vals[2], 1.0);

    // Test calculate_values
    instance.exit_initialization_mode();
    let mut ders = [0.0f64; 2];
    instance.get_float64(&[ValRef::DerX0 as u32, ValRef::DerX1 as u32], &mut ders);
    assert_eq!(ders[0], 0.0); // der_x0 = x1 = 0.0
    assert_eq!(ders[1], -2.0); // mu * ((1 - x0*x0) * x1) - x0 = 1 * ((1 - 4) * 0) - 2 = -2

    fmi_export::export_fmu!(VanDerPol);
}
