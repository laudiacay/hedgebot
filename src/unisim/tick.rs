use std::HashMap;
use crate::fee_growth::*;

pub type Tick = i32; // lol you suck
pub type SecondsPerLiquidity = float128; // this is wrong too

pub struct TickData {
    liquidity_net: i128,
    liquidity_gross: u128,
    fee_growth_outside: FeeGrowth,
    seconds_outside: u32,
    tick_cumulative_outside: u128,
    seconds_per_liquidity_outside: SecondsOutside,
}
/*
/// TODO write me
/// @notice Derives max liquidity per tick from given tick spacing
/// @dev Executed within the pool constructor
/// @param tickSpacing The amount of required tick separation, realized in multiples of `tickSpacing`
///     e.g., a tickSpacing of 3 requires ticks to be initialized every 3rd tick i.e., ..., -6, -3, 0, 3, 6, ...
/// @return The max liquidity per tick
function tickSpacingToMaxLiquidityPerTick(int24 tickSpacing) internal pure returns (uint128) {
int24 minTick = (TickMath.MIN_TICK / tickSpacing) * tickSpacing;
int24 maxTick = (TickMath.MAX_TICK / tickSpacing) * tickSpacing;
uint24 numTicks = uint24((maxTick - minTick) / tickSpacing) + 1;
return type(uint128).max / numTicks;
}*/

// TODO fix the docs here lol
/// @notice Retrieves fee growth data
/// @param self The mapping containing all tick information for initialized ticks
/// @param tickLower The lower tick boundary of the position
/// @param tickUpper The upper tick boundary of the position
/// @param tickCurrent The current tick
/// @param feeGrowthGlobal0X128 The all-time global fee growth, per unit of liquidity, in token0
/// @param feeGrowthGlobal1X128 The all-time global fee growth, per unit of liquidity, in token1
/// @return feeGrowthInside0X128 The all-time fee growth in token0, per unit of liquidity, inside the position's tick boundaries
/// @return feeGrowthInside1X128 The all-time fee growth in token1, per unit of liquidity, inside the position's tick boundaries

impl TickTable {
    fn get_fee_growth_inside(&self,
    tick_lower: Tick,
    tick_upper: Tick,
    tick_current: Tick,
    fee_growth_global: FeeGrowth,
    ) -> FeeGrowth {
        let lower_data = self.get(tick_lower);
        let upper_data = self.get(tick_upper);

// calculate fee growth below
    let fee_growth_below = if (tick_current >= tick_lower) {
        lower_data.fee_growth_outside
    } else {
        fee_growth_global - lower_data.fee_growth_outside
    };

    // calculate fee growth above
    let fee_growth_above = if (tickCurrent < tickUpper) {
        upper_data.fee_growth_outside
    } else {
        fee_growth_global - upper_data.fee_growth_outside
    };
    fee_growth_global - fee_growth_below - fee_growth_above

    }
    // TODO update these parameters :)
/// @notice Updates a tick and returns true if the tick was flipped from initialized to uninitialized, or vice versa
/// @param self The mapping containing all tick information for initialized ticks
/// @param tick The tick that will be updated
/// @param tickCurrent The current tick
/// @param liquidityDelta A new amount of liquidity to be added (subtracted) when tick is crossed from left to right (right to left)
/// @param feeGrowthGlobal0X128 The all-time global fee growth, per unit of liquidity, in token0
/// @param feeGrowthGlobal1X128 The all-time global fee growth, per unit of liquidity, in token1
/// @param secondsPerLiquidityCumulativeX128 The all-time seconds per max(1, liquidity) of the pool
/// @param tickCumulative The tick * time elapsed since the pool was first initialized
/// @param time The current block timestamp cast to a uint32
/// @param upper true for updating a position's upper tick, or false for updating a position's lower tick
/// @param maxLiquidity The maximum liquidity allocation for a single tick
/// @return flipped Whether the tick was flipped from initialized to uninitialized, or vice versa
fn update(
&self,
tick: Tick,
tick_current: Tick,
liquidity_delta: i128,
fee_growth_global: FeeGrowth,
seconds_per_liquidity_cumulative: SecondsOutside,
tick_cumulative: i128,
time: u32,
is_upper: bool,
max_liquidity: u128,
) -> bool {
        let data = self.get(tick_lower);
// TODO implement me
// uint128 liquidityGrossBefore = info.liquidityGross;
// uint128 liquidityGrossAfter = LiquidityMath.addDelta(liquidityGrossBefore, liquidityDelta);
//
// require(liquidityGrossAfter <= maxLiquidity, 'LO');
//
// flipped = (liquidityGrossAfter == 0) != (liquidityGrossBefore == 0);
//
// if (liquidityGrossBefore == 0) {
// // by convention, we assume that all growth before a tick was initialized happened _below_ the tick
// if (tick <= tickCurrent) {
// info.feeGrowthOutside0X128 = feeGrowthGlobal0X128;
// info.feeGrowthOutside1X128 = feeGrowthGlobal1X128;
// info.secondsPerLiquidityOutsideX128 = secondsPerLiquidityCumulativeX128;
// info.tickCumulativeOutside = tickCumulative;
// info.secondsOutside = time;
// }
// info.initialized = true;
// }
//
// info.liquidityGross = liquidityGrossAfter;
//
// // when the lower (upper) tick is crossed left to right (right to left), liquidity must be added (removed)
// info.liquidityNet = upper
// ? int256(info.liquidityNet).sub(liquidityDelta).toInt128()
// : int256(info.liquidityNet).add(liquidityDelta).toInt128();
        false
    }
}

// TODO: implement me
/// @notice Clears tick data
/// @param self The mapping containing all initialized tick information for initialized ticks
/// @param tick The tick that will be cleared
// function clear(mapping(int24 => Tick.Info) storage self, int24 tick) internal {
// delete self[tick];
// }

// TODO: implement me
/// @notice Transitions to next tick as needed by price movement
/// @param self The mapping containing all tick information for initialized ticks
/// @param tick The destination tick of the transition
/// @param feeGrowthGlobal0X128 The all-time global fee growth, per unit of liquidity, in token0
/// @param feeGrowthGlobal1X128 The all-time global fee growth, per unit of liquidity, in token1
/// @param secondsPerLiquidityCumulativeX128 The current seconds per liquidity
/// @param tickCumulative The tick * time elapsed since the pool was first initialized
/// @param time The current block.timestamp
/// @return liquidityNet The amount of liquidity added (subtracted) when tick is crossed from left to right (right to left)
// function cross(
// mapping(int24 => Tick.Info) storage self,
// int24 tick,
// uint256 feeGrowthGlobal0X128,
// uint256 feeGrowthGlobal1X128,
// uint160 secondsPerLiquidityCumulativeX128,
// int56 tickCumulative,
// uint32 time
// ) internal returns (int128 liquidityNet) {
// Tick.Info storage info = self[tick];
// info.feeGrowthOutside0X128 = feeGrowthGlobal0X128 - info.feeGrowthOutside0X128;
// info.feeGrowthOutside1X128 = feeGrowthGlobal1X128 - info.feeGrowthOutside1X128;
// info.secondsPerLiquidityOutsideX128 = secondsPerLiquidityCumulativeX128 - info.secondsPerLiquidityOutsideX128;
// info.tickCumulativeOutside = tickCumulative - info.tickCumulativeOutside;
// info.secondsOutside = time - info.secondsOutside;
// liquidityNet = info.liquidityNet;
// }
// }