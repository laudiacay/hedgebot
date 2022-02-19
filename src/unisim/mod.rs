mod fee;
mod liq_math;
mod swap_math;
mod tick;

use crate::unisim::fee::Fee;
use anyhow::{anyhow, Result};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use tick::*;

type Address = String; // lmao, for now

struct PositionID {
    address: Address,
    lower_bound: Tick,
    upper_bound: Tick,
}

struct Position {
    // the amount of liquidity owned by this position
    liquidity: u128,
    // fee growth per unit of liquidity as of the last update to liquidity or fees owed
    fee_growth_inside_last: Fee,
    // the fees owed to the position owner in token0/token1
    tokensOwed0: u128,
    tokensOwed1: u128,
}

struct UniV3PoolMutableState {
    // current virtual liquidity within tick
    liquidity: f64,
    // the current price
    sqrt_price: f64,
    // the current tick
    tick: Tick,
    // current total fees globally
    fee_growth_global: Fee,
    // protocol fees total accumulated
    protocol_fees: Fee,
    // all the ticks with their info
    tick_chart: TickTable,
    // all the positions
    positions: HashMap<PositionID, Position>,
    // just my positions (to calculate gains/losses)
    my_positions: HashSet<PositionID>,
    // pool balance of token 0
    balance_0: f64,
    // pool balance of token 1
    balance_1: f64,
}

impl UniV3PoolMutableState {
    fn new(start_price: f64) -> Self {
        Self {
            liquidity: 0.0,
            sqrt_price: start_price.sqrt(),
            tick: tick::tick_from_sqrt_price(start_price.sqrt()),
            fee_growth_global: Fee::zero(),
            protocol_fees: Fee::zero(),
            tick_chart: HashMap::new(),
            positions: HashMap::new(),
            my_positions: HashSet::new(),
            balance_0: 0.0,
            balance_1: 0.0,
        }
    }
}

struct UniV3Pool {
    // spacing between ticks...
    tick_spacing: i32,
    // the current lp fee as a percentage of the swap fee taken on withdrawal
    // represented as an integer denominator (1/x)%
    lp_fees: f64,
    // fee that goes to the protocol
    protocol_fees: f64,
    // and the internal mutable state
    state: RefCell<UniV3PoolMutableState>,
}

enum SwapEvent {
    SetPosition {
        position_id: PositionID,
        delta_liquidity: f64,
    },
    Swap0to1 {
        amount: f64,
    },
    Swap1to0 {
        amount: f64,
    },
}

impl UniV3Pool {
    pub fn new(start_price: f64, tick_spacing: i32, lp_fees: f64, protocol_fees: f64) -> Self {
        Self {
            tick_spacing,
            lp_fees,
            protocol_fees,
            state: RefCell::new(UniV3PoolMutableState::new(start_price)),
        }
    }
    fn next_highest_tick(&self, tick: Tick) -> Tick {
        return tick + self.tick_spacing;
    }
    fn next_lowest_tick(&self, tick: Tick) -> Tick {
        return tick - self.tick_spacing;
    }
}

// the top level state of the swap, the results of which are recorded in storage at the end
struct SwapState {
    // the amount remaining to be swapped in/out of the input/output asset
    amount_specified_remaining: f64,
    // the amount already swapped out/in of the output/input asset
    amount_calculated: f64,
    // current sqrt(price)
    sqrt_price: f64,
    // the tick associated with the current price
    tick: Tick,
    // the global fee growth of the input token
    fee_growth_global: f64,
    // amount of input token paid as protocol fee
    protocol_fee: f64,
    // the current liquidity in range
    liquidity: f64,
}

struct StepComputations {
    // the price at the beginning of the step
    sqrt_price_start: f64,
    // the next tick to swap to from the current tick in the swap direction
    tick_next: Tick,
    // whether tick_next is initialized or not
    initialized: bool,
    // sqrt(price) for the next tick (1/0)
    sqrt_price_next: f64,
    // how much is being swapped in in this step
    amount_in: f64,
    // how much is being swapped out
    amount_out: f64,
    // how much fee is being paid in
    fee_amount: f64,
}

// TODO: document me!
impl UniV3Pool {
    pub fn set_position(&mut self, position_id: PositionID, state: Position, mine: bool) {}
    pub fn get_my_holdings_value(&self, in_token_1: bool) -> f64 {
        return 0.0;
    }
    pub fn get_my_fees_value(&self) -> (u128, u128) {
        (0, 0)
    }
    pub fn mint() {}
    pub fn burn() {}
    pub fn swap(
        &self,
        zero_for_one: bool,
        amount_specified: f64,
        sqrt_price_limit: f64,
        block_timestamp: u32,
    ) -> Result<(f64, f64)> {
        let old_mutable_state = self.state.borrow();

        let mut state = SwapState {
            amount_specified_remaining: amount_specified,
            amount_calculated: 0.0,
            sqrt_price: old_mutable_state.sqrt_price,
            tick: old_mutable_state.tick,
            fee_growth_global: old_mutable_state.fee_growth_global,
            protocol_fee: old_mutable_state.protocol_fees,
            liquidity: self.state.borrow().liquidity,
        };

        // continue swapping as long as we haven't used the entire input/output and haven't reached the price limit
        while state.amount_specified_remaining >= 0.0 && state.sqrt_price != sqrt_price_limit {
            step.sqrt_price_start = state.sqrt_price;

            let (tick_next, initialized) = tick_bitmap.nextInitializedTickWithinOneWord(
                state.tick,
                self.tick_spacing,
                zero_for_one,
            );

            // ensure that we do not overshoot the min/max tick, as the tick bitmap is not aware of these bounds
            if step.tick_next < tick::MIN_TICK {
                step.tick_next = tick::MIN_TICK;
            } else if step.tick_next > tick::MAX_TICK {
                step.tick_next = tick::MAX_TICK;
            }

            // get the price for the next tick
            step.sqrt_price_next = TickMath.getSqrtRatioAtTick(step.tick_next);

            // compute values to swap to the target tick, price limit, or point where input/output amount is exhausted
            let (state_sqrt_price, step_amount_in, step_amount_out, step_fee_amount) =
                swap_math::compute_swap_step(
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

            if exact_input {
                state.amount_specified_remaining -= step.amount_in + step.fee_amount;
                state.amount_calculated -= step.amount_out;
            } else {
                state.amount_specified_remaining += step.amount_out.toInt256();
                state.amount_calculated += step.amount_in + step.fee_amount;
            }

            // if the protocol fee is on, calculate how much is owed, decrement feeAmount, and increment protocolFee
            if cache.fee_protocol > 0 {
                let delta = step.fee_amount / cache.fee_protocol;
                step.fee_amount -= delta;
                state.protocol_fee += uint128(delta);
            }

            // update global fee tracker
            if state.liquidity.is_positive() {
                state.fee_growth_global += step.fee_amount.div(state.liquidity);
            };

            // shift tick if we reached the next price
            if state.sqrt_price == step.sqrt_price_next {
                // if the tick is initialized, run the tick transition
                if step.initialized {
                    // check for the placeholder value, which we replace with the actual value the first time the swap
                    // crosses an initialized tick
                    let mut liquidity_net = tick::cross(
                        self.tick_chart,
                        step.tick_next,
                        if zero_for_one {
                            Fee {
                                token_0: state.fee_growth_global.token_0,
                                token_1: fee_growth_global.token_1,
                            }
                        } else {
                            Fee {
                                token_0: fee_growth_global.token_0,
                                token_1: state.fee_growth_global.token_1,
                            }
                        },
                        cache.seconds_per_liquidity_cumulative.clone(),
                        cache.tick_cumulative,
                        cache.block_timestamp,
                    );
                    // if we're moving leftward, we interpret liquidity_net as the opposite sign
                    // safe because liquidity_net cannot be type(int128).min
                    if zero_for_one {
                        liquidity_net = -liquidity_net
                    };

                    state.liquidity =
                        LiquidityMath.addDelta(state.liquidity.clone(), liquidity_net);
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

        // update tick if the tick change
        if state.tick != slot0Start.tick {
            (self.sqrt_price, self.tick) = (state.sqrt_price, state.tick);
        } else {
            // otherwise just update the price
            self.state.borrow().sqrt_price = state.sqrt_price;
        }

        // update liquidity if it changed
        if cache.liquidity_start != state.liquidity {
            liquidity = state.liquidity.clone()
        };

        // update fee growth global and, if necessary, protocol fees
        // overflow is acceptable, protocol has to withdraw before it hits type(uint128).max fees
        if zero_for_one {
            fee_growth_global0 = state.fee_growth_global;
            if state.protocol_fee.is_positive() {
                protocolFees.token0 += state.protocol_fee
            };
        } else {
            fee_growth_global1 = state.fee_growth_global;
            if state.protocol_fee.is_positive() {
                protocolFees.token1 += state.protocol_fee
            };
        }

        let (amount_0, amount_1) = if zero_for_one == exact_input {
            (
                amount_specified - state.amount_specified_remaining,
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
            self.balance_1 -= amount_1;
            self.balance_0 += amount_0;
            if amount_1.is_negative() {
                // TransferHelper.safeTransfer(token1, recipient, uint256(-amount_1))
                self.balance_1 -= amount_1;
            };

            let balance_0_before = balance0();
            IUniswapV3SwapCallback(msg.sender).uniswapV3SwapCallback(amount_0, amount_1, data);
            if !(balance_0_before.add(uint256(amount_0)) <= balance0()) {
                Err(anyhow!("IIA"))?
            }
        } else {
            if amount_0.is_negative() {
                // TransferHelper.safeTransfer(token0, recipient, uint256(-amount_0))
                self.balance_0 -= amount_0;
            };

            IUniswapV3SwapCallback(msg.sender).uniswapV3SwapCallback(amount_0, amount_1, data);
        }

        Ok((amount_0, amount_1))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        unimplemented!()
    }
}
