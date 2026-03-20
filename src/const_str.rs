// This module is adapted from the `monostate` crate by `dtolnay`, MIT license.
//
// ref: https://github.com/dtolnay/monostate/blob/efb63b7ab6bfe73e7ccf20f71d17d3705cff9fcc/src/string.rs

use std::{marker::PhantomData, mem::ManuallyDrop};

/// Allows effectively using `const S: &'static str` type parameter on stable rust.
///
/// You need nightly Rust with `#![feature(adt_const_params)]` for this:
///
/// ```ignore
/// const S: &'static str
/// ```
///
/// So while that's not stable yet, this can be used instead:
///
/// ```ignore
/// S: ConstStr // then refer to the actual string via `S::VALUE`
/// ```
pub trait ConstStr {
    const VALUE: &'static str;
}

#[allow(nonstandard_style)]
pub struct StrLen<const N: usize>;

impl<Str, const LEN: usize> ConstStr for (StrLen<LEN>, Str)
where
    Str: StringBuffer,
{
    const VALUE: &'static str = {
        // force the data into a static memory location
        struct StaticBuffer<T: StringBuffer, const N: usize>(PhantomData<T>);
        impl<T: StringBuffer, const N: usize> StaticBuffer<T, N> {
            const BYTES: [u8; N] = unsafe {
                Cast::<T, N> {
                    encoded: ManuallyDrop::new(T::BYTES),
                }
                .array
            };
        }

        unsafe { std::str::from_utf8_unchecked(&StaticBuffer::<Str, LEN>::BYTES) }
    };
}

union Cast<T: StringBuffer, const N: usize> {
    encoded: ManuallyDrop<T::Type>,
    array: [u8; N],
}

/// # Safety
///
/// - `Type`: contains no padding bytes and has an alignment of 1.
/// - `BYTES` is valid UTF-8
pub unsafe trait StringBuffer {
    type Type: 'static;
    const BYTES: Self::Type;
}

/// A unicode scalar value, [`char`], that is 1 byte long.
pub struct C1<const CH: char>;

/// A unicode scalar value, [`char`], that is 2 bytes long.
pub struct C2<const CH: char>;

/// A unicode scalar value, [`char`], that is 3 bytes long.
pub struct C3<const CH: char>;

/// A unicode scalar value, [`char`], that is 4 bytes long.
pub struct C4<const CH: char>;

/// Mask for 1-byte unicode scalar value sequence start
const TAG_CONT: u8 = 0b1000_0000;

/// Mask for 2-byte unicode scalar value sequence start
const TAG_TWO_B: u8 = 0b1100_0000;

/// Mask for 3-byte unicode scalar value sequence start
const TAG_THREE_B: u8 = 0b1110_0000;

/// Mask for 4-byte unicode scalar value sequence start
const TAG_FOUR_B: u8 = 0b1111_0000;

// Safety:
//
// - `u8` has no padding bytes and an alignment of 1
// - Bytes are valid UTF-8
unsafe impl<const CH: char> StringBuffer for C1<CH> {
    type Type = [u8; 1];
    const BYTES: Self::Type = [CH as u8];
}

// Safety:
//
// - `[u8; 2]` has no padding bytes and an alignment of 1
// - Bytes are valid UTF-8
unsafe impl<const CH: char> StringBuffer for C2<CH> {
    type Type = [u8; 2];
    const BYTES: Self::Type = [
        ((CH as u32 >> 6) & 0x1F) as u8 | TAG_TWO_B,
        (CH as u32 & 0x3F) as u8 | TAG_CONT,
    ];
}

// Safety:
//
// - `[u8; 3]` has no padding bytes and an alignment of 1
// - Bytes are valid UTF-8
unsafe impl<const CH: char> StringBuffer for C3<CH> {
    type Type = [u8; 3];
    const BYTES: Self::Type = [
        ((CH as u32 >> 12) & 0x0F) as u8 | TAG_THREE_B,
        ((CH as u32 >> 6) & 0x3F) as u8 | TAG_CONT,
        (CH as u32 & 0x3F) as u8 | TAG_CONT,
    ];
}

// Safety:
//
// - `[u8; 4]` has no padding bytes and an alignment of 1
// - Bytes are valid UTF-8
unsafe impl<const CH: char> StringBuffer for C4<CH> {
    type Type = [u8; 4];
    const BYTES: Self::Type = [
        ((CH as u32 >> 18) & 0x07) as u8 | TAG_FOUR_B,
        ((CH as u32 >> 12) & 0x3F) as u8 | TAG_CONT,
        ((CH as u32 >> 6) & 0x3F) as u8 | TAG_CONT,
        (CH as u32 & 0x3F) as u8 | TAG_CONT,
    ];
}

// SAFETY:
//
// - `()` has no padding bytes and an alignment of 1
// - `()` is same as `[u8; ()]`, and is always valid UTF-8
unsafe impl StringBuffer for () {
    type Type = ();
    const BYTES: Self::Type = ();
}

#[repr(C)]
pub struct Concat2<A, B>(A, B);

// SAFETY:
//
// - `repr(C)` combined with `Type`s that have alignment 1 and no padding means
//    that the resulting struct also has alignment 1 and no padding, as fields are adjacent.
// - Concatenating valid UTF-8 sequences always results in a valid UTF-8 sequence.
unsafe impl<A, B> StringBuffer for (A, B)
where
    A: StringBuffer,
    B: StringBuffer,
{
    type Type = Concat2<A::Type, B::Type>;
    const BYTES: Self::Type = Concat2(A::BYTES, B::BYTES);
}

#[repr(C)]
pub struct Concat3<A, B, C>(A, B, C);

// SAFETY:
//
// - `repr(C)` combined with `Type`s that have alignment 1 and no padding means
//    that the resulting struct also has alignment 1 and no padding, as fields are adjacent.
// - Concatenating valid UTF-8 sequences always results in a valid UTF-8 sequence.
unsafe impl<A, B, C> StringBuffer for (A, B, C)
where
    A: StringBuffer,
    B: StringBuffer,
    C: StringBuffer,
{
    type Type = Concat3<A::Type, B::Type, C::Type>;
    const BYTES: Self::Type = Concat3(A::BYTES, B::BYTES, C::BYTES);
}

#[repr(C)]
pub struct Concat4<A, B, C, D>(A, B, C, D);

// SAFETY:
//
// - `repr(C)` combined with `Type`s that have alignment 1 and no padding means
//    that the resulting struct also has alignment 1 and no padding, as fields are adjacent.
// - Concatenating valid UTF-8 sequences always results in a valid UTF-8 sequence.
unsafe impl<A, B, C, D> StringBuffer for (A, B, C, D)
where
    A: StringBuffer,
    B: StringBuffer,
    C: StringBuffer,
    D: StringBuffer,
{
    type Type = Concat4<A::Type, B::Type, C::Type, D::Type>;
    const BYTES: Self::Type = Concat4(A::BYTES, B::BYTES, C::BYTES, D::BYTES);
}

#[repr(C)]
pub struct Concat5<A, B, C, D, E>(A, B, C, D, E);

// SAFETY:
//
// - `repr(C)` combined with `Type`s that have alignment 1 and no padding means
//    that the resulting struct also has alignment 1 and no padding, as fields are adjacent.
// - Concatenating valid UTF-8 sequences always results in a valid UTF-8 sequence.
unsafe impl<A, B, C, D, E> StringBuffer for (A, B, C, D, E)
where
    A: StringBuffer,
    B: StringBuffer,
    C: StringBuffer,
    D: StringBuffer,
    E: StringBuffer,
{
    type Type = Concat5<A::Type, B::Type, C::Type, D::Type, E::Type>;
    const BYTES: Self::Type = Concat5(A::BYTES, B::BYTES, C::BYTES, D::BYTES, E::BYTES);
}

#[repr(C)]
pub struct Concat6<A, B, C, D, E, F>(A, B, C, D, E, F);

// SAFETY:
//
// - `repr(C)` combined with `Type`s that have alignment 1 and no padding means
//    that the resulting struct also has alignment 1 and no padding, as fields are adjacent.
// - Concatenating valid UTF-8 sequences always results in a valid UTF-8 sequence.
unsafe impl<A, B, C, D, E, F> StringBuffer for (A, B, C, D, E, F)
where
    A: StringBuffer,
    B: StringBuffer,
    C: StringBuffer,
    D: StringBuffer,
    E: StringBuffer,
    F: StringBuffer,
{
    type Type = Concat6<A::Type, B::Type, C::Type, D::Type, E::Type, F::Type>;
    const BYTES: Self::Type = Concat6(A::BYTES, B::BYTES, C::BYTES, D::BYTES, E::BYTES, F::BYTES);
}
