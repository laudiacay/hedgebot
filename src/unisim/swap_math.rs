use crate::solidmath::int_types::I256;
use crate::solidmath::int_types::U160;
use crate::solidmath::sqrt_price_math;
use crate::solidmath::U256;

struct ComputeSwapStepReturn {
    sqrt_ratio_next: U160,
    amount_in: U256,
    amount_out: U256,
    fee_amount: U256,
}
/// @notice Computes the result of swapping some amount in, or amount out, given the parameters of the swap
/// @dev The fee, plus the amount in, will never exceed the amount remaining if the swap's `amountSpecified` is positive
/// @param sqrt_ratio_current The current sqrt price of the pool
/// @param sqrt_ratio_target The price that cannot be exceeded, from which the direction of the swap is inferred
/// @param liquidity The usable liquidity
/// @param amountRemaining How much input or output amount is remaining to be swapped in/out
/// @param feePips The fee taken from the input amount, expressed in hundredths of a bip
/// @return sqrtRatioNextX96 The price after swapping the amount in/out, not to exceed the price target
/// @return amountIn The amount to be swapped in, of either token0 or token1, based on the direction of the swap
/// @return amountOut The amount to be received, of either token0 or token1, based on the direction of the swap
/// @return feeAmount The amount of input that will be taken as a fee
pub fn compute_swap_step(
    sqrt_ratio_current: U160,
    sqrt_ratio_target: U160,
    liquidity: u128,
    amountRemaining: I256,
    fee_pips: u32,
) -> ComputeSwapStepReturn {
    let zeroForOne = sqrt_ratio_current >= sqrt_ratio_target;
    let exactIn = amountRemaining >= I256::zero();

    if exactIn {
        let amountRemainingLessFee = amountRemaining * ((1e6 - fee_pips) as f64) / 1e6; // FIXME hmm, this math seems off
        let amountIn = if zeroForOne {
            sqrt_price_math::getAmount0DeltaHelper(
                sqrt_ratio_target,
                sqrt_ratio_current,
                liquidity,
                true,
            )
        } else {
            sqrt_price_math::getAmount1DeltaHelper(
                sqrt_ratio_current,
                sqrt_ratio_target,
                liquidity,
                true,
            )
        };
        let sqrtRatioNextX96 = if amountRemainingLessFee >= amountIn {
            sqrt_ratio_target
        } else {
            sqrt_price_math::getNextSqrtPriceFromInput(
                sqrt_ratio_current,
                liquidity,
                amountRemainingLessFee,
                zeroForOne,
            )
        };
    } else {
        amountOut = if zeroForOne {
            sqrt_price_math::getAmount1DeltaHelper(
                sqrt_ratio_target,
                sqrt_ratio_current,
                liquidity,
                false,
            )
        } else {
            sqrt_price_math::getAmount0DeltaHelper(
                sqrt_ratio_current,
                sqrt_ratio_target,
                liquidity,
                false,
            )
        };
        let sq = if -amountRemaining >= amountOut {
            sqrt_ratio_target
        } else {
            sqrt_price_math::getNextSqrtPriceFromOutput(
                sqrt_ratio_current,
                liquidity,
                uint256(-amountRemaining),
                zeroForOne,
            )
        };
    }

    let max = sqrt_ratio_target == sqrtRatioNextX96;

    // get the input/output amounts
    if zeroForOne {
        amountIn = if max && exactIn {
            amountIn
        } else {
            sqrt_price_math::getAmount0Delta(sqrtRatioNextX96, sqrt_ratio_current, liquidity, true)
        };
        amountOut = if max && !exactIn {
            amountOut
        } else {
            sqrt_price_math::getAmount1Delta(sqrtRatioNextX96, sqrt_ratio_current, liquidity, false)
        };
    } else {
        amountIn = if max && exactIn {
            amountIn
        } else {
            sqrt_price_math::getAmount1DeltaHelper(
                sqrt_ratio_current,
                sqrtRatioNextX96,
                liquidity,
                true,
            )
        };
        amountOut = if max && !exactIn {
            amountOut
        } else {
            sqrt_price_math::getAmount0DeltaHelper(
                sqrt_ratio_current,
                sqrtRatioNextX96,
                liquidity,
                false,
            )
        };
    }

    // cap the output amount to not exceed the remaining output amount
    if !exactIn && amountOut > uint256(-amountRemaining) {
        amountOut = uint256(-amountRemaining);
    }

    if exactIn && sqrtRatioNextX96 != sqrt_ratio_target {
        // we didn't reach the target, so take the remainder of the maximum input as fee
        feeAmount = uint256(amountRemaining) - amountIn;
    } else {
        feeAmount = FullMath.mulDivRoundingUp(amountIn, fee_pips, 1e6 - fee_pips);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        unimplemented!()
    }
}
