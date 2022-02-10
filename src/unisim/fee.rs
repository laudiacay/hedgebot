use num_rational::BigRational;
use std::ops::{Add, Sub};

#[derive(Debug, Clone, Copy)]
pub struct Fee {
    pub(crate) token_0: BigRational,
    pub(crate) token_1: BigRational,
}
impl Fee {
    pub(crate) fn new(token_0: BigRational, token_1: BigRational) -> Self {
        Self { token_0, token_1 }
    }
    pub(crate) fn zero() -> Self {
        Self {
            token_0: BigRational::zero(),
            token_1: BigRational::zero(),
        }
    }
}

impl Add for Fee {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            token_0: self.token_0 + other.token_0,
            token_1: self.token_1 + other.token_1,
        }
    }
}

impl Sub for Fee {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            token_0: self.token_0 - other.token_0,
            token_1: self.token_1 - other.token_1,
        }
    }
}
