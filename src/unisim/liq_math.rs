use anyhow::{anyhow, Result};

/// Add a signed liquidity delta to liquidity and revert if it overflows or underflows
/// * x The liquidity before change
/// * y The delta by which liquidity should be changed
/// Returns the liquidity delta
pub fn addDelta(x: u128, y: i128) -> Result<u128> {
    if y < 0 {
        let z = x - (-y as u128);
        if !(z < x) {
            Err(anyhow!("LS"))
        } else {
            Ok(z)
        }
    } else {
        let z = x + (y as u128);
        if !(z >= x) {
            Err(anyhow!("LA"))
        } else {
            Ok(z)
        }
    }
}
