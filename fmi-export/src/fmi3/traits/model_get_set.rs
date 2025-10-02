use fmi::fmi3::{Fmi3Error, binding};

use crate::fmi3::{Clock, instance::ModelContext};

use super::Model;

/// Macro to generate getter and setter method declarations for the ModelGetSet trait
macro_rules! model_getter_setter {
    ($name:ident, $ty:ty) => {
        paste::paste! {
            /// Get [<$name>] values from the model
            /// Returns the number of elements that were actually read
            fn [<get_ $name>](
                &self,
                _vr: binding::fmi3ValueReference,
                _values: &mut [$ty],
                _context: &ModelContext<M>,
            ) -> Result<usize, Fmi3Error> {
                Err(Fmi3Error::Error)
            }

            /// Set [<$name>] values in the model
            /// Returns the number of elements that were actually written
            fn [<set_ $name>](
                &mut self,
                _vr: binding::fmi3ValueReference,
                _values: &[$ty],
                _context: &ModelContext<M>,
            ) -> Result<usize, Fmi3Error> {
                Err(Fmi3Error::Error)
            }
        }
    };
}

/// Macro to implement ModelGetSet for primitive types
macro_rules! impl_model_get_set_primitive {
    ($name:ident, $ty:ty, $data_type:expr) => {
        paste::paste! {
            impl<M: Model> ModelGetSet<M> for $ty {
                const FIELD_COUNT: usize = 1;
                fn [<get_ $name>](
                    &self,
                    vr: binding::fmi3ValueReference,
                    values: &mut [$ty],
                    _context: &ModelContext<M>,
                ) -> Result<usize, Fmi3Error> {
                    if vr == 0 && !values.is_empty() {
                        values[0] = *self;
                        Ok(1)
                    } else {
                        Err(Fmi3Error::Error)
                    }
                }
                fn [<set_ $name>](
                    &mut self,
                    vr: binding::fmi3ValueReference,
                    values: &[$ty],
                    _context: &ModelContext<M>,
                ) -> Result<usize, Fmi3Error> {
                    if vr == 0 && !values.is_empty() {
                        *self = values[0];
                        Ok(1)
                    } else {
                        Err(Fmi3Error::Error)
                    }
                }
            }

            impl<M: Model, const N: usize> ModelGetSet<M> for [$ty; N] {
                const FIELD_COUNT: usize = N;
                fn [<get_ $name>](
                    &self,
                    vr: binding::fmi3ValueReference,
                    values: &mut [$ty],
                    _context: &ModelContext<M>,
                ) -> Result<usize, Fmi3Error> {
                    if (vr as usize) < N && !values.is_empty() {
                        let len = std::cmp::min(N - (vr as usize), values.len());
                        values[..len].copy_from_slice(&self[(vr as usize)..(vr as usize + len)]);
                        Ok(len)
                    } else {
                        Err(Fmi3Error::Error)
                    }
                }
                fn [<set_ $name>](
                    &mut self,
                    vr: binding::fmi3ValueReference,
                    values: &[$ty],
                    _context: &ModelContext<M>,
                ) -> Result<usize, Fmi3Error> {
                    if (vr as usize) < N && !values.is_empty() {
                        let len = std::cmp::min(N - (vr as usize), values.len());
                        self[(vr as usize)..(vr as usize + len)].copy_from_slice(&values[..len]);
                        Ok(len)
                    } else {
                        Err(Fmi3Error::Error)
                    }
                }
            }
        }
    };
}

pub trait ModelGetSet<M: Model> {
    /// The total number of primitive fields when flattened
    const FIELD_COUNT: usize;

    model_getter_setter!(boolean, bool);
    model_getter_setter!(float32, f32);
    model_getter_setter!(float64, f64);
    model_getter_setter!(int8, i8);
    model_getter_setter!(int16, i16);
    model_getter_setter!(int32, i32);
    model_getter_setter!(int64, i64);
    model_getter_setter!(uint8, u8);
    model_getter_setter!(uint16, u16);
    model_getter_setter!(uint32, u32);
    model_getter_setter!(uint64, u64);
    model_getter_setter!(string, std::ffi::CString);

    /// Get binary values from the model
    /// Returns the sizes of the binary data that were actually read
    fn get_binary(
        &self,
        _vr: binding::fmi3ValueReference,
        _values: &mut [&mut [u8]],
        _context: &ModelContext<M>,
    ) -> Result<Vec<usize>, Fmi3Error> {
        Err(Fmi3Error::Error)
    }

    /// Set binary values in the model
    /// Returns the number of binary elements that were actually written
    fn set_binary(
        &mut self,
        _vr: binding::fmi3ValueReference,
        _values: &[&[u8]],
        _context: &ModelContext<M>,
    ) -> Result<usize, Fmi3Error> {
        Err(Fmi3Error::Error)
    }

    /// Get clock values from the model
    fn get_clock(
        &self,
        _vr: binding::fmi3ValueReference,
        _value: &mut binding::fmi3Clock,
        _context: &ModelContext<M>,
    ) -> Result<(), Fmi3Error> {
        Err(Fmi3Error::Error)
    }

    /// Set clock values in the model
    fn set_clock(
        &mut self,
        _vr: binding::fmi3ValueReference,
        _value: &binding::fmi3Clock,
        _context: &ModelContext<M>,
    ) -> Result<(), Fmi3Error> {
        Err(Fmi3Error::Error)
    }
}

impl_model_get_set_primitive!(boolean, bool, schema::DataType::Boolean);
impl_model_get_set_primitive!(float32, f32, schema::DataType::Float32);
impl_model_get_set_primitive!(float64, f64, schema::DataType::Float64);
impl_model_get_set_primitive!(int8, i8, schema::DataType::Int8);
impl_model_get_set_primitive!(int16, i16, schema::DataType::Int16);
impl_model_get_set_primitive!(int32, i32, schema::DataType::Int32);
impl_model_get_set_primitive!(int64, i64, schema::DataType::Int64);
impl_model_get_set_primitive!(uint8, u8, schema::DataType::Uint8);
impl_model_get_set_primitive!(uint16, u16, schema::DataType::Uint16);
impl_model_get_set_primitive!(uint32, u32, schema::DataType::Uint32);
impl_model_get_set_primitive!(uint64, u64, schema::DataType::Uint64);

impl<M: Model> ModelGetSet<M> for String {
    const FIELD_COUNT: usize = 1;
    fn get_string(
        &self,
        vr: binding::fmi3ValueReference,
        values: &mut [std::ffi::CString],
        _context: &ModelContext<M>,
    ) -> Result<usize, Fmi3Error> {
        if vr == 0 && !values.is_empty() {
            values[0] = std::ffi::CString::new(self.as_str()).unwrap();
            Ok(1)
        } else {
            Err(Fmi3Error::Error)
        }
    }
    fn set_string(
        &mut self,
        vr: binding::fmi3ValueReference,
        values: &[std::ffi::CString],
        _context: &ModelContext<M>,
    ) -> Result<usize, Fmi3Error> {
        if vr == 0 && !values.is_empty() {
            *self = values[0]
                .to_str()
                .map_err(|_| Fmi3Error::Error)?
                .to_string();
            Ok(1)
        } else {
            Err(Fmi3Error::Error)
        }
    }
}

impl<M: Model> ModelGetSet<M> for Clock {
    const FIELD_COUNT: usize = 1;
    fn get_clock(
        &self,
        vr: binding::fmi3ValueReference,
        value: &mut binding::fmi3Clock,
        _context: &ModelContext<M>,
    ) -> Result<(), Fmi3Error> {
        if vr == 0 {
            *value = self.0;
            Ok(())
        } else {
            Err(Fmi3Error::Error)
        }
    }
    fn set_clock(
        &mut self,
        vr: binding::fmi3ValueReference,
        value: &binding::fmi3Clock,
        _context: &ModelContext<M>,
    ) -> Result<(), Fmi3Error> {
        if vr == 0 {
            self.0 = *value;
            Ok(())
        } else {
            Err(Fmi3Error::Error)
        }
    }
}

pub trait ModelGetSetStates {
    /// The number of continuous states in the model
    const NUM_STATES: usize;

    /// Get continuous states from the model
    /// Returns the current values of all continuous state variables
    fn get_continuous_states(&self, states: &mut [f64]) -> Result<(), Fmi3Error>;

    /// Set continuous states in the model
    /// Sets new values for all continuous state variables
    fn set_continuous_states(&mut self, states: &[f64]) -> Result<(), Fmi3Error>;

    /// Get derivatives of continuous states
    /// Returns the first-order time derivatives of all continuous state variables
    fn get_continuous_state_derivatives(
        &mut self,
        derivatives: &mut [f64],
    ) -> Result<(), Fmi3Error>;
}

impl ModelGetSetStates for f64 {
    const NUM_STATES: usize = 1;

    fn get_continuous_states(&self, states: &mut [f64]) -> Result<(), Fmi3Error> {
        if states.is_empty() {
            Err(Fmi3Error::Error)
        } else {
            states[0] = *self;
            Ok(())
        }
    }

    fn set_continuous_states(&mut self, states: &[f64]) -> Result<(), Fmi3Error> {
        if states.is_empty() {
            Err(Fmi3Error::Error)
        } else {
            *self = states[0];
            Ok(())
        }
    }

    fn get_continuous_state_derivatives(
        &mut self,
        derivatives: &mut [f64],
    ) -> Result<(), Fmi3Error> {
        if derivatives.is_empty() {
            Err(Fmi3Error::Error)
        } else {
            derivatives[0] = *self;
            Ok(())
        }
    }
}

impl<const N: usize> ModelGetSetStates for [f64; N] {
    const NUM_STATES: usize = N;

    fn get_continuous_states(&self, states: &mut [f64]) -> Result<(), Fmi3Error> {
        if states.len() < Self::NUM_STATES {
            Err(Fmi3Error::Error)
        } else {
            states[0..N].copy_from_slice(self);
            Ok(())
        }
    }

    fn set_continuous_states(&mut self, states: &[f64]) -> Result<(), Fmi3Error> {
        if states.len() < Self::NUM_STATES {
            Err(Fmi3Error::Error)
        } else {
            self.copy_from_slice(&states[0..N]);
            Ok(())
        }
    }

    fn get_continuous_state_derivatives(
        &mut self,
        derivatives: &mut [f64],
    ) -> Result<(), Fmi3Error> {
        if derivatives.len() < Self::NUM_STATES {
            Err(Fmi3Error::Error)
        } else {
            derivatives[0..N].copy_from_slice(self);
            Ok(())
        }
    }
}
