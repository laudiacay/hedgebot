// TODO: is not doing it their way a potential source of slowness?

use crate::solidmath::U256;
use anyhow::{anyhow, ensure, Result};

pub fn mulmod(a: U256, b: U256, modulus: U256) -> Result<U256> {
    let res = a.full_mul(b).checked_rem(modulus.into());
    match res {
        Some(u512) => U256::try_from(u512).map_err(|e| anyhow!("{:?}", e)),
        None => Err(anyhow!("modulus was zero.")),
    }
}

pub fn muldiv(a: U256, b: U256, div: U256) -> Result<U256> {
    let res = a.full_mul(b).checked_div(div.into());
    match res {
        Some(u512) => U256::try_from(u512).map_err(|e| anyhow!("{:?}", e)),
        None => Err(anyhow!("denominator was zero.")),
    }
}

pub fn mulDivRoundingUp(a: U256, b: U256, denominator: U256) -> Result<U256> {
    let result = muldiv(a, b, denominator)?;
    if mulmod(a, b, denominator)? > U256::zero() {
        ensure!(result < U256::MAX, "result overflowed");
        return Ok(result + 1 as i8);
    }
    Ok(result)
}

pub fn unsafeDivRoundingUp(x: U256, y: U256) -> Result<U256> {
    let remainder = match x.checked_rem(y) {
        Some(thing) => thing,
        None => Err(anyhow!("denominator was zero."))?,
    };
    let round = remainder > U256::zero();
    Ok(x / y + (if round { 1 } else { 0 } as i8))
}
