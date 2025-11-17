/// A runtime clock type for FMU variables
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Clock(pub bool);

impl std::ops::Deref for Clock {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Clock {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// A runtime binary type for FMU variables
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Binary(pub Vec<u8>);

impl std::ops::Deref for Binary {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Binary {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Trait for initializing fields from start value expressions
pub trait InitializeFromStart<T> {
    fn set_from_start(&mut self, value: T);
}

/// Default implementation for most types - direct assignment
impl<T> InitializeFromStart<T> for T {
    fn set_from_start(&mut self, value: T) {
        *self = value;
    }
}

/// Special implementation for Binary to handle byte string literals
impl InitializeFromStart<&[u8]> for Binary {
    fn set_from_start(&mut self, value: &[u8]) {
        *self = Binary(value.to_vec());
    }
}

/// Special implementation for Binary to handle byte array literals
impl<const N: usize> InitializeFromStart<&[u8; N]> for Binary {
    fn set_from_start(&mut self, value: &[u8; N]) {
        *self = Binary(value.to_vec());
    }
}
