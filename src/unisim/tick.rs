use super::fee::Fee;
use anyhow::{anyhow, Result};

/// Tick datatype. In the smart contracts it's an i24. TODO: fix the datatypes
pub type Tick = i32;
/// SecondsPerLiquidity datatype. uint160 in the contracts with X128 formatting. TODO: fix the datatypes
pub type SecondsPerLiquidity = float128;

const MIN_TICK: i32 = -887272;
const MAX_TICK: i32 = -MIN_TICK;

/// One tick's data, as stored in the tick table. For all sorts of dynamic programming goodies.
pub struct TickData {
    /// the total position liquidity that references this tick
    liquidity_gross: u128,
    /// amount of net liquidity added (subtracted) when tick is crossed from left to right (right to left),
    liquidity_net: i128,
    /// fee growth per unit of liquidity on the _other_ side of this tick (relative to the current tick)
    /// only has relative meaning, not absolute — the value depends on when the tick is initialized
    fee_growth_outside: Fee,
    /// the cumulative tick value on the other side of the tick
    tick_cumulative_outside: u128,
    /// the seconds per unit of liquidity on the _other_ side of this tick (relative to the current tick)
    /// only has relative meaning, not absolute — the value depends on when the tick is initialized
    seconds_per_liquidity_outside: SecondsOutside,
    /// the seconds spent on the other side of the tick (relative to the current tick)
    /// only has relative meaning, not absolute — the value depends on when the tick is initialized
    seconds_outside: u32,
    // true iff the tick is initialized, i.e. the value is exactly equivalent to the expression liquidityGross != 0
    // these 8 bits are set to prevent fresh sstores when crossing newly initialized ticks
    initialized: bool,
}

/// Derives max liquidity per tick from given tick spacing. Executed within the pool constructor
/// # Arguments
///
/// * `tickSpacing` The amount of required tick separation, realized in multiples of `tickSpacing`
///     e.g., a tickSpacing of 3 requires ticks to be initialized every 3rd tick i.e., ..., -6, -3, 0, 3, 6, ...
/// returns the max liquidity per tick
/// TODO: this may be incorrect due to int type things! check it.
fn tick_spacing_to_max_liquidity_per_tick(tick_spacing: i32) -> u128 {
    let min_tick = (MIN_TICK / tick_spacing) * tick_spacing;
    let max_tick = (MAX_TICK / tick_spacing) * tick_spacing;
    let num_ticks = ((max_tick - min_tick) / tick_spacing) + 1;
    return u128.MAX_INT / num_ticks;
}

impl TickTable {
    /// Retrieves fee growth data
    /// # Arguments
    ///
    /// * `tick_lower` The lower tick boundary of the position
    /// * `tick_upper` The upper tick boundary of the position
    /// * `tick_current` The current tick
    /// * `fee_growth_global` The all-time global fee growth, per unit of liquidity, in token0 and token1
    ///
    /// returns the all-time fee growth in token0 and token1, per unit of liquidity, inside the position's tick boundaries
    fn get_fee_growth_inside(
        &self,
        tick_lower: Tick,
        tick_upper: Tick,
        tick_current: Tick,
        fee_growth_global: Fee,
    ) -> FeeGrowth {
        let lower_data = self.get(tick_lower);
        let upper_data = self.get(tick_upper);

        // calculate fee growth below
        let fee_growth_below = if tick_current >= tick_lower {
            lower_data.fee_growth_outside
        } else {
            fee_growth_global - lower_data.fee_growth_outside
        };

        // calculate fee growth above
        let fee_growth_above = if tickCurrent < tickUpper {
            upper_data.fee_growth_outside
        } else {
            fee_growth_global - upper_data.fee_growth_outside
        };
        fee_growth_global - fee_growth_below - fee_growth_above
    }

    /// Updates a tick and returns true if the tick was flipped from initialized to uninitialized, or vice versa
    /// # Arguments
    ///
    /// * `tick` The tick that will be updated
    /// * `tick_current` The current tick
    /// * `liquidity_delta` A new amount of liquidity to be added (subtracted) when tick is crossed from left to right (right to left)
    /// * `fee_growth_global` The all-time global fee growth, per unit of liquidity, in token0 and token1
    /// * `seconds_per_liquidity_cumulative` The all-time seconds per max(1, liquidity) of the pool
    /// * `tick_cumulative`  The tick * time elapsed since the pool was first initialized
    /// * `time` The current block timestamp cast to a uint32
    /// * `is_upper` true for updating a position's upper tick, or false for updating a position's lower tick
    /// * `max_liquidity` the maximum liquidity allocation for a single tick
    ///
    /// returns true if the tick was flipped from initialized to uninitialized, or vice versa
    fn update(
        &mut self,
        tick: Tick,
        tick_current: Tick,
        liquidity_delta: i128,
        fee_growth_global: Fee,
        seconds_per_liquidity_cumulative: SecondsPerLiquidity,
        tick_cumulative: i128,
        time: u32,
        is_upper: bool,
        max_liquidity: u128,
    ) -> Result<bool> {
        let data = self.get(tick_lower);
        let liquidity_gross_before = data.liquidity_gross;
        // TODO: they use a library called liquiditymath here- maybe we implement that?
        let liquidity_gross_after = liquidity_gross_before + liquidity_delta;
        if liquidity_gross_after > max_liquidity {
            return Err(anyhow!("LO"));
        };
        let flipped = (liquidity_gross_after == 0) != (liquidity_gross_before == 0);
        if liquidity_gross_before == 0 {
            //  by convention, we assume that all growth before a tick was initialized happened _below_ the tick
            if tick <= tick_current {
                data.fee_growth_outside = fee_growth_global;
                data.seconds_per_liquidity_outside = seconds_per_liquidity_cumulative;
                data.tick_cumulative_outside = tick_cumulative;
                data.seconds_outside = time;
            }
            data.initialized = true;
        }

        data.liquidity_gross = liquidity_gross_after;

        // when the lower (upper) tick is crossed left to right (right to left), liquidity must be added (removed)
        // TODO: check math here

        data.liquidity_net = if is_upper {
            data.liquidity_net - liquidity_delta
        } else {
            data.liquidity_net + liquidity_delta
        };
        self.put(tick_lower, data);
        Ok(flipped)
    }

    // TODO: is this even necessary?
    // @notice Clears tick data
    // @param self The mapping containing all initialized tick information for initialized ticks
    // @param tick The tick that will be cleared
    // function clear(mapping(int24 => Tick.Info) storage self, int24 tick) internal {
    //     delete self[tick];
    // }

    /// Transitions to next tick as needed by price movement
    /// # Arguments
    ///
    /// * `tick`  The destination tick of the transition
    /// * `fee_growth_global` The all-time global fee growth, per unit of liquidity, in token0 and token1
    /// * `seconds_per_liquidity_cumulative` The current seconds per liquidity
    /// * `tick_cumulative`  The tick * time elapsed since the pool was first initialized
    /// * `time` The current block.timestamp
    ///
    /// returns The amount of liquidity added (subtracted) when tick is crossed from left to right (right to left)
    fn cross(
        &mut self,
        tick: Tick,
        fee_growth_global: Fee,
        seconds_per_liquidity_cumulative: SecondsPerLiquidity,
        tick_cumulative: i128,
        time: u32,
    ) -> i128 {
        let mut data = self.get(tick);
        data.fee_growth_outside = fee_growth_global - data.fee_growth_outside;
        data.seconds_per_liquidity_outside =
            seconds_per_liquidity_cumulative - data.seconds_per_liquidity_outside;
        data.tick_cumulative_outside = tick_cumulative - data.tick_cumulative_outside;
        data.seconds_outside = time - data.seconds_outside;
        self.put(tick, data);
        data.liquidity_net
    }
}
