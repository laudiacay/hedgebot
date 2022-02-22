use crate::solidints::U256;
use crate::solidints::{I256::I256, U160::U160};
// FIXME why the bad paths
use anyhow::{ensure, Result};
use std::ops::Add;

use super::fixed_point;
use super::full_math;

/// @notice Gets the next sqrt price given a delta of token0
/// @dev Always rounds up, because in the exact output case (increasing price) we need to move the price at least
/// far enough to get the desired output amount, and in the exact input case (decreasing price) we need to move the
/// price less in order to not send too much output.
/// The most precise formula for this is liquidity * sqrt_px96 / (liquidity +- amount * sqrt_px96),
/// if this is impossible because of overflow, we calculate liquidity / (liquidity / sqrt_px96 +- amount).
/// @param sqrt_px96 The starting price, i.e. before accounting for the token0 delta
/// @param liquidity The amount of usable liquidity
/// @param amount How much of token0 to add or remove from virtual reserves
/// @param add Whether to add or remove the amount of token0
/// @return The price after adding or removing amount, depending on add
pub fn get_next_sqrt_price_from_amount0_rounding_up(
    sqrt_px96: U160,
    liquidity: u128,
    amount: U256,
    add: bool,
) -> Result<U160> {
    // we short circuit amount == 0 because the result is otherwise not guaranteed to equal the input price
    if amount == U256::zero() {
        return Ok(sqrt_px96);
    };
    let numerator1: U256 = U256::try_from(liquidity)? << fixed_point::FP96_RESOLUTION;

    if add {
        let product = amount * sqrt_px96;
        if product / amount == sqrt_px96.into() {
            let denominator: U256 = numerator1 + product;
            if denominator >= numerator1 {
                // always fits in 160 bits
                return full_math::mul_div_rounding_up(numerator1, sqrt_px96.into(), denominator)?
                    .try_into();
            }
        }

        return (full_math::unsafe_div_rounding_up(numerator1, (numerator1 / sqrt_px96).add(amount)))?
            .try_into();
    } else {
        // if the product overflows, we know the denominator underflows
        // in addition, we must check that the denominator does not underflow
        let product: U256 = amount * sqrt_px96;
        ensure!(
            product / amount == sqrt_px96.into() && numerator1 > product,
            "todo better error message fdjskla"
        );
        let denominator: U256 = numerator1 - product;
        return full_math::mul_div_rounding_up(numerator1, sqrt_px96.into(), denominator)?.try_into();
    }
}

/// @notice Gets the next sqrt price given a delta of token1
/// @dev Always rounds down, because in the exact output case (decreasing price) we need to move the price at least
/// far enough to get the desired output amount, and in the exact input case (increasing price) we need to move the
/// price less in order to not send too much output.
/// The formula we compute is within <1 wei of the lossless version: sqrt_px96 +- amount / liquidity
/// @param sqrt_px96 The starting price, i.e., before accounting for the token1 delta
/// @param liquidity The amount of usable liquidity
/// @param amount How much of token1 to add, or remove, from virtual reserves
/// @param add Whether to add, or remove, the amount of token1
/// @return The price after adding or removing `amount`
fn get_next_sqrt_price_from_amount1_rounding_down(
    sqrt_px96: U160,
    liquidity: u128,
    amount: U256,
    add: bool,
) -> Result<U160> {
    // if we're adding (subtracting), rounding down requires rounding the quotient down (up)
    // in both cases, avoid a mulDiv for most inputs
    if add {
        let quotient: U256 = if amount <= U160::MAX.into() {
            (amount << fixed_point::FP96_RESOLUTION) / liquidity
        } else {
            full_math::muldiv(amount, *fixed_point::Q96, liquidity.into())?
        };

        (U256::from(sqrt_px96) + quotient).try_into()
    } else {
        let quotient: U256 = if amount <= U160::MAX.into() {
            full_math::unsafe_div_rounding_up(
                amount << fixed_point::FP96_RESOLUTION,
                liquidity.into(),
            )?
        } else {
            full_math::mul_div_rounding_up(amount, *fixed_point::Q96, liquidity.into())?
        };

        ensure!(U256::from(sqrt_px96) > quotient, "fdsajkfldsaw TODO");
        // always fits 160 bits
        Ok(sqrt_px96 - quotient.try_into()?)
    }
}

/// @notice Gets the next sqrt price given an input amount of token0 or token1
/// @dev Throws if price or liquidity are 0, or if the next price is out of bounds
/// @param sqrt_px96 The starting price, i.e., before accounting for the input amount
/// @param liquidity The amount of usable liquidity
/// @param amount_in How much of token0, or token1, is being swapped in
/// @param zero_for_one Whether the amount in is token0 or token1
/// @return sqrtQX96 The price after adding the input amount to token0 or token1
pub fn get_next_sqrt_price_from_input(
    sqrt_px96: U160,
    liquidity: u128,
    amount_in: U256,
    zero_for_one: bool,
) -> Result<U160> {
    ensure!(sqrt_px96 > U160::zero(), "zero price");
    ensure!(liquidity > 0, "zero liquidity");

    // round to make sure that we don't pass the target price
    if zero_for_one {
        get_next_sqrt_price_from_amount0_rounding_up(sqrt_px96, liquidity, amount_in, true)
    } else {
        get_next_sqrt_price_from_amount1_rounding_down(sqrt_px96, liquidity, amount_in, true)
    }
}

/// @notice Gets the next sqrt price given an output amount of token0 or token1
/// @dev Throws if price or liquidity are 0 or the next price is out of bounds
/// @param sqrt_px96 The starting price before accounting for the output amount
/// @param liquidity The amount of usable liquidity
/// @param amount_out How much of token0, or token1, is being swapped out
/// @param zero_for_one Whether the amount out is token0 or token1
/// @return sqrtQX96 The price after removing the output amount of token0 or token1
pub fn get_next_sqrt_price_from_output(
    sqrt_px96: U160,
    liquidity: u128,
    amount_out: U256,
    zero_for_one: bool,
) -> Result<U160> {
    ensure!(sqrt_px96 > U160::zero(), "zero price");
    ensure!(liquidity > 0, "zero liquidity");

    // round to make sure that we pass the target price
    if zero_for_one {
        get_next_sqrt_price_from_amount1_rounding_down(sqrt_px96, liquidity, amount_out, false)
    } else {
        get_next_sqrt_price_from_amount0_rounding_up(sqrt_px96, liquidity, amount_out, false)
    }
}

/// @notice Gets the amount0 delta between two prices
/// @dev Calculates liquidity / sqrt(lower) - liquidity / sqrt(upper),
/// i.e. liquidity * (sqrt(upper) - sqrt(lower)) / (sqrt(upper) * sqrt(lower))
/// @param sqrt_ratio_ax96 A sqrt price
/// @param sqrt_ratio_bx96 Another sqrt price
/// @param liquidity The amount of usable liquidity
/// @param round_up Whether to round the amount up or down
/// @return amount0 Amount of token0 required to cover a position of size liquidity between the two passed prices
pub fn get_amount0_delta_helper(
    sqrt_ratio_ax96: U160,
    sqrt_ratio_bx96: U160,
    liquidity: u128,
    round_up: bool,
) -> Result<U256> {
    let (sqrt_ratio_ax96, sqrt_ratio_bx96) = if sqrt_ratio_ax96 > sqrt_ratio_bx96 {
        (sqrt_ratio_bx96, sqrt_ratio_ax96)
    } else {
        (sqrt_ratio_ax96, sqrt_ratio_bx96)
    };

    let numerator1: U256 = (liquidity << fixed_point::FP96_RESOLUTION).into();
    let numerator2: U256 = (sqrt_ratio_bx96 - sqrt_ratio_ax96).into();

    ensure!(sqrt_ratio_ax96 > U160::zero(), "price zero");

    return if round_up {
        full_math::unsafe_div_rounding_up(
            full_math::mul_div_rounding_up(numerator1, numerator2, sqrt_ratio_bx96.into())?,
            sqrt_ratio_ax96.into(),
        )
    } else {
        Ok(full_math::muldiv(numerator1, numerator2, sqrt_ratio_bx96.into())? / sqrt_ratio_ax96)
    };
}

/// @notice Gets the amount1 delta between two prices
/// @dev Calculates liquidity * (sqrt(upper) - sqrt(lower))
/// @param sqrt_ratio_ax96 A sqrt price
/// @param sqrt_ratio_bx96 Another sqrt price
/// @param liquidity The amount of usable liquidity
/// @param round_up Whether to round the amount up, or down
/// @return amount1 Amount of token1 required to cover a position of size liquidity between the two passed prices
pub fn get_amount1_delta_helper(
    sqrt_ratio_ax96: U160,
    sqrt_ratio_bx96: U160,
    liquidity: u128,
    round_up: bool,
) -> Result<U256> {
    let (sqrt_ratio_ax96, sqrt_ratio_bx96) = if sqrt_ratio_ax96 > sqrt_ratio_bx96 {
        (sqrt_ratio_bx96, sqrt_ratio_ax96)
    } else {
        (sqrt_ratio_ax96, sqrt_ratio_bx96)
    };

    if round_up {
        full_math::mul_div_rounding_up(
            liquidity.into(),
            (sqrt_ratio_bx96 - sqrt_ratio_ax96).into(),
            *fixed_point::Q96,
        )
    } else {
        full_math::muldiv(
            liquidity.into(),
            (sqrt_ratio_bx96 - sqrt_ratio_ax96).into(),
            *fixed_point::Q96,
        )
    }
}

/// @notice Helper that gets signed token0 delta
/// @param sqrt_ratio_ax96 A sqrt price
/// @param sqrt_ratio_bx96 Another sqrt price
/// @param liquidity The change in liquidity for which to compute the amount0 delta
/// @return amount0 Amount of token0 corresponding to the passed liquidityDelta between the two prices
pub fn get_amount0_delta(sqrt_ratio_ax96: U160, sqrt_ratio_bx96: U160, liquidity: i128) -> Result<I256> {
    if liquidity < 0 {
        Ok(
            -(get_amount0_delta_helper(sqrt_ratio_ax96, sqrt_ratio_bx96, -liquidity as u128, false)?
                .try_into()?),
        )
    } else {
        get_amount0_delta_helper(sqrt_ratio_ax96, sqrt_ratio_bx96, liquidity as u128, true)?.try_into()
    }
}

/// @notice Helper that gets signed token1 delta
/// @param sqrt_ratio_ax96 A sqrt price
/// @param sqrt_ratio_bx96 Another sqrt price
/// @param liquidity The change in liquidity for which to compute the amount1 delta
/// @return amount1 Amount of token1 corresponding to the passed liquidityDelta between the two prices
pub fn get_amount1_delta(sqrt_ratio_ax96: U160, sqrt_ratio_bx96: U160, liquidity: i128) -> Result<I256> {
    if liquidity < 0 {
        Ok(
            -(get_amount1_delta_helper(sqrt_ratio_ax96, sqrt_ratio_bx96, -liquidity as u128, false)?
                .try_into()?),
        )
    } else {
        get_amount1_delta_helper(sqrt_ratio_ax96, sqrt_ratio_bx96, liquidity as u128, true)?.try_into()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        unimplemented!()
    }
}
