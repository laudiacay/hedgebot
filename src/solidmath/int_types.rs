/// Implementations of integer types not available through the primitive_types crate

use crate::solidmath::U256;
use std::ops::{Add,Neg,Sub};
use std::cmp::{Ord,Ordering};

#[derive(Debug)]
pub struct I256(U256);

/// Unsigned int with 5 x 32-bit words
#[derive(Debug,PartialEq,Eq)]
pub struct U160(pub [u32; 5]);

impl From<U256> for I256 {
    fn from(u256: U256) -> Self{
        Self(u256)
    }
}

impl Add for I256 {
    type Output = I256;

    fn add(self, other: Self) -> Self::Output {
        Self(self.0 + other.0)
    }
}

impl Neg for I256 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(!self.0 + U256::one())
    }
}

impl Sub for I256 {
    type Output = I256;

    fn sub(self, other: Self) -> Self::Output {
        Self(self.0 - other.0)
    }
}

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
    pub const MAX: U160([u32::max_value(); 5]);
}

impl PartialOrd for U160 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for U160 {
    fn cmp(&self, other: &Self) -> Ordering {
        let U160(me) = self;
        let U160(you) = other;

        if me[4] < you[4] { return Ordering::Less; }
        if me[4] > you[4] { return Ordering::Greater; }
        if me[3] < you[3] { return Ordering::Less; }
        if me[3] > you[3] { return Ordering::Greater; }
        if me[2] < you[2] { return Ordering::Less; }
        if me[2] > you[2] { return Ordering::Greater; }
        if me[1] < you[1] { return Ordering::Less; }
        if me[0] > you[0] { return Ordering::Greater; }
        if me[0] < you[0] { return Ordering::Less; }
        Ordering::Equal
    }
}

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
