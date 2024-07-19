//! Lexicon integer types with minimum or maximum acceptable values.

use std::num::{NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8};

use serde::{de::Error, Deserialize};

macro_rules! uint {
    ($primitive:ident, $nz:ident, $lim:ident, $lim_nz:ident, $bounded:ident) => {
        /// An unsigned integer with a maximum value of `MAX`.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, Hash)]
        #[repr(transparent)]
        #[serde(transparent)]
        pub struct $lim<const MAX: $primitive>($primitive);

        impl<const MAX: $primitive> $lim<MAX> {
            /// The smallest value that can be represented by this limited integer type.
            pub const MIN: Self = Self(<$primitive>::MIN);

            /// The largest value that can be represented by this limited integer type.
            pub const MAX: Self = Self(MAX);

            fn new(value: $primitive) -> Result<Self, String> {
                if value > MAX {
                    Err(format!("value is greater than {}", MAX))
                } else {
                    Ok(Self(value))
                }
            }
        }

        impl<const MAX: $primitive> TryFrom<$primitive> for $lim<MAX> {
            type Error = String;

            fn try_from(value: $primitive) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl<'de, const MAX: $primitive> Deserialize<'de> for $lim<MAX> {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let value = Deserialize::deserialize(deserializer)?;
                Self::new(value).map_err(D::Error::custom)
            }
        }

        impl<const MAX: $primitive> From<$lim<MAX>> for $primitive {
            fn from(value: $lim<MAX>) -> Self {
                value.0
            }
        }

        /// An unsigned integer with a minimum value of 1 and a maximum value of `MAX`.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, Hash)]
        #[repr(transparent)]
        #[serde(transparent)]
        pub struct $lim_nz<const MAX: $primitive>($nz);

        impl<const MAX: $primitive> $lim_nz<MAX> {
            /// The smallest value that can be represented by this limited non-zero
            /// integer type.
            pub const MIN: Self = Self($nz::MIN);

            /// The largest value that can be represented by this limited non-zero integer
            /// type.
            pub const MAX: Self = Self(unsafe { $nz::new_unchecked(MAX) });

            fn new(value: $primitive) -> Result<Self, String> {
                if value > MAX {
                    Err(format!("value is greater than {}", MAX))
                } else if let Some(value) = $nz::new(value) {
                    Ok(Self(value))
                } else {
                    Err("value is zero".into())
                }
            }
        }

        impl<const MAX: $primitive> TryFrom<$primitive> for $lim_nz<MAX> {
            type Error = String;

            fn try_from(value: $primitive) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl<'de, const MAX: $primitive> Deserialize<'de> for $lim_nz<MAX> {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let value = Deserialize::deserialize(deserializer)?;
                Self::new(value).map_err(D::Error::custom)
            }
        }

        impl<const MAX: $primitive> From<$lim_nz<MAX>> for $nz {
            fn from(value: $lim_nz<MAX>) -> Self {
                value.0
            }
        }

        impl<const MAX: $primitive> From<$lim_nz<MAX>> for $primitive {
            fn from(value: $lim_nz<MAX>) -> Self {
                value.0.into()
            }
        }

        /// An unsigned integer with a minimum value of `MIN` and a maximum value of `MAX`.
        ///
        /// `MIN` must be non-zero.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, Hash)]
        #[repr(transparent)]
        #[serde(transparent)]
        pub struct $bounded<const MIN: $primitive, const MAX: $primitive>($nz);

        impl<const MIN: $primitive, const MAX: $primitive> $bounded<MIN, MAX> {
            /// The smallest value that can be represented by this bounded integer type.
            pub const MIN: Self = Self(unsafe { $nz::new_unchecked(MIN) });

            /// The largest value that can be represented by this bounded integer type.
            pub const MAX: Self = Self(unsafe { $nz::new_unchecked(MAX) });

            fn new(value: $primitive) -> Result<Self, String> {
                if value < MIN {
                    Err(format!("value is less than {}", MIN))
                } else if value > MAX {
                    Err(format!("value is greater than {}", MAX))
                } else if let Some(value) = $nz::new(value) {
                    Ok(Self(value))
                } else {
                    Err("value is zero".into())
                }
            }
        }

        impl<const MIN: $primitive, const MAX: $primitive> TryFrom<$primitive>
            for $bounded<MIN, MAX>
        {
            type Error = String;

            fn try_from(value: $primitive) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl<'de, const MIN: $primitive, const MAX: $primitive> Deserialize<'de>
            for $bounded<MIN, MAX>
        {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let value = Deserialize::deserialize(deserializer)?;
                Self::new(value).map_err(D::Error::custom)
            }
        }

        impl<const MIN: $primitive, const MAX: $primitive> From<$bounded<MIN, MAX>> for $nz {
            fn from(value: $bounded<MIN, MAX>) -> Self {
                value.0
            }
        }

        impl<const MIN: $primitive, const MAX: $primitive> From<$bounded<MIN, MAX>> for $primitive {
            fn from(value: $bounded<MIN, MAX>) -> Self {
                value.0.into()
            }
        }
    };
}

uint!(u8, NonZeroU8, LimitedU8, LimitedNonZeroU8, BoundedU8);
uint!(u16, NonZeroU16, LimitedU16, LimitedNonZeroU16, BoundedU16);
uint!(u32, NonZeroU32, LimitedU32, LimitedNonZeroU32, BoundedU32);
uint!(u64, NonZeroU64, LimitedU64, LimitedNonZeroU64, BoundedU64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u8_min_max() {
        assert_eq!(Ok(LimitedU8::<10>::MIN), 0.try_into());
        assert_eq!(Ok(LimitedU8::<10>::MAX), 10.try_into());
        assert_eq!(Ok(LimitedNonZeroU8::<10>::MIN), 1.try_into());
        assert_eq!(Ok(LimitedNonZeroU8::<10>::MAX), 10.try_into());
        assert_eq!(Ok(BoundedU8::<7, 10>::MIN), 7.try_into());
        assert_eq!(Ok(BoundedU8::<7, 10>::MAX), 10.try_into());
    }
}
