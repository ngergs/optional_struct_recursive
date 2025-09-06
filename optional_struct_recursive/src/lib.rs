use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

/// Marker trait that signals that this struct has a corresponding type where all potential
/// inner fields are optional.
/// In detail this means that an `Option<T::Optioned>` should allow for every combination
/// of itself being set as well as just partial subfields of itself being set.
/// Hence, for types without inner structure like `i32` the `Optioned` type will resolve to itself,
/// as e.g. `Option<i32>` already expresses the needed granularity.
pub trait Optionable {
    type Optioned;
}

// Blanket implementation for references to `Optionalable` types.
impl<'a, T: Optionable> Optionable for &'a T {
    type Optioned = &'a T::Optioned;
}

/// Helper macro to generate an impl for `Optionalable` where the `Optioned` type
/// resolves to itself for types without inner structure like primitives (e.g. `i32`).
macro_rules! impl_optional_self {
    ($($t:ty),* $(,)?) => {
        $(impl Optionable for $t{
            type Optioned = $t;
        })*
    };
}

impl_optional_self!(
    // Rust primitives don't have inner structure, https://doc.rust-lang.org/rust-by-example/primitives.html
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64, char, bool,
    // Other types without inner structure
    String,
);

/// Helper macro to generate an impl for `Optionable` for Containers.
/// Containers can be made optional by getting a corresponding container over the associated optional type.
macro_rules! impl_container {
    ($($t:ident),* $(,)?) => {
        $(impl<T: Optionable> Optionable for $t<T>{
            type Optioned = $t<T::Optioned>;
        })*
    };
}

impl_container!(
    // Collections without an extra key, https://doc.rust-lang.org/std/collections/index.html
    Vec, VecDeque, LinkedList, HashSet, BTreeSet, BinaryHeap,
    // Smart pointer and sync-container
    Box, Rc, Arc, RefCell, Mutex
);

/// Helper macro to generate an impl for `Optionable` for Maps.
/// Maps can be made optional by getting a corresponding map over the associated optional type.
macro_rules! impl_map {
    ($($t:ident),* $(,)?) => {
        $(impl<K,T: Optionable> Optionable for $t<K,T>{
            type Optioned = $t<K,T::Optioned>;
        })*
    };
}

impl_map!(HashMap, BTreeMap,);

#[cfg(test)]
mod tests {
    use crate::Optionable;
    use std::collections::{BTreeMap, HashMap};

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
    /// Check that `Vec` implements optionable as an example container.
    fn container() {
        let a = vec![1, 2, 3];
        let _: <Vec<i64> as Optionable>::Optioned = a;
    }

    #[test]
    /// Check that `HashMap` and `BTreeMap` implements optionable.
    fn map() {
        let a = HashMap::from([(1, "a".to_owned())]);
        let _: <HashMap<i32, String> as Optionable>::Optioned = a;

        let a = BTreeMap::from([(1, "a".to_owned())]);
        let _: <BTreeMap<i32, String> as Optionable>::Optioned = a;
    }
}
