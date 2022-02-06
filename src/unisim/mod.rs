mod tick;
mod fee_growth;

use std::HashMap;
use tick::*;

type Address = String; // lmao, for now

struct PositionID {
    address: String,
    lower_bound: Tick,
    upper_bound: Tick,
}

struct Position {
    liquidity: u128,
    fee_growth_inside_0_last: float256,
    fee_growth_inside_1_last: float256,
}

struct UniV3Pool {
    liquidity: u128,
    sqrt_price: float64,
    tick: Tick,
    tick_spacing: u32,
    fee_growth_global_0: float256,
    fee_growth_global_1: float256,
    protocol_fees_token_0: u128,
    protocol_fees_token_1: u128,
    gamma_percent_fee: float32,
    phi_percent_fee: float32,
    tick_chart: HashMap<Tick, TickData>,
    positions: HashMap<PositionID, Position>,
}

enum SwapEvent {
    SetPosition{position_id: PositionID, delta_liquidity: i128},
    Swap0to1{amount: u128},
    Swap1to0{amount: u128},
}

impl UniV3Pool {
    pub fn new(start_price: float64, tick_spacing: u32, gamma_percent_fee: float32, phi_percent_fee: float32) -> Self {
        Self {
            liquidity: 0,
            sqrt_price: math::sqrt(start_price),
            tick: tick_math::tick_from_price(start_price),
            tick_spacing,
            fee_growth_global_0: 0,
            fee_growth_global_1: 0,
            protocol_fees_token_0: 0,
            protocol_fees_token_1: 0,
            gamma_percent_fee,
            phi_percent_fee,
            tick_chart: HashMap::empty(),
            positions: HashMap::empty(),
        }
    }
    fn next_highest_tick(&self) {
        return self.tick + self.tick_spacing;
    }
    fn next_lowest_tick(&self) {
        return self.tick - self.tick_spacing
    }
}


