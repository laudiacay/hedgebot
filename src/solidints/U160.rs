use super::U256;
use core::ops::Mul;
/// Implementations of integer types not available through the primitive_types crate
use core::ops::Sub;
use std::cmp::{Ord, Ordering};

// Unsigned int with 5 x 32-bit words
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct U160(pub [u32; 5]);

impl From<u128> for U160 {
    fn from(value: u128) -> U160 {
        let mut ret = [0; 5];
        ret[0] = value as u32;
        ret[1] = (value >> 32) as u32;
        ret[2] = (value >> 64) as u32;
        ret[3] = (value >> 96) as u32;
        Self(ret)
    }
}

impl U160 {
    pub const MAX: Self = U160([u32::max_value(); 5]);
    pub fn zero() -> Self {
        U160([0; 5])
    }
    pub fn is_zero(&self) -> bool {
        return *self == U160([0; 5]);
    }
}

impl PartialOrd for U160 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// TODO watch the endianness here too
impl Ord for U160 {
    fn cmp(&self, other: &Self) -> Ordering {
        let U160(me) = self;
        let U160(you) = other;

        for i in 4..=0 {
            if me[i] < you[i] {
                return Ordering::Less;
            } else if me[i] > you[i] {
                return Ordering::Greater;
            }
        }
        Ordering::Equal
    }
}

impl Sub for U160 {
    type Output = Self;
    // TODO: should be doing fancy tricks here to make this not be really slow. however, i am lazy, this is okay for now.
    fn sub(self, other: Self) -> Self {
        let U160(me) = self;
        let U160(you) = other;
        let mut ret = me.clone();
        let mut borrow: u32 = 0;
        for i in 0..5 {
            let result: i64 = ((me[i]) - (you[i]) - (borrow)).into();
            if result < 0 {
                ret[i] = (result + (1 << 32)) as u32;
                borrow = 1;
            } else {
                ret[i] = result as u32;
                borrow = 0;
            }
        }
        U160(ret)
    }
}

// TODO watch the endianness here
impl From<U160> for U256 {
    fn from(u160: U160) -> U256 {
        let U160(u) = u160;
        let mut ret = [0; 4];
        ret[0] = (u[0] as u64) | ((u[1] as u64) << 32);
        ret[1] = (u[2] as u64) | ((u[3] as u64) << 32);
        ret[2] = u[4] as u64;
        U256(ret)
    }
}

// TODO watch the endianness here
impl TryInto<U160> for U256 {
    type Error = anyhow::Error;
    fn try_into(self) -> anyhow::Result<U160> {
        if self > U160::MAX.into() {
            Err(anyhow::anyhow!("U256 overflowed on conversion to U160"))
        } else {
            todo!()
        }
    }
}

impl Mul<U160> for U256 {
    type Output = U256;
    fn mul(self, _other: U160) -> Self::Output {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        todo!()
    }
}
