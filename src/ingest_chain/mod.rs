// we want an on-disk database of all uniswap LPs and swaps, and all hegic actions as well.
// should be able to parse this from chain!

// so what we do is we take the on-disk database, check what's already been scanned for and in what version.
// if something's not there, we do a pass over the relevant blocks to get those events.

use sled;
use anyhow::Result;


const HEADERS_TREE_KEY: b"HEADERS_TREE"

#[derive(Debug, Display, Clone, Copy)]
enum Protocol {
	UniswapV2,
	HegicOptions,
}

/// this labels which event type we're parsing out!
struct EventParserLabel {
	/// a protocol- has a scanner and a parser
	protocol: Protocol,
	/// kept sorted for obvious reasons.
	our_ranges: Vec<BlockRangesWeHave>
}

fn sled_init(sled_path: &str, supported_protocols: Vec<Protocol>) -> Result<sled::Db> {
	let db = sled::open(path)?;
	let headers_tree = db.open_tree(HEADERS_TREE_KEY);


	db
}


fn ingest_to_sled(start_block: u64, end_block: u64, things_to_ingest: HashSet<) {

}