use crate::solidints::I256::I256;
use crate::solidints::U160::U160;
use crate::solidints::U256;
use crate::solidmath::{full_math, sqrt_price_math};
use anyhow::Result;

/// @notice Computes the result of swapping some amount in, or amount out, given the parameters of the swap
/// @dev The fee, plus the amount in, will never exceed the amount remaining if the swap's `amountSpecified` is positive
/// @param sqrtRatioCurrentX96 The current sqrt price of the pool
/// @param sqrtRatioTargetX96 The price that cannot be exceeded, from which the direction of the swap is inferred
/// @param liquidity The usable liquidity
/// @param amountRemaining How much input or output amount is remaining to be swapped in/out
/// @param feePips The fee taken from the input amount, expressed in hundredths of a bip
/// @return sqrtRatioNextX96 The price after swapping the amount in/out, not to exceed the price target
/// @return amount_in The amount to be swapped in, of either token0 or token1, based on the direction of the swap
/// @return amount_out The amount to be received, of either token0 or token1, based on the direction of the swap
/// @return feeAmount The amount of input that will be taken as a fee

struct ComputeSwapStepReturn {
    sqrtRatioNextX96: U160,
    amount_in: U256,
    amount_out: U256,
    feeAmount: U256,
}

fn computeSwapStep(
    sqrtRatioCurrentX96: U160,
    sqrtRatioTargetX96: U160,
    liquidity: u128,
    amountRemaining: I256,
    feePips: u32,
) -> Result<ComputeSwapStepReturn> {
    let zero_for_one = sqrtRatioCurrentX96 >= sqrtRatioTargetX96;
    let exactIn = amountRemaining >= I256::zero();

    let (amountRemainingLessFee, amount_in, amount_out, sqrtRatioNextX96) = if exactIn {
        let amountRemainingLessFee =
            full_math::muldiv(amountRemaining.try_into()?, (1000000 - feePips).into(), 1000000 as.into())?;
        let amount_in = if zero_for_one {
            sqrt_price_math::get_amount0_delta_helper(
                sqrtRatioTargetX96,
                sqrtRatioCurrentX96,
                liquidity,
                true,
            )?
        } else {
            sqrt_price_math::get_amount1_delta_helper(
                sqrtRatioCurrentX96,
                sqrtRatioTargetX96,
                liquidity,
                true,
            )?
        };
        let sqrtRatioNextX96 = if amountRemainingLessFee >= amount_in {
            sqrtRatioTargetX96
        } else {
            sqrt_price_math::getNextSqrtPriceFromInput(
                sqrtRatioCurrentX96,
                liquidity,
                amountRemainingLessFee,
                zero_for_one,
            )?
        };
        (
            Some(amountRemainingLessFee),
            Some(amount_in),
            None,
            sqrtRatioNextX96,
        );
    } else {
        let amount_out = if zero_for_one {
            sqrt_price_math::get_amount1_delta_helper(
                sqrtRatioTargetX96,
                sqrtRatioCurrentX96,
                liquidity,
                false,
            )
        } else {
            sqrt_price_math::get_amount0_delta_helper(
                sqrtRatioCurrentX96,
                sqrtRatioTargetX96,
                liquidity,
                false,
            )
        };
        let sqrtRatioNextX96 = if -amountRemaining.try_into()? >= amount_out {
            sqrtRatioTargetX96
        } else {
            sqrt_price_math::getNextSqrtPriceFromOutput(
                sqrtRatioCurrentX96,
                liquidity,
                (-amountRemaining).try_into()?,
                zero_for_one,
            )?
        };
        (None, None, Some(amount_out), sqrtRatioNextX96)
    };

    let max = sqrtRatioTargetX96 == sqrtRatioNextX96;

    // get the input/output amounts
    let (amount_in, amount_out) = if zero_for_one {
        let amount_in = if max && exactIn {
            amount_in
        } else {
            sqrt_price_math::get_amount0_delta_helper(
                sqrtRatioNextX96,
                sqrtRatioCurrentX96,
                liquidity,
                true,
            )?
        };
        let amount_out = if max && !exactIn {
            amount_out
        } else {
            sqrt_price_math::get_amount1_delta_helper(
                sqrtRatioNextX96,
                sqrtRatioCurrentX96,
                liquidity,
                false,
            )?
        };
        (amount_in, amount_out)
    } else {
        let amount_in = if max && exactIn {
            amount_in
        } else {
            sqrt_price_math::get_amount1_delta_helper(
                sqrtRatioCurrentX96,
                sqrtRatioNextX96,
                liquidity,
                true,
            )?
        };
        amount_out = if max && !exactIn {
            amount_out
        } else {
            sqrt_price_math::get_amount0_delta_helper(
                sqrtRatioCurrentX96,
                sqrtRatioNextX96,
                liquidity,
                false,
            )?
        };
        (amount_in, amount_out)
    };

    // cap the output amount to not exceed the remaining output amount
    if !exactIn && amount_out > -amountRemaining.into() {
        amount_out = -amountRemaining.into();
    }

    let feeAmount = if exactIn && sqrtRatioNextX96 != sqrtRatioTargetX96 {
        // we didn't reach the target, so take the remainder of the maximum input as fee
        amountRemaining.into() - amount_in
    } else {
        full_math::mul_div_rounding_up(amount_in, feePips.into(), (1000000 - feePips).into())
    };

    Ok(ComputeSwapStepReturn {
        sqrtRatioNextX96,
        amount_in,
        amount_out,
        feeAmount,
    })
}
