//! secp224r1 scalar field elements.

#![allow(clippy::unusual_byte_groupings)]

#[cfg_attr(target_pointer_width = "32", path = "scalar/p224_scalar_32.rs")]
#[cfg_attr(target_pointer_width = "64", path = "scalar/p224_scalar_64.rs")]
#[allow(
    clippy::identity_op,
    clippy::too_many_arguments,
    clippy::unnecessary_cast
)]
mod scalar_impl;

use self::scalar_impl::*;
use crate::{FieldBytes, FieldBytesEncoding, NistP224, Uint, ORDER_HEX};
use core::{
    iter::{Product, Sum},
    ops::{AddAssign, MulAssign, Neg, Shr, ShrAssign, SubAssign},
};
use elliptic_curve::{
    bigint::Limb,
    ff::PrimeField,
    ops::{Invert, Reduce},
    scalar::{FromUintUnchecked, IsHigh},
    subtle::{Choice, ConditionallySelectable, ConstantTimeEq, ConstantTimeGreater, CtOption},
    Curve as _, Error, Result, ScalarPrimitive,
};

#[cfg(feature = "bits")]
use {crate::ScalarBits, elliptic_curve::group::ff::PrimeFieldBits};

#[cfg(feature = "serde")]
use serdect::serde::{de, ser, Deserialize, Serialize};

#[cfg(doc)]
use core::ops::{Add, Mul, Sub};

/// Scalars are elements in the finite field modulo `n`.
///
/// # Trait impls
///
/// Much of the important functionality of scalars is provided by traits from
/// the [`ff`](https://docs.rs/ff/) crate, which is re-exported as
/// `p224::elliptic_curve::ff`:
///
/// - [`Field`](https://docs.rs/ff/latest/ff/trait.Field.html) -
///   represents elements of finite fields and provides:
///   - [`Field::random`](https://docs.rs/ff/latest/ff/trait.Field.html#tymethod.random) -
///     generate a random scalar
///   - `double`, `square`, and `invert` operations
///   - Bounds for [`Add`], [`Sub`], [`Mul`], and [`Neg`] (as well as `*Assign` equivalents)
///   - Bounds for [`ConditionallySelectable`] from the `subtle` crate
/// - [`PrimeField`](https://docs.rs/ff/latest/ff/trait.PrimeField.html) -
///   represents elements of prime fields and provides:
///   - `from_repr`/`to_repr` for converting field elements from/to big integers.
///   - `multiplicative_generator` and `root_of_unity` constants.
/// - [`PrimeFieldBits`](https://docs.rs/ff/latest/ff/trait.PrimeFieldBits.html) -
///   operations over field elements represented as bits (requires `bits` feature)
///
/// Please see the documentation for the relevant traits for more information.
#[derive(Clone, Copy, Debug, PartialOrd, Ord)]
pub struct Scalar(Uint);

primeorder::impl_mont_field_element!(
    NistP224,
    Scalar,
    FieldBytes,
    Uint,
    NistP224::ORDER,
    fiat_p224_scalar_montgomery_domain_field_element,
    fiat_p224_scalar_from_montgomery,
    fiat_p224_scalar_to_montgomery,
    fiat_p224_scalar_add,
    fiat_p224_scalar_sub,
    fiat_p224_scalar_mul,
    fiat_p224_scalar_opp,
    fiat_p224_scalar_square
);

impl Scalar {
    /// Compute [`Scalar`] inversion: `1 / self`.
    pub fn invert(&self) -> CtOption<Self> {
        todo!("`invert` not yet implemented")
    }

    /// Compute modular square root.
    pub fn sqrt(&self) -> CtOption<Self> {
        todo!("`sqrt` not yet implemented")
    }

    /// Right shifts the scalar.
    ///
    /// Note: not constant-time with respect to the `shift` parameter.
    pub const fn shr_vartime(&self, shift: usize) -> Scalar {
        Self(self.0.shr_vartime(shift))
    }
}

impl AsRef<Scalar> for Scalar {
    fn as_ref(&self) -> &Scalar {
        self
    }
}

impl FromUintUnchecked for Scalar {
    type Uint = Uint;

    fn from_uint_unchecked(uint: Self::Uint) -> Self {
        Self::from_uint_unchecked(uint)
    }
}

impl Invert for Scalar {
    type Output = CtOption<Self>;

    fn invert(&self) -> CtOption<Self> {
        self.invert()
    }
}

impl IsHigh for Scalar {
    fn is_high(&self) -> Choice {
        const MODULUS_SHR1: Uint = NistP224::ORDER.shr_vartime(1);
        self.to_canonical().ct_gt(&MODULUS_SHR1)
    }
}

impl Shr<usize> for Scalar {
    type Output = Self;

    fn shr(self, rhs: usize) -> Self::Output {
        self.shr_vartime(rhs)
    }
}

impl Shr<usize> for &Scalar {
    type Output = Scalar;

    fn shr(self, rhs: usize) -> Self::Output {
        self.shr_vartime(rhs)
    }
}

impl ShrAssign<usize> for Scalar {
    fn shr_assign(&mut self, rhs: usize) {
        *self = *self >> rhs;
    }
}

impl PrimeField for Scalar {
    type Repr = FieldBytes;

    const MODULUS: &'static str = ORDER_HEX;
    const CAPACITY: u32 = 223;
    const NUM_BITS: u32 = 224;
    const TWO_INV: Self = Self::ZERO; // TODO
    const MULTIPLICATIVE_GENERATOR: Self = Self::from_u64(2);
    const S: u32 = 2;
    #[cfg(target_pointer_width = "32")]
    const ROOT_OF_UNITY: Self =
        Self::from_hex("317fd4f4d5947c88975e7ca95d8c1164ceed46e611c9e5bafaa1aa3d");
    #[cfg(target_pointer_width = "64")]
    const ROOT_OF_UNITY: Self =
        Self::from_hex("00000000317fd4f4d5947c88975e7ca95d8c1164ceed46e611c9e5bafaa1aa3d");
    const ROOT_OF_UNITY_INV: Self = Self::ZERO; // TODO
    const DELTA: Self = Self::from_u64(16);

    // NOTE: t = 0x3fffffffffffffffffffffffffffc5a8b82e3c0f84f74a5157170a8f

    #[inline]
    fn from_repr(bytes: FieldBytes) -> CtOption<Self> {
        Self::from_bytes(&bytes)
    }

    #[inline]
    fn to_repr(&self) -> FieldBytes {
        self.to_bytes()
    }

    #[inline]
    fn is_odd(&self) -> Choice {
        self.is_odd()
    }
}

#[cfg(feature = "bits")]
impl PrimeFieldBits for Scalar {
    type ReprBits = fiat_p224_scalar_montgomery_domain_field_element;

    fn to_le_bits(&self) -> ScalarBits {
        self.to_canonical().to_words().into()
    }

    fn char_le_bits() -> ScalarBits {
        NistP224::ORDER.to_words().into()
    }
}

impl Reduce<Uint> for Scalar {
    type Bytes = FieldBytes;

    fn reduce(w: Uint) -> Self {
        let (r, underflow) = w.sbb(&NistP224::ORDER, Limb::ZERO);
        let underflow = Choice::from((underflow.0 >> (Limb::BITS - 1)) as u8);
        Self::from_uint_unchecked(Uint::conditional_select(&w, &r, !underflow))
    }

    #[inline]
    fn reduce_bytes(bytes: &FieldBytes) -> Self {
        let w = <Uint as FieldBytesEncoding<NistP224>>::decode_field_bytes(bytes);
        Self::reduce(w)
    }
}

impl From<ScalarPrimitive<NistP224>> for Scalar {
    fn from(w: ScalarPrimitive<NistP224>) -> Self {
        Scalar::from(&w)
    }
}

impl From<&ScalarPrimitive<NistP224>> for Scalar {
    fn from(w: &ScalarPrimitive<NistP224>) -> Scalar {
        Scalar::from_uint_unchecked(*w.as_uint())
    }
}

impl From<Scalar> for ScalarPrimitive<NistP224> {
    fn from(scalar: Scalar) -> ScalarPrimitive<NistP224> {
        ScalarPrimitive::from(&scalar)
    }
}

impl From<&Scalar> for ScalarPrimitive<NistP224> {
    fn from(scalar: &Scalar) -> ScalarPrimitive<NistP224> {
        ScalarPrimitive::new(scalar.into()).unwrap()
    }
}

impl From<Scalar> for FieldBytes {
    fn from(scalar: Scalar) -> Self {
        scalar.to_repr()
    }
}

impl From<&Scalar> for FieldBytes {
    fn from(scalar: &Scalar) -> Self {
        scalar.to_repr()
    }
}

impl From<Scalar> for Uint {
    fn from(scalar: Scalar) -> Uint {
        Uint::from(&scalar)
    }
}

impl From<&Scalar> for Uint {
    fn from(scalar: &Scalar) -> Uint {
        scalar.to_canonical()
    }
}

// TODO
// impl From<&SecretKey> for Scalar {
//     fn from(secret_key: &SecretKey) -> Scalar {
//         *secret_key.to_nonzero_scalar()
//     }
// }

impl TryFrom<Uint> for Scalar {
    type Error = Error;

    fn try_from(w: Uint) -> Result<Self> {
        Option::from(Self::from_uint(w)).ok_or(Error)
    }
}
