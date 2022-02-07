use super::fee::Fee;
use anyhow::{anyhow, Result};
use num_bigint::{BigInt, BigUint};
use num_rational::BigRational;
use std::borrow::Borrow;
use std::collections::HashMap;

pub type Tick = i32;

pub(crate) const MIN_TICK: i32 = -887272;
pub(crate) const MAX_TICK: i32 = -MIN_TICK;

/// One tick's data, as stored in the tick table. For all sorts of dynamic programming goodies.
pub struct TickInfo {
    /// the total position liquidity that references this tick
    liquidity_gross: BigUint,
    /// amount of net liquidity added (subtracted) when tick is crossed from left to right (right to left),
    liquidity_net: BigInt,
    /// fee growth per unit of liquidity on the _other_ side of this tick (relative to the current tick)
    /// only has relative meaning, not absolute — the value depends on when the tick is initialized
    fee_growth_outside: Fee,
    /// the cumulative tick value on the other side of the tick
    tick_cumulative_outside: i64,
    /// the seconds per unit of liquidity on the _other_ side of this tick (relative to the current tick)
    /// only has relative meaning, not absolute — the value depends on when the tick is initialized
    seconds_per_liquidity_outside: BigRational,
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
/// TODO: this is incorrect due to int type things! check it.
fn tick_spacing_to_max_liquidity_per_tick(tick_spacing: i32) -> u128 {
    let min_tick = (MIN_TICK / tick_spacing) * tick_spacing;
    let max_tick = (MAX_TICK / tick_spacing) * tick_spacing;
    let num_ticks = ((max_tick - min_tick) / tick_spacing) + 1;
    return u128::MAX_INT / num_ticks;
}

type TickTable = HashMap<Tick, TickInfo>;

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
    table: &TickTable,
    tick_lower: Tick,
    tick_upper: Tick,
    tick_current: Tick,
    fee_growth_global: Fee,
) -> Result<Fee> {
    let lower_data = table
        .get(tick_lower.borrow())
        .ok_or(anyhow!("tick not found"))?;
    let upper_data = table
        .get(tick_upper.borrow())
        .ok_or(anyhow!("tick not found"))?;

    // calculate fee growth below
    let fee_growth_below = if tick_current >= tick_lower {
        lower_data.fee_growth_outside
    } else {
        fee_growth_global - lower_data.fee_growth_outside
    };

    // calculate fee growth above
    let fee_growth_above = if tick_current < tick_upper {
        upper_data.fee_growth_outside
    } else {
        fee_growth_global - upper_data.fee_growth_outside
    };
    Ok(fee_growth_global - fee_growth_below - fee_growth_above)
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
    table: &mut TickTable,
    tick: Tick,
    tick_current: Tick,
    liquidity_delta: i128,
    fee_growth_global: Fee,
    seconds_per_liquidity_cumulative: SecondsPerLiquidity,
    tick_cumulative: i64,
    time: u32,
    is_upper: bool,
    max_liquidity: u128,
) -> Result<bool> {
    let info = table.get(tick.borrow()).ok_or(anyhow!("tick not found"))?;
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
            info.fee_growth_outside = fee_growth_global;
            info.seconds_per_liquidity_outside = seconds_per_liquidity_cumulative;
            info.tick_cumulative_outside = tick_cumulative;
            info.seconds_outside = time;
        }
        info.initialized = true;
    }

    info.liquidity_gross = liquidity_gross_after;

    // when the lower (upper) tick is crossed left to right (right to left), liquidity must be added (removed)
    // TODO: check math here

    info.liquidity_net = if is_upper {
        info.liquidity_net - liquidity_delta
    } else {
        info.liquidity_net + liquidity_delta
    };
    table.insert(tick_lower, *info);
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
    table: &mut TickTable,
    tick: Tick,
    fee_growth_global: Fee,
    seconds_per_liquidity_cumulative: SecondsPerLiquidity,
    tick_cumulative: i64,
    time: u32,
) -> Result<BigInt> {
    let info = table.get(tick.borrow()).ok_or(anyhow!("tick not found"))?;
    let new_info = TickInfo {
        fee_growth_outside: fee_growth_global - info.fee_growth_outside,
        seconds_per_liquidity_outside: seconds_per_liquidity_cumulative
            - info.seconds_per_liquidity_outside,
        tick_cumulative_outside: tick_cumulative - info.tick_cumulative_outside,
        seconds_outside: time - info.seconds_outside,
        ..*info
    };
    table.insert(tick, new_info);
    Ok(new_info.liquidity_net.clone())
}
