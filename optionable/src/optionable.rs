use crate::{Optionable, OptionableConvert};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};
use std::hash::{BuildHasher, Hash};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

/// Represents errors that occur when trying to build a full type from its optioned variant.
pub struct Error {
    /// Fields that are missing
    pub missing_fields: Vec<&'static str>,
}

/// Merges the errors from the two arguments by appending the missing field lists.
#[must_use]
pub fn merge_errors(mut a: Error, mut b: Error) -> Error {
    a.missing_fields.append(&mut b.missing_fields);
    a
}

// Blanket implementation for references to `Optionable` types.
impl<'a, T: Optionable> Optionable for &'a T {
    type Optioned = &'a T::Optioned;
}

/// Helper macro to generate an impl for `Optionalable` where the `Optioned` type
/// resolves to itself for types without inner structure like primitives (e.g. `i32`).
macro_rules! impl_optional_self {
    ($($t:ty),* $(,)?) => {
        $(impl Optionable for $t{
            type Optioned = Self;
        }

        impl OptionableConvert for $t{
            fn into_optioned(self) -> Self::Optioned {
                self
            }

            fn try_from_optioned(value: Self::Optioned) -> Result<Self, Error> {
                Ok(value)
            }

            fn merge(&mut self, other: Self::Optioned) -> Result<(), Error> {
                *self = other;
                Ok(())
            }
        })*
    };
}

impl_optional_self!(
    // Rust primitives don't have inner structure, https://doc.rust-lang.org/rust-by-example/primitives.html
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64, char, bool,
    // Other types without inner structure
    String, &str
);
pub(crate) use impl_optional_self;

/// Helper macro to generate an impl for `Optionable` for Containers.
/// Containers can be made optional by getting a corresponding container over the associated optional type.
macro_rules! impl_container {
    ($($t:ident),* $(,)?) => {
        $(impl<T: Optionable> Optionable for $t<T>{
            type Optioned = $t<T::Optioned>;
        })*
    };
}

/// Static macro to hold the inner impl for an `IntoIterator` type
macro_rules! inner_impl_convert_into_iter {
    () => {
        fn into_optioned(self) -> Self::Optioned {
            self.into_iter().map(T::into_optioned).collect()
        }

        fn try_from_optioned(value: Self::Optioned) -> Result<Self, Error> {
            value.into_iter().map(T::try_from_optioned).collect()
        }

        fn merge(&mut self, other: Self::Optioned) -> Result<(), Error> {
            *self = Self::try_from_optioned(other)?;
            Ok(())
        }
    };
}

/// Helper macro to generate an impl for `OptionableConvert` for Containers with linear structure (e.g. `Vec`).
macro_rules! impl_container_convert_linear {
    ($($t:ident),* $(, where=$w:ident)?) => {
        $(impl<T: OptionableConvert> OptionableConvert for $t<T>{
            inner_impl_convert_into_iter!();
        })*
    };
}

/// Helper macro to generate an impl for `OptionableConvert` for Containers with linear structure that require `cmp:Ord` (e.g. `BTreeSet`).
macro_rules! impl_container_convert_linear_ord {
    ($($t:ident),* $(, where=$w:ident)?) => {
        $(impl<T: OptionableConvert> OptionableConvert for $t<T>
            where T: Ord,
                  T::Optioned: Ord{
            inner_impl_convert_into_iter!();
        })*
    };
}

impl_container!(
    Option,
    // Collections without an extra key, https://doc.rust-lang.org/std/collections/index.html
    Vec, VecDeque, LinkedList, BTreeSet, BinaryHeap, // Smart pointer and sync-container
    Box, Rc, Arc, RefCell, Mutex,
);
impl_container_convert_linear!(Vec, VecDeque, LinkedList);
impl_container_convert_linear_ord!(BTreeSet, BinaryHeap);

impl<T: OptionableConvert> OptionableConvert for Box<T> {
    fn into_optioned(self) -> Self::Optioned {
        let inner = *self;
        Box::new(inner.into_optioned())
    }

    fn try_from_optioned(value: Self::Optioned) -> Result<Self, Error> {
        let inner = *value;
        Ok(Box::new(T::try_from_optioned(inner)?))
    }

    fn merge(&mut self, other: Self::Optioned) -> Result<(), Error> {
        let inner = &mut **self;
        let other_inner = *other;
        inner.merge(other_inner)?;
        Ok(())
    }
}

impl<T: Optionable, E> Optionable for Result<T, E> {
    type Optioned = Result<T::Optioned, E>;
}

impl<T: Optionable, S> Optionable for HashSet<T, S> {
    type Optioned = HashSet<T::Optioned, S>;
}

impl<T: OptionableConvert, S: Default + BuildHasher> OptionableConvert for HashSet<T, S>
where
    T: Ord + Hash,
    T::Optioned: Ord + Hash,
{
    inner_impl_convert_into_iter!();
}

impl<K, T: Optionable> Optionable for BTreeMap<K, T> {
    type Optioned = BTreeMap<K, T::Optioned>;
}

/// Static macro to hold the inner impl for map-like types
macro_rules! inner_impl_convert_map {
    () => {
        fn into_optioned(self) -> Self::Optioned {
            self.into_iter()
                .map(|(k, v)| (k, T::into_optioned(v)))
                .collect()
        }

        fn try_from_optioned(value: Self::Optioned) -> Result<Self, Error> {
            value
                .into_iter()
                .map(|(k, v)| Ok((k, T::try_from_optioned(v)?)))
                .collect()
        }

        fn merge(&mut self, other: Self::Optioned) -> Result<(), Error> {
            other.into_iter().try_for_each(|(k, v)| {
                self.insert(k, T::try_from_optioned(v)?);
                Ok(())
            })
        }
    };
}

impl<K: Ord, T: OptionableConvert> OptionableConvert for BTreeMap<K, T> {
    inner_impl_convert_map!();
}

impl<K, T: Optionable, S> Optionable for HashMap<K, T, S> {
    type Optioned = HashMap<K, T::Optioned, S>;
}

impl<K: Ord + Hash, T: OptionableConvert> OptionableConvert for HashMap<K, T> {
    inner_impl_convert_map!();
}

#[cfg(test)]
mod tests {
    use crate::Optionable;
    use std::collections::{BTreeMap, HashMap};
    use std::fmt::Error;

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
    /// Check that &str implements `Optionable`.
    fn str() {
        let a = "hello";
        let _: <&str as Optionable>::Optioned = a;
    }

    #[test]
    /// Check that pointer to `Optionable` types implement optionable.
    fn ptr() {
        let a = 2;
        let _: <&i32 as Optionable>::Optioned = &a;
    }

    #[test]
    /// Check that `Vec` implements optionable as an example container.
    fn container() {
        let a = vec![1, 2, 3];
        let _: <Vec<i64> as Optionable>::Optioned = a;
    }

    #[test]
    /// Check that `Result` implements optionable.
    fn result() {
        let a = Ok::<_, Error>(42);
        let _: <Result<i32, _> as Optionable>::Optioned = a;
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
