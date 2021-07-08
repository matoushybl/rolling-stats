//! Integer types in Rust do not allow for converting them into f32 using the `From/Into` traits
//! as these conversions are lossy.
//!
//! The following trait allows for workarounding this problem via a custom trait.
pub trait LossyF32Convertible {
    /// Convert `self` into a float.
    fn convert(&self) -> f32;
}

impl LossyF32Convertible for i32 {
    fn convert(&self) -> f32 {
        *self as f32
    }
}
