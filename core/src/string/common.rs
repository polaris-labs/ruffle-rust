use super::{ops, AvmString, WStr, WStrMut, WString};
use std::cmp;
use std::fmt;
use std::hash;

#[cold]
pub(super) fn panic_on_invalid_length(len: usize) -> ! {
    panic!("Too many code units in Ruffle string (len = {})", len)
}

/// A raw string buffer containing `u8` or `u16` code units.
#[derive(Copy, Clone, Debug)]
pub enum Units<T, U> {
    /// A buffer containing `u8` code units, interpreted as LATIN-1.
    Bytes(T),
    /// A buffer containing `u16` code units, interpreted as UTF-16
    /// but allowing unpaired surrogates.
    Wide(U),
}

impl<T: AsRef<[u8]>, U: AsRef<[u16]>> Units<T, U> {
    #[inline]
    pub(super) fn len(&self) -> usize {
        match self {
            Units::Bytes(buf) => buf.as_ref().len(),
            Units::Wide(buf) => buf.as_ref().len(),
        }
    }
}

/// Generate `From` implementations for `Units` type.
macro_rules! units_from {
    (impl[$($generics:tt)*] Units<$ty_bytes:ty, $ty_wide:ty>; $($rest:tt)*) => {
        units_from! {
            impl[$($generics)*] Units<$ty_bytes, $ty_wide> {
                units: Units<$ty_bytes, $ty_wide> => units
            } $($rest)*
        }
    };

    (impl[$($generics:tt)*] Units<$ty_bytes:ty, $ty_wide:ty> {
        $units:ident : Units<$from_bytes:ty, $from_wide:ty> => $expr:expr
    } $($rest:tt)*) => {
        impl<$($generics)*> From<$from_bytes> for Units<$ty_bytes, $ty_wide> {
            #[inline]
            fn from($units: $from_bytes) -> Self {
                Units::Bytes($expr)
            }
        }

        impl<$($generics)*> From<$from_wide> for Units<$ty_bytes, $ty_wide> {
            #[inline]
            fn from($units: $from_wide) -> Self {
                Units::Wide($expr)
            }
        }

        units_from! { $($rest)* }
    };

    () => {};
}

units_from! {
    impl['a] Units<&'a [u8], &'a [u16]>;

    impl['a] Units<&'a mut [u8], &'a mut [u16]>;

    impl['a, const N: usize] Units<&'a [u8], &'a [u16]> {
        units: Units<&'a [u8; N], &'a [u16; N]> => &units[..]
    }

    impl['a, const N: usize] Units<&'a mut [u8], &'a mut [u16]> {
        units: Units<&'a mut [u8; N], &'a mut [u16; N]> => &mut units[..]
    }

    impl[] Units<Vec<u8>, Vec<u16>>;
}

// TODO: Once GATs are here, this should become an actual trait.
macro_rules! impl_str_methods {
    (
        lifetime: $lt:lifetime;
        $self:ident: $receiver:ty;
        deref: $deref:expr;
    ) => {
        /// Provides access to the underlying buffer.
        #[inline]
        pub fn units($self: $receiver) -> crate::string::Units<&$lt [u8], &$lt [u16]> {
            crate::string::slice::units($deref)
        }

        /// Returns `true` if `self` is a wide string.
        #[inline]
        pub fn is_wide($self: $receiver) -> bool {
            crate::string::slice::raw_str($deref).is_wide()
        }

        /// Returns the number of code units.
        #[inline]
        pub fn len($self: $receiver) -> usize {
            crate::string::slice::raw_str($deref).len()
        }

        /// Returns `true` if `self` contains no code units.
        #[inline]
        pub fn is_empty($self: $receiver) -> bool {
            $self.len() == 0
        }

        /// Returns the `i`th code unit of `self`; panics if the index is out of range.
        #[inline]
        pub fn get($self: $receiver, i: usize) -> u16 {
            $self.try_get(i).expect("string index out of bounds")
        }

        /// Returns the `i`th code unit of `self`, or `None` if the index is out of range.
        #[inline]
        pub fn try_get($self: $receiver, i: usize) -> Option<u16> {
            crate::string::slice::try_index($deref, i)
        }

        /// Returns a subslice of `self`; panics if the slice indices are out of range.
        #[inline]
        pub fn slice<R: std::ops::RangeBounds<usize>>($self: $receiver, range: R) -> crate::string::WStr<$lt> {
            $self.try_slice(range).expect("string indices out of bounds")
        }

        /// Returns a subslice of `self`, or `None` if the slice indices are out of range.
        #[inline]
        pub fn try_slice<R: std::ops::RangeBounds<usize>>($self: $receiver, range: R) -> Option<crate::string::WStr<$lt>> {
            crate::string::slice::try_slice($deref, range)
        }

        /// Iterates over the code units of `self`.
        #[inline]
        pub fn iter($self: $receiver) -> crate::string::ops::Iter<$lt> {
            crate::string::ops::str_iter($deref)
        }

        #[inline]
        /// Tests if two strings are equal, ignoring case as done by the Flash Player.
        /// Note that the case mapping is different than Rust's case mapping.
        pub fn eq_ignore_case($self: $receiver, other: WStr<'_>) -> bool {
            crate::string::ops::str_eq_ignore_case($deref, other)
        }

        #[inline]
        /// Compares two strings, ignoring case as done by the Flash Player.
        /// Note that the case mapping is different than Rust's case mapping.
        pub fn cmp_ignore_case($self: $receiver, other: WStr<'_>) -> std::cmp::Ordering {
            crate::string::ops::str_cmp_ignore_case($deref, other)
        }

        /// Analogue of [`str::find`].
        // TODO: add our own Pattern trait to support several kinds of needles?
        #[inline]
        pub fn find($self: $receiver, needle: WStr<'_>) -> Option<usize> {
            crate::string::ops::str_find($deref, needle)
        }

        /// Analogue of [`str::rfind`].
        // TODO: add our own Pattern trait to support several kinds of needles?
        #[inline]
        pub fn rfind($self: $receiver, needle: WStr<'_>) -> Option<usize> {
            crate::string::ops::str_rfind($deref, needle)
        }

        /// Analogue of [`str::split`].
        // TODO: add our own Pattern trait to support several kinds of needles?
        #[inline]
        pub fn split<'s>($self: $receiver, separator: WStr<'s>) -> crate::string::ops::Split<$lt, 's> {
            crate::string::ops::str_split($deref, separator)
        }
    }
}

macro_rules! impl_str_mut_methods {
    (
        lifetime: $lt:lifetime;
        $self:ident: $receiver:ty;
        deref_mut: $deref:expr;
    ) => {
        /// Provides mutable access to the underlying buffer.
        #[inline]
        pub fn units_mut($self: $receiver) -> crate::string::Units<&$lt mut [u8], &$lt mut [u16]> {
            crate::string::slice::units_mut($deref)
        }

        /// Returns a mutable subslice of `self`; panics if the slice indices are out of range.
        #[inline]
        pub fn slice_mut<R: std::ops::RangeBounds<usize>>($self: $receiver, range: R) -> crate::string::WStrMut<$lt> {
            $deref.try_slice_mut(range)
                .expect("string indices out of bounds")
        }

        /// Returns a mutable subslice of `self`, or `None` if the slice indices are out of range.
        #[inline]
        pub fn try_slice_mut<R: std::ops::RangeBounds<usize>>($self: $receiver, range: R) -> Option<crate::string::WStrMut<$lt>> {
            crate::string::slice::try_slice_mut($deref, range)
        }
    }
}
pub trait BorrowWStr {
    fn borrow(&self) -> WStr<'_>;
}

pub trait BorrowWStrMut {
    fn borrow_mut(&mut self) -> WStrMut<'_>;
}

macro_rules! impl_str_traits {
    (@eq_ord impl[$($generics:tt)*] for $ty:ty, $ty2:ty) => {
        impl<'_0, $($generics)*> cmp::PartialEq<&'_0 $ty2> for $ty {
            #[inline]
            fn eq(&self, other: &&'_0 $ty2) -> bool {
                ops::str_eq(BorrowWStr::borrow(self), WStr::from_units(*other))
            }
        }

        impl<'_0, $($generics)*> cmp::PartialOrd<&'_0 $ty2> for $ty {
            #[inline]
            fn partial_cmp(&self, other: &&'_0 $ty2) -> Option<cmp::Ordering> {
                Some(ops::str_cmp(BorrowWStr::borrow(self), WStr::from_units(*other)))
            }
        }

        impl<'_0, $($generics)*> cmp::PartialEq<$ty> for &'_0 $ty2 {
            #[inline]
            fn eq(&self, other: &$ty) -> bool {
                ops::str_eq(WStr::from_units(*self), BorrowWStr::borrow(other))
            }
        }

        impl<'_0, $($generics)*> cmp::PartialOrd<$ty> for &'_0 $ty2 {
            #[inline]
            fn partial_cmp(&self, other: &$ty) -> Option<cmp::Ordering> {
                Some(ops::str_cmp(WStr::from_units(*self), BorrowWStr::borrow(other)))
            }
        }
    };
    (impl[$($generics:tt)*] for $ty:ty; $($rest:tt)*) => {
        impl<$($generics)*> fmt::Display for $ty {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                ops::str_fmt(BorrowWStr::borrow(self), f)
            }
        }

        impl<$($generics)*> fmt::Debug for $ty {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                ops::str_debug_fmt(BorrowWStr::borrow(self), f)
            }
        }

        impl<$($generics)*> cmp::Eq for $ty {}

        impl<$($generics)* Other: BorrowWStr> cmp::PartialEq<Other> for $ty {
            #[inline]
            fn eq(&self, other: &Other) -> bool {
                ops::str_eq(BorrowWStr::borrow(self), BorrowWStr::borrow(other))
            }
        }

        impl<$($generics)*> cmp::Ord for $ty {
            #[inline]
            fn cmp(&self, other: &Self) -> cmp::Ordering {
                ops::str_cmp(BorrowWStr::borrow(self), BorrowWStr::borrow(other))
            }
        }

        impl<$($generics)* Other: BorrowWStr> cmp::PartialOrd<Other> for $ty {
            #[inline]
            fn partial_cmp(&self, other: &Other) -> Option<cmp::Ordering> {
                Some(ops::str_cmp(BorrowWStr::borrow(self), BorrowWStr::borrow(other)))
            }
        }

        impl_str_traits! { @eq_ord impl[$($generics)* const N: usize] for $ty, [u8; N] }
        impl_str_traits! { @eq_ord impl[$($generics)* const N: usize] for $ty, [u16; N] }
        impl_str_traits! { @eq_ord impl[$($generics)*] for $ty, [u8] }
        impl_str_traits! { @eq_ord impl[$($generics)*] for $ty, [u16] }

        impl<$($generics)*> hash::Hash for $ty {
            #[inline]
            fn hash<H: hash::Hasher>(&self, state: &mut H) {
                ops::str_hash(BorrowWStr::borrow(self), state)
            }
        }

        impl<'_0, $($generics)*> IntoIterator for &'_0 $ty {
            type Item = u16;
            type IntoIter = crate::string::Iter<'_0>;

            #[inline]
            fn into_iter(self) -> Self::IntoIter {
                ops::str_iter(BorrowWStr::borrow(self))
            }
        }

        impl_str_traits! { $($rest)* }
    };

    () => {};
}

impl_str_traits! {
    impl['a,] for WStr<'a>;
    impl['a,] for WStrMut<'a>;
    impl[] for WString;
    impl['gc,] for AvmString<'gc>;
}
