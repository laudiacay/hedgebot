use crate::solidmath::U256;
use anyhow::{anyhow, ensure, Result};
use std::ops::Add;

use super::fixed_point;
use super::full_math;

pub type U160 = U256; // FIXME this is wrong. need a real int160 type lol

/// @notice Gets the next sqrt price given a delta of token0
/// @dev Always rounds up, because in the exact output case (increasing price) we need to move the price at least
/// far enough to get the desired output amount, and in the exact input case (decreasing price) we need to move the
/// price less in order to not send too much output.
/// The most precise formula for this is liquidity * sqrtPX96 / (liquidity +- amount * sqrtPX96),
/// if this is impossible because of overflow, we calculate liquidity / (liquidity / sqrtPX96 +- amount).
/// @param sqrtPX96 The starting price, i.e. before accounting for the token0 delta
/// @param liquidity The amount of usable liquidity
/// @param amount How much of token0 to add or remove from virtual reserves
/// @param add Whether to add or remove the amount of token0
/// @return The price after adding or removing amount, depending on add
pub fn getNextSqrtPriceFromAmount0RoundingUp(
    sqrtPX96: U160,
    liquidity: u128,
    amount: U256,
    add: bool,
) -> Result<U160> {
    // we short circuit amount == 0 because the result is otherwise not guaranteed to equal the input price
    if amount == U256::zero() {
        return Ok(sqrtPX96);
    };
    let numerator1: U256 = U256::try_from(liquidity)? << fixed_point::FP96_RESOLUTION;

    if add {
        let product = amount * sqrtPX96;
        if product / amount == sqrtPX96 {
            let denominator: U256 = numerator1 + product;
            if denominator >= numerator1 {
                // always fits in 160 bits
                return full_math::mulDivRoundingUp(numerator1, sqrtPX96, denominator);
            }
        }

        return (full_math::unsafeDivRoundingUp(numerator1, (numerator1 / sqrtPX96).add(amount)))
            .into();
    } else {
        // if the product overflows, we know the denominator underflows
        // in addition, we must check that the denominator does not underflow
        let product: U256 = amount * sqrtPX96;
        ensure!(
            product / amount == sqrtPX96 && numerator1 > product,
            "todo better error message fdjskla"
        );
        let denominator: U256 = numerator1 - product;
        return full_math::mulDivRoundingUp(numerator1, sqrtPX96, denominator);
    }
}

/// @notice Gets the next sqrt price given a delta of token1
/// @dev Always rounds down, because in the exact output case (decreasing price) we need to move the price at least
/// far enough to get the desired output amount, and in the exact input case (increasing price) we need to move the
/// price less in order to not send too much output.
/// The formula we compute is within <1 wei of the lossless version: sqrtPX96 +- amount / liquidity
/// @param sqrtPX96 The starting price, i.e., before accounting for the token1 delta
/// @param liquidity The amount of usable liquidity
/// @param amount How much of token1 to add, or remove, from virtual reserves
/// @param add Whether to add, or remove, the amount of token1
/// @return The price after adding or removing `amount`
fn getNextSqrtPriceFromAmount1RoundingDown(
    sqrtPX96: U160,
    liquidity: u128,
    amount: U256,
    add: bool,
) -> Result<U160> {
    // if we're adding (subtracting), rounding down requires rounding the quotient down (up)
    // in both cases, avoid a mulDiv for most inputs
    if add {
        let quotient: U256 = if amount <= U160::MAX {
            (amount << fixed_point::FP96_RESOLUTION) / liquidity
        } else {
            full_math::muldiv(amount, *fixed_point::q96, liquidity.into())?
        };

        Ok((sqrtPX96 as U256).add(quotient))
    } else {
        let quotient: U256 = if amount <= U160::MAX {
            full_math::unsafeDivRoundingUp(
                amount << fixed_point::FP96_RESOLUTION,
                liquidity.into(),
            )?
        } else {
            full_math::mulDivRoundingUp(amount, *fixed_point::q96, liquidity.into())?
        };

        ensure!(sqrtPX96 > quotient, "fdsajkfldsaw");
        // always fits 160 bits
        Ok(sqrtPX96 - quotient)
    }
}

/// @notice Gets the next sqrt price given an input amount of token0 or token1
/// @dev Throws if price or liquidity are 0, or if the next price is out of bounds
/// @param sqrtPX96 The starting price, i.e., before accounting for the input amount
/// @param liquidity The amount of usable liquidity
/// @param amountIn How much of token0, or token1, is being swapped in
/// @param zeroForOne Whether the amount in is token0 or token1
/// @return sqrtQX96 The price after adding the input amount to token0 or token1
fn getNextSqrtPriceFromInput(
    sqrtPX96: U160,
    liquidity: u128,
    amountIn: U256,
    zeroForOne: bool,
) -> Result<U160> {
    ensure!(sqrtPX96 > U256::zero(), "zero price");
    ensure!(liquidity > 0, "zero liquidity");

    // round to make sure that we don't pass the target price
    if zeroForOne {
        getNextSqrtPriceFromAmount0RoundingUp(sqrtPX96, liquidity, amountIn, true)
    } else {
        getNextSqrtPriceFromAmount1RoundingDown(sqrtPX96, liquidity, amountIn, true)
    }
}

/// @notice Gets the next sqrt price given an output amount of token0 or token1
/// @dev Throws if price or liquidity are 0 or the next price is out of bounds
/// @param sqrtPX96 The starting price before accounting for the output amount
/// @param liquidity The amount of usable liquidity
/// @param amountOut How much of token0, or token1, is being swapped out
/// @param zeroForOne Whether the amount out is token0 or token1
/// @return sqrtQX96 The price after removing the output amount of token0 or token1
fn getNextSqrtPriceFromOutput(
    sqrtPX96: U160,
    liquidity: u128,
    amountOut: U256,
    zeroForOne: bool,
) -> Result<U160> {
    ensure!(sqrtPX96 > U256::zero(), "zero price");
    ensure!(liquidity > 0, "zero liquidity");

    // round to make sure that we pass the target price
    if zeroForOne {
        getNextSqrtPriceFromAmount1RoundingDown(sqrtPX96, liquidity, amountOut, false)
    } else {
        getNextSqrtPriceFromAmount0RoundingUp(sqrtPX96, liquidity, amountOut, false)
    }
}

/// @notice Gets the amount0 delta between two prices
/// @dev Calculates liquidity / sqrt(lower) - liquidity / sqrt(upper),
/// i.e. liquidity * (sqrt(upper) - sqrt(lower)) / (sqrt(upper) * sqrt(lower))
/// @param sqrtRatioAX96 A sqrt price
/// @param sqrtRatioBX96 Another sqrt price
/// @param liquidity The amount of usable liquidity
/// @param roundUp Whether to round the amount up or down
/// @return amount0 Amount of token0 required to cover a position of size liquidity between the two passed prices
fn getAmount0DeltaHelper(
    sqrtRatioAX96: U160,
    sqrtRatioBX96: U160,
    liquidity: u128,
    roundUp: bool,
) -> Result<U256> {
    let (sqrtRatioAX96, sqrtRatioBX96) = if sqrtRatioAX96 > sqrtRatioBX96 {
        (sqrtRatioBX96, sqrtRatioAX96)
    } else {
        (sqrtRatioAX96, sqrtRatioBX96)
    };

    let numerator1: U256 = (liquidity << fixed_point::FP96_RESOLUTION).into();
    let numerator2: U256 = sqrtRatioBX96 - sqrtRatioAX96;

    ensure!(sqrtRatioAX96 > U256::zero(), "price zero");

    return if roundUp {
        full_math::unsafeDivRoundingUp(
            full_math::mulDivRoundingUp(numerator1, numerator2, sqrtRatioBX96)?,
            sqrtRatioAX96,
        )
    } else {
        Ok(full_math::muldiv(numerator1, numerator2, sqrtRatioBX96)? / sqrtRatioAX96)
    };
}

/// @notice Gets the amount1 delta between two prices
/// @dev Calculates liquidity * (sqrt(upper) - sqrt(lower))
/// @param sqrtRatioAX96 A sqrt price
/// @param sqrtRatioBX96 Another sqrt price
/// @param liquidity The amount of usable liquidity
/// @param roundUp Whether to round the amount up, or down
/// @return amount1 Amount of token1 required to cover a position of size liquidity between the two passed prices
fn getAmount1DeltaHelper(
    sqrtRatioAX96: U160,
    sqrtRatioBX96: U160,
    liquidity: u128,
    roundUp: bool,
) -> Result<U256> {
    let (sqrtRatioAX96, sqrtRatioBX96) = if sqrtRatioAX96 > sqrtRatioBX96 {
        (sqrtRatioBX96, sqrtRatioAX96)
    } else {
        (sqrtRatioAX96, sqrtRatioBX96)
    };

    if roundUp {
        full_math::mulDivRoundingUp(
            liquidity.into(),
            sqrtRatioBX96 - sqrtRatioAX96,
            *fixed_point::q96,
        )
    } else {
        full_math::muldiv(
            liquidity.into(),
            sqrtRatioBX96 - sqrtRatioAX96,
            *fixed_point::q96,
        )
    }
}

/// @notice Helper that gets signed token0 delta
/// @param sqrtRatioAX96 A sqrt price
/// @param sqrtRatioBX96 Another sqrt price
/// @param liquidity The change in liquidity for which to compute the amount0 delta
/// @return amount0 Amount of token0 corresponding to the passed liquidityDelta between the two prices
fn getAmount0Delta(sqrtRatioAX96: U160, sqrtRatioBX96: U160, liquidity: i128) -> I256 {
    if liquidity < 0 {
        -getAmount0DeltaHelper(sqrtRatioAX96, sqrtRatioBX96, uint128(-liquidity), false).toInt256()
    } else {
        getAmount0DeltaHelper(sqrtRatioAX96, sqrtRatioBX96, uint128(liquidity), true).toInt256()
    }
}

/// @notice Helper that gets signed token1 delta
/// @param sqrtRatioAX96 A sqrt price
/// @param sqrtRatioBX96 Another sqrt price
/// @param liquidity The change in liquidity for which to compute the amount1 delta
/// @return amount1 Amount of token1 corresponding to the passed liquidityDelta between the two prices
fn getAmount1Delta(sqrtRatioAX96: U160, sqrtRatioBX96: U160, liquidity: i128) -> I256 {
    if liquidity < 0 {
        -getAmount1DeltaHelper(sqrtRatioAX96, sqrtRatioBX96, uint128(-liquidity), false).toInt256()
    } else {
        getAmount1DeltaHelper(sqrtRatioAX96, sqrtRatioBX96, uint128(liquidity), true).toInt256()
    }
}
