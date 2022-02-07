mod fee;
mod tick;

use crate::unisim::fee::Fee;
use anyhow::{anyhow, Result};
use num_bigint::{BigInt, BigUint};
use num_rational::BigRational;
use std::collections::{HashMap, HashSet};
use tick::*;

type Address = String; // lmao, for now

struct PositionID {
    address: String,
    lower_bound: Tick,
    upper_bound: Tick,
}

struct Position {
    liquidity: BigInt,
    fee_growth_inside: Fee,
}

struct UniV3Pool {
    liquidity: BigRational,
    // the current price
    sqrt_price: BigRational,
    // the current tick
    tick: Tick,
    tick_spacing: u32,
    fee_growth_global: Fee,
    protocol_fees: Fee,
    // the current protocol fee as a percentage of the swap fee taken on withdrawal
    // represented as an integer denominator (1/x)%
    gamma_percent_fee: u32,
    phi_percent_fee: u32,
    tick_chart: TickTable,
    positions: HashMap<PositionID, Position>,
    my_positions: HashSet<PositionID>,
    // the most-recently updated index of the observations array
    observation_index: u16,
    // the current maximum number of observations that are being stored
    observation_cardinality: u16,
    // pool balance of token 0
    balance_0: BigRational,
    // pool balance of token 1
    balance_1: BigRational,
}

enum SwapEvent {
    SetPosition {
        position_id: PositionID,
        delta_liquidity: BigRational,
    },
    Swap0to1 {
        amount: BigRational,
    },
    Swap1to0 {
        amount: BigRational,
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

struct SwapCache {
    // the protocol fee for the input token
    fee_protocol: u8,
    // liquidity at the beginning of the swap
    liquidity_start: BigRational,
    // the timestamp of the current block
    block_timestamp: u32,
    // the current value of the tick accumulator, computed only if we cross an initialized tick
    tick_cumulative: i64,
    // the current value of seconds per liquidity accumulator, computed only if we cross an initialized tick
    seconds_per_liquidity_cumulative: BigUint,
    // whether we've computed and cached the above two accumulators
    computed_latest_observation: bool,
}

// the top level state of the swap, the results of which are recorded in storage at the end
struct SwapState {
    // the amount remaining to be swapped in/out of the input/output asset
    amount_specified_remaining: BigRational,
    // the amount already swapped out/in of the output/input asset
    amount_calculated: BigRational,
    // current sqrt(price)
    sqrt_price: SqrtPriceUnits,
    // the tick associated with the current price
    tick: Tick,
    // the global fee growth of the input token
    fee_growth_global: BigRational,
    // amount of input token paid as protocol fee
    protocol_fee: BigRational,
    // the current liquidity in range
    liquidity: BigRational,
}

struct StepComputations {
    // the price at the beginning of the step
    sqrt_price_start: BigRational,
    // the next tick to swap to from the current tick in the swap direction
    tick_next: Tick,
    // whether tick_next is initialized or not
    initialized: bool,
    // sqrt(price) for the next tick (1/0)
    sqrt_price_next: BigRational,
    // how much is being swapped in in this step
    amount_in: BigRational,
    // how much is being swapped out
    amount_out: BigRational,
    // how much fee is being paid in
    fee_amount: BigRational,
}

// TODO: document me!
impl UniV3PoolSimulator {
    pub fn set_position(&mut self, position_id: PositionID, state: Position, mine: bool) {}
    pub fn get_my_holdings_value(&self, in_token_1: bool) -> BigInt {
        return BigInt::zero();
    }
    pub fn get_my_fees_value(&self) -> (u128, u128) {
        (0, 0)
    }
    pub fn swap(
        &self,
        zero_for_one: bool,
        amount_specified: BigInt,
        sqrt_price_limit: SqrtPriceUnits,
        block_timestamp: BigUint,
    ) -> Result<(BigInt, BigInt)> {
        if amount_specified == BigInt::zero() {
            Err(anyhow!("AS"))?
        };

        if zero_for_one {
            if !(sqrt_price_limit < slot0Start.sqrt_price
                && sqrt_price_limit > TickMath.MIN_SQRT_RATIO)
            {
                Err(anyhow!("SPL"))?
            }
        } else {
            if !(sqrt_price_limit > slot0Start.sqrt_price
                && sqrt_price_limit < TickMath.MAX_SQRT_RATIO)
            {
                Err(anyhow!("SPL"))?
            }
        };

        let mut cache = SwapCache {
            liquidity_start: liquidity,
            block_timestamp: _blockTimestamp(),
            tick_cumulative: 0,
            fee_protocol: if zero_for_one {
                slot0Start.feeProtocol % 16
            } else {
                slot0Start.feeProtocol >> 4
            },
            seconds_per_liquidity_cumulative: BigUint::zero(),
            computed_latest_observation: false,
        };

        let exact_input = amount_specified.greater_than(BigInt::zero());

        let mut state = SwapState {
            amount_specified_remaining: amountSpecified,
            amount_calculated: BigRational::zero(),
            sqrt_price: slot0Start.sqrt_price,
            tick: slot0Start.tick,
            fee_growth_global: if zero_for_one {
                self.fee_growth_global.token_0
            } else {
                self.fee_growth_global.token_1
            },
            protocol_fee: BigRational::zero(),
            liquidity: cache.liquidity_start,
        };

        // continue swapping as long as we haven't used the entire input/output and haven't reached the price limit
        while !state.amount_specified_remaining.is_zero() && state.sqrt_price != sqrt_price_limit {
            let mut step = StepComputations {
                sqrt_price_start: BigRational::zero(),
                tick_next: 0,
                initialized: false,
                sqrt_price_next: BigRational::zero(),
                amount_in: BigRational::zero(),
                amount_out: BigRational::zero(),
                fee_amount: BigRational::zero(),
            };

            step.sqrt_price_start = state.sqrt_price;

            (step.tick_next, step.initialized) =
                tickBitmap.nextInitializedTickWithinOneWord(state.tick, tickSpacing, zero_for_one);

            // ensure that we do not overshoot the min/max tick, as the tick bitmap is not aware of these bounds
            if step.tick_next < tick::MIN_TICK {
                step.tick_next = tick::MIN_TICK;
            } else if step.tick_next > tick::MAX_TICK {
                step.tick_next = tick::MAX_TICK;
            }

            // get the price for the next tick
            step.sqrt_price_next = TickMath.getSqrtRatioAtTick(step.tick_next);

            // compute values to swap to the target tick, price limit, or point where input/output amount is exhausted
            (
                state.sqrt_price,
                step.amount_in,
                step.amount_out,
                step.fee_amount,
            ) = SwapMath.computeSwapStep(
                state.sqrt_price,
                {
                    let direction = if zero_for_one {
                        step.sqrt_price_next < sqrt_price_limit
                    } else {
                        step.sqrt_price_next > sqrt_price_limit
                    };
                    if direction {
                        sqrt_price_limit
                    } else {
                        step.sqrt_price_next
                    }
                },
                state.liquidity,
                state.amount_specified_remaining,
                fee,
            );

            if exactInput {
                state.amount_specified_remaining -= (step.amount_in + step.fee_amount);
                state.amount_calculated = state.amount_calculated - step.amount_out;
            } else {
                state.amount_specified_remaining += step.amount_out.toInt256();
                state.amount_calculated =
                    state.amount_calculated + (step.amount_in + step.fee_amount);
            }

            // if the protocol fee is on, calculate how much is owed, decrement feeAmount, and increment protocolFee
            if cache.fee_protocol > 0 {
                let delta = step.fee_amount / cache.fee_protocol;
                step.fee_amount -= delta;
                state.protocol_fee += uint128(delta);
            }

            // update global fee tracker
            if state.liquidity.is_positive() {
                state.fee_growth_global +=
                    FullMath.mulDiv(step.fee_amount, FixedPoint128.Q128, state.liquidity)
            };

            // shift tick if we reached the next price
            if state.sqrt_price == step.sqrt_price_next {
                // if the tick is initialized, run the tick transition
                if step.initialized {
                    // check for the placeholder value, which we replace with the actual value the first time the swap
                    // crosses an initialized tick
                    if !cache.computed_latest_observation {
                        (
                            cache.tick_cumulative,
                            cache.seconds_per_liquidity_cumulative,
                        ) = observations.observeSingle(
                            cache.block_timestamp,
                            0,
                            slot0Start.tick,
                            slot0Start.observation_index,
                            cache.liquidity_start,
                            slot0Start.observation_cardinality,
                        );
                        cache.computed_latest_observation = true;
                    }
                    let liquidity_net = ticks.cross(
                        step.tick_next,
                        if zero_for_one {
                            state.fee_growth_global
                        } else {
                            feeGrowthGlobal0X128
                        },
                        if zero_for_one {
                            fee_growth_global1
                        } else {
                            state.fee_growth_global
                        },
                        cache.seconds_per_liquidity_cumulative,
                        cache.tick_cumulative,
                        cache.block_timestamp,
                    );
                    // if we're moving leftward, we interpret liquidity_net as the opposite sign
                    // safe because liquidity_net cannot be type(int128).min
                    if zero_for_one {
                        liquidity_net = -liquidity_net
                    };

                    state.liquidity = LiquidityMath.addDelta(state.liquidity, liquidity_net);
                }

                state.tick = if zero_for_one {
                    step.tick_next - 1
                } else {
                    step.tick_next
                };
            } else if state.sqrt_price != step.sqrt_price_start {
                // recompute unless we're on a lower tick boundary (i.e. already transitioned ticks), and haven't moved
                state.tick = TickMath.getTickAtSqrtRatio(state.sqrt_price);
            }
        }

        // update tick and write an oracle entry if the tick change
        if state.tick != slot0Start.tick {
            let (observation_index, observation_cardinality) = observations.write(
                slot0Start.observation_index,
                cache.block_timestamp,
                slot0Start.tick,
                cache.liquidity_start,
                slot0Start.observation_cardinality,
                slot0Start.observationCardinalityNext,
            );
            (
                self.sqrt_price,
                self.tick,
                self.observation_index,
                self.observation_cardinality,
            ) = (
                state.sqrt_price,
                state.tick,
                observation_index,
                observation_cardinality,
            );
        } else {
            // otherwise just update the price
            self.sqrt_price = state.sqrt_price;
        }

        // update liquidity if it changed
        if cache.liquidity_start != state.liquidity {
            liquidity = state.liquidity
        };

        // update fee growth global and, if necessary, protocol fees
        // overflow is acceptable, protocol has to withdraw before it hits type(uint128).max fees
        if zero_for_one {
            fee_growth_global0 = state.fee_growth_global;
            if state.protocol_fee.is_positive() {
                protocolFees.token0 += state.protocol_fee
            };
        } else {
            feeGrowthGlobal1X128 = state.fee_growth_global;
            if state.protocol_fee.is_positive() {
                protocolFees.token1 += state.protocol_fee
            };
        }

        let (amount0, amount1) = if zero_for_one == exactInput {
            (
                amountSpecified - state.amount_specified_remaining,
                state.amount_calculated,
            )
        } else {
            (
                state.amount_calculated,
                amountSpecified - state.amount_specified_remaining,
            )
        };

        // do the transfers and collect payment
        if zero_for_one {
            if amount1.is_negative() {
                // TransferHelper.safeTransfer(token1, recipient, uint256(-amount1))
                self.balance_1 -= amount_1;
            };

            let balance_0_before = balance0();
            IUniswapV3SwapCallback(msg.sender).uniswapV3SwapCallback(amount0, amount1, data);
            if !(balance_0_before.add(uint256(amount0)) <= balance0()) {
                Err(anyhow!("IIA"))?
            }
        } else {
            if amount0.is_negative() {
                // TransferHelper.safeTransfer(token0, recipient, uint256(-amount0))
                self.balance_0 -= amount_0;
            };

            let balance_1_before = balance1();
            IUniswapV3SwapCallback(msg.sender).uniswapV3SwapCallback(amount0, amount1, data);
            if !(balance_1_before + amount1) <= balance1() {
                Err(anyhow!("IIA"))?
            }
        }

        Ok(amount0, amount1)
    }
}
