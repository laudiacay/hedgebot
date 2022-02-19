use super::U256;
use anyhow::{anyhow, Result};
use std::cmp::{Ord, Ordering};
use std::ops::{Add, Neg, Sub};

#[derive(Debug, PartialEq, Eq)]
pub struct I256(U256);

lazy_static! {
    pub static ref mask: U256 = U256::one() << (255.into(): U256);
}

impl PartialOrd for I256 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for I256 {
    fn cmp(&self, other: &Self) -> Ordering {
        let I256(self_as_u256) = self;
        let I256(other_as_u256) = other;

        let self_positive: bool = (*self_as_u256 & *mask) == U256::zero();
        let other_positive: bool = (*other_as_u256 & *mask) == U256::zero();

        let positive_comparison = match (self_positive, other_positive) {
            (true, false) => return Ordering::Greater,
            (false, true) => return Ordering::Less,
            (a, _) => a,
        };

        let U256(self_u64s) = self_as_u256;
        let U256(other_u64s) = other_as_u256;

        for i in 3..=0 {
            if self_u64s[i] < other_u64s[i] {
                return if positive_comparison {
                    Ordering::Less
                } else {
                    Ordering::Greater
                };
            } else if self_u64s[i] > other_u64s[i] {
                return if positive_comparison {
                    Ordering::Greater
                } else {
                    Ordering::Less
                };
            }
        }

        Ordering::Equal
    }
}
impl Neg for I256 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(!self.0 + U256::one())
    }
}
impl Add for I256 {
    type Output = I256;

    fn add(self, other: Self) -> Self::Output {
        Self(self.0 + other.0)
    }
}

impl Sub for I256 {
    type Output = I256;

    fn sub(self, other: Self) -> Self::Output {
        Self(self.0 - other.0)
    }
}

impl TryFrom<U256> for I256 {
    type Error = anyhow::Error;
    fn try_from(u256: U256) -> Result<I256> {
        if (u256 & *mask) != U256::zero() {
            Err(anyhow!("U256 overflowed on conversion to I256"))
        } else {
            Ok(I256(u256))
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        unimplemented!()
    }
}
