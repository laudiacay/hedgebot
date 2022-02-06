mod fee;
mod tick;

use crate::unisim::fee::Fee;
use anyhow::{anyhow, Result};
use std::collections::{HashMap, HashSet};
use tick::*;

type SqrtPriceUnits = f64; // FIXME: types

type Address = String; // lmao, for now

struct PositionID {
    address: String,
    lower_bound: Tick,
    upper_bound: Tick,
}

struct Position {
    liquidity: u128,
    fee_growth_inside: Fee,
}

struct UniV3Pool {
    liquidity: u128,
    sqrt_price: SqrtPriceUnits,
    tick: Tick,
    tick_spacing: u32,
    fee_growth_global: Fee,
    protocol_fees: Fee,
    gamma_percent_fee: f32,
    phi_percent_fee: f32,
    tick_chart: HashMap<Tick, TickData>,
    positions: HashMap<PositionID, Position>,
    my_positions: HashSet<PositionID>,
}

enum SwapEvent {
    SetPosition {
        position_id: PositionID,
        delta_liquidity: i128,
    },
    Swap0to1 {
        amount: u128,
    },
    Swap1to0 {
        amount: u128,
    },
}

impl UniV3PoolSimulator {
    pub fn new(
        start_price: SqrtPriceUnits,
        tick_spacing: u32,
        gamma_percent_fee: f32,
        phi_percent_fee: f32,
    ) -> Self {
        Self {
            liquidity: 0,
            sqrt_price: start_price.sqrt(),
            tick: tick_math::tick_from_price(start_price),
            tick_spacing,
            fee_growth_global: Fee::zero(),
            protocol_fees: Fee::zero(),
            gamma_percent_fee,
            phi_percent_fee,
            tick_chart: HashMap::new(),
            positions: HashMap::new(),
            my_positions: HashSet::new(),
        }
    }
    fn next_highest_tick(&self) -> Tick {
        return self.tick + self.tick_spacing;
    }
    fn next_lowest_tick(&self) -> Tick {
        return self.tick - self.tick_spacing;
    }
}
// TODO: document me!
impl UniV3PoolSimulator {
    pub fn set_position(&mut self, position_id: PositionID, state: Position, mine: bool) {}
    pub fn get_my_holdings_value(&self, in_token_1: bool) -> i256 {
        return 0;
    }
    pub fn swap(
        zero_for_one: bool,
        amount_specified: i256,
        sqrt_price_limit: SqrtPriceUnits,
    ) -> Result<(i256, i256)> {
        if amount_specified == 0 {
            Err(anyhow!("AS"))?
        };

        if zero_for_one {
            if !(sqrtPriceLimitX96 < slot0Start.sqrtPriceX96
                && sqrtPriceLimitX96 > TickMath.MIN_SQRT_RATIO)
            {
                Err(anyhow!("SPL"))?
            }
        } else {
            if !(sqrtPriceLimitX96 > slot0Start.sqrtPriceX96
                && sqrtPriceLimitX96 < TickMath.MAX_SQRT_RATIO)
            {
                Err(anyhow!("SPL"))?
            }
        };

        // SwapCache memory cache =
        //     SwapCache({
        //         liquidityStart: liquidity,
        //         blockTimestamp: _blockTimestamp(),
        //         feeProtocol: zeroForOne ? (slot0Start.feeProtocol % 16) : (slot0Start.feeProtocol >> 4),
        //         secondsPerLiquidityCumulativeX128: 0,
        //         tickCumulative: 0,
        //         computedLatestObservation: false
        //     });
        //
        // bool exactInput = amountSpecified > 0;
        //
        // SwapState memory state =
        //     SwapState({
        //         amountSpecifiedRemaining: amountSpecified,
        //         amountCalculated: 0,
        //         sqrtPriceX96: slot0Start.sqrtPriceX96,
        //         tick: slot0Start.tick,
        //         feeGrowthGlobalX128: zeroForOne ? feeGrowthGlobal0X128 : feeGrowthGlobal1X128,
        //         protocolFee: 0,
        //         liquidity: cache.liquidityStart
        //     });
        //
        // // continue swapping as long as we haven't used the entire input/output and haven't reached the price limit
        // while (state.amountSpecifiedRemaining != 0 && state.sqrtPriceX96 != sqrtPriceLimitX96) {
        //     StepComputations memory step;
        //
        //     step.sqrtPriceStartX96 = state.sqrtPriceX96;
        //
        //     (step.tickNext, step.initialized) = tickBitmap.nextInitializedTickWithinOneWord(
        //         state.tick,
        //         tickSpacing,
        //         zeroForOne
        //     );
        //
        //     // ensure that we do not overshoot the min/max tick, as the tick bitmap is not aware of these bounds
        //     if (step.tickNext < TickMath.MIN_TICK) {
        //         step.tickNext = TickMath.MIN_TICK;
        //     } else if (step.tickNext > TickMath.MAX_TICK) {
        //         step.tickNext = TickMath.MAX_TICK;
        //     }
        //
        //     // get the price for the next tick
        //     step.sqrtPriceNextX96 = TickMath.getSqrtRatioAtTick(step.tickNext);
        //
        //     // compute values to swap to the target tick, price limit, or point where input/output amount is exhausted
        //     (state.sqrtPriceX96, step.amountIn, step.amountOut, step.feeAmount) = SwapMath.computeSwapStep(
        //         state.sqrtPriceX96,
        //         (zeroForOne ? step.sqrtPriceNextX96 < sqrtPriceLimitX96 : step.sqrtPriceNextX96 > sqrtPriceLimitX96)
        //         ? sqrtPriceLimitX96
        //         : step.sqrtPriceNextX96,
        //     state.liquidity,
        //     state.amountSpecifiedRemaining,
        //     fee
        //     );
        //
        //     if (exactInput) {
        //         state.amountSpecifiedRemaining -= (step.amountIn + step.feeAmount).toInt256();
        //         state.amountCalculated = state.amountCalculated.sub(step.amountOut.toInt256());
        //     } else {
        //         state.amountSpecifiedRemaining += step.amountOut.toInt256();
        //         state.amountCalculated = state.amountCalculated.add((step.amountIn + step.feeAmount).toInt256());
        //     }
        //
        //     // if the protocol fee is on, calculate how much is owed, decrement feeAmount, and increment protocolFee
        //     if (cache.feeProtocol > 0) {
        //         uint256 delta = step.feeAmount / cache.feeProtocol;
        //         step.feeAmount -= delta;
        //         state.protocolFee += uint128(delta);
        //     }
        //
        //     // update global fee tracker
        //     if (state.liquidity > 0)
        //     state.feeGrowthGlobalX128 += FullMath.mulDiv(step.feeAmount, FixedPoint128.Q128, state.liquidity);
        //
        //     // shift tick if we reached the next price
        //     if (state.sqrtPriceX96 == step.sqrtPriceNextX96) {
        //         // if the tick is initialized, run the tick transition
        //         if (step.initialized) {
        //             // check for the placeholder value, which we replace with the actual value the first time the swap
        //             // crosses an initialized tick
        //             if (!cache.computedLatestObservation) {
        //                 (cache.tickCumulative, cache.secondsPerLiquidityCumulativeX128) = observations.observeSingle(
        //                     cache.blockTimestamp,
        //                     0,
        //                     slot0Start.tick,
        //                     slot0Start.observationIndex,
        //                     cache.liquidityStart,
        //                     slot0Start.observationCardinality
        //                 );
        //                 cache.computedLatestObservation = true;
        //             }
        //             int128 liquidityNet =
        //                 ticks.cross(
        //                     step.tickNext,
        //                     (zeroForOne ? state.feeGrowthGlobalX128 : feeGrowthGlobal0X128),
        //             (zeroForOne ? feeGrowthGlobal1X128 : state.feeGrowthGlobalX128),
        //             cache.secondsPerLiquidityCumulativeX128,
        //             cache.tickCumulative,
        //             cache.blockTimestamp
        //             );
        //             // if we're moving leftward, we interpret liquidityNet as the opposite sign
        //             // safe because liquidityNet cannot be type(int128).min
        //             if (zeroForOne) liquidityNet = -liquidityNet;
        //
        //             state.liquidity = LiquidityMath.addDelta(state.liquidity, liquidityNet);
        //         }
        //
        //         state.tick = zeroForOne ? step.tickNext - 1 : step.tickNext;
        //     } else if (state.sqrtPriceX96 != step.sqrtPriceStartX96) {
        //         // recompute unless we're on a lower tick boundary (i.e. already transitioned ticks), and haven't moved
        //         state.tick = TickMath.getTickAtSqrtRatio(state.sqrtPriceX96);
        //     }
        // }
        //
        // // update tick and write an oracle entry if the tick change
        // if (state.tick != slot0Start.tick) {
        //     (uint16 observationIndex, uint16 observationCardinality) =
        //     observations.write(
        //         slot0Start.observationIndex,
        //         cache.blockTimestamp,
        //         slot0Start.tick,
        //         cache.liquidityStart,
        //         slot0Start.observationCardinality,
        //         slot0Start.observationCardinalityNext
        //     );
        //     (slot0.sqrtPriceX96, slot0.tick, slot0.observationIndex, slot0.observationCardinality) = (
        //         state.sqrtPriceX96,
        //         state.tick,
        //         observationIndex,
        //         observationCardinality
        //     );
        // } else {
        //     // otherwise just update the price
        //     slot0.sqrtPriceX96 = state.sqrtPriceX96;
        // }
        //
        // // update liquidity if it changed
        // if (cache.liquidityStart != state.liquidity) liquidity = state.liquidity;
        //
        // // update fee growth global and, if necessary, protocol fees
        // // overflow is acceptable, protocol has to withdraw before it hits type(uint128).max fees
        // if (zeroForOne) {
        //     feeGrowthGlobal0X128 = state.feeGrowthGlobalX128;
        //     if (state.protocolFee > 0) protocolFees.token0 += state.protocolFee;
        // } else {
        //     feeGrowthGlobal1X128 = state.feeGrowthGlobalX128;
        //     if (state.protocolFee > 0) protocolFees.token1 += state.protocolFee;
        // }
        //
        // (amount0, amount1) = zeroForOne == exactInput
        //     ? (amountSpecified - state.amountSpecifiedRemaining, state.amountCalculated)
        //     : (state.amountCalculated, amountSpecified - state.amountSpecifiedRemaining);
        //
        // // do the transfers and collect payment
        // if (zeroForOne) {
        //     if (amount1 < 0) TransferHelper.safeTransfer(token1, recipient, uint256(-amount1));
        //
        //     uint256 balance0Before = balance0();
        //     IUniswapV3SwapCallback(msg.sender).uniswapV3SwapCallback(amount0, amount1, data);
        //     require(balance0Before.add(uint256(amount0)) <= balance0(), 'IIA');
        // } else {
        //     if (amount0 < 0) TransferHelper.safeTransfer(token0, recipient, uint256(-amount0));
        //
        //     uint256 balance1Before = balance1();
        //     IUniswapV3SwapCallback(msg.sender).uniswapV3SwapCallback(amount0, amount1, data);
        //     require(balance1Before.add(uint256(amount1)) <= balance1(), 'IIA');
        // }
        //
        // emit Swap(msg.sender, recipient, amount0, amount1, state.sqrtPriceX96, state.liquidity, state.tick);
        // slot0.unlocked = true;

        Ok((0, 0))
    }
}
