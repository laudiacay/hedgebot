use std::ops::{Add, Sub};

#[derive(Debug, Clone, Copy)]
pub struct Fee {
    token_0: f64, // technically wrong datatype
    token_1: f64, // technically wrong datatype
}
impl Fee {
    pub(crate) fn new(token_0: f64, token_1: f64) -> Self {
        Self { token_0, token_1 }
    }
    pub(crate) fn zero() -> Self {
        Self {
            token_0: 0.0,
            token_1: 0.0,
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
