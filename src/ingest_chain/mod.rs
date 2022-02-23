// we want an on-disk database of all uniswap LPs and swaps, and all hegic actions as well.
// should be able to parse this from chain!

// so what we do is we take the on-disk database, check what's already been scanned for and in what version.
// if something's not there, we do a pass over the relevant blocks to get those events.

mod talk_to_sled;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
enum Protocol {
	UniswapV2,
	HegicOptions,
}