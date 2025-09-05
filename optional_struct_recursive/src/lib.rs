use std::rc::Rc;
use std::sync::{Arc, Mutex};

/// Marker trait that signals that this struct has a corresponding type where all potential
/// Optioned fields are optional.
/// In detail this means that an `Option<T::Optioned>` should allow for every combination
/// of itself being set as well as just partial subfields of itself being set.
/// Hence, for types without Optioned structure like `i32` the `Optioned` type will resolve to itself,
/// as e.g. `Option<i32>` already expresses the needed granularity.
pub trait Optionable {
    type Optioned: Optionable;
}

// Blanket implementation for references to `Optionalable` types.
impl<'a, T: Optionable + ?Sized> Optionable for &'a T {
    type Optioned = &'a T::Optioned;
}

/// Helper macro to generate an impl for `Optionalable` where the `Optional` type
/// resolves to itself for types without Optioned structure like primitives (e.g. `i32`).
macro_rules! impl_optional_self {
    ($($t:ty),* $(,)?) => {
        $(impl Optionable for $t{
            type Optioned = $t;
        })*
    };
}

impl_optional_self!(
    // Rust primitives don't have Optioned structure, https://doc.rust-lang.org/rust-by-example/primitives.html
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64, char, bool,
    // Other types without Optioned structure
    String,
);

/// Helper macro to generate an impl for `Optionable` for Containers.
/// Containers can be made optional be getting a container over the corresponding associated optional type.
macro_rules! impl_container {
    ($($t:ident),* $(,)?) => {
        $(impl<T: Optionable> Optionable for $t<T>{
            type Optioned = $t<T::Optioned>;
        })*
    };
}

impl_container!(Vec, Box, Rc, Arc, Mutex);

#[cfg(test)]
mod tests {
    use crate::Optionable;

    #[test]
    /// Check that an exemplary primitive type like `i32` resolves to itself as `Optioned` type.
    /// As all primitives share the same macro-generated code it does not add any value to iterate through
    /// all of them. If we missed a primitive type at the macro invocation we would also miss it at listing
    /// the types for the test.
    fn primitive_types_optioned_self() {
        let a: i32 = 10;
        let _: <i32 as Optionable>::Optioned = a;
    }

    #[test]
    /// Check that `Vec` implements optionable.
    fn container() {
        let a = vec![1, 2, 3];
        let _: <Vec<i64> as Optionable>::Optioned = a;
    }
}
