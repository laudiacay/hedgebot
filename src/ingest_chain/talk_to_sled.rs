use crate::ingest_chain::Protocol;
use std::collections::HashSet;
use sled;
use anyhow::Result;
use ranges::Ranges;
use serde::{Serialize, Deserialize};

type BlockNumber = u64;

#[derive(Serialize, Deserialize)]
struct Timestamp {
	block_number: BlockNumber,
// preserving my boyfriend's "code" here in a comment from when he "helped" me "work"
// 	hack hack hack hack hack hack hack hack hack (hellow, owrld);
// 	{hack ,hack hack} (hellow, world}
// 		if ur are a noob
// 		then l0l i h4xx u8: 
// }
	log_id: u64 // this is wrong and sucks and won't work. fix it later.
}

#[derive(Serialize, Deserialize)]
struct Event<'a> {
	protocol: Protocol,
	event_data: &'a [u8], //sucks but works for now
}

type Blocks = Ranges<BlockNumber>;

/// this is the tree key for the headers tree, which tells us which data is stored in the sled database.
/// the structure of the DB is as follows. there's a headers tree, which has keys of protocol IDs, and RangeSets of blocks that these protocols are synced to disk for
/// then there's another sled tree for all the protocols' events during the relevant ranges. 
/// these store timestamped events, meant to be iterated over. they're timestamped by block and then an index of the event inside the block.
/// this event timestamp is the key in the protocol btree- you iterate over them in order, obviously.
/// and the data inside (the event) contains a descriptor of which protocol the event pertains to, as well as the event data for you to do what you want with.
const HEADERS_TREE_KEY: &[u8] = b"HEADERS_TREE";
const DATA_TREE_KEY:  &[u8]  =b"DATA_TREE";

struct SledHandle {
	db: sled::Db,
	header_tree: sled::Tree,
	data_tree: sled::Tree,
}

impl SledHandle {
	/// opens/creates the database at the requested sled path. ensures all requested protocols are in the headers table.
	fn new(sled_path: &str) -> Result<Self> {
		let db = sled::open(sled_path)?;
		let handle = SledHandle{ db, header_tree: db.open_tree(HEADERS_TREE_KEY)?, data_tree: db.open_tree(DATA_TREE_KEY)?};
		// this should set header tree merge to be the rangemap merge
		handle.header_tree.set_merge_operator(|_, _, _| todo!());
		Ok(handle)
	}

	fn add_time_range(&self, protocols_covered: Vec<Protocol>, block_range: Blocks, events: Vec<(Timestamp, Event)>) -> Result<()> {
		// FIXME THIS IS NOT ATOMIC!!!! can fail at bad times in bad ways .... no good
		// FIXME make sure this short-circuits if needed.
		let mut batch = sled::Batch::default();
		for (ts, event) in events.iter() {
			batch.insert(bincode::serialize(&ts)?, bincode::serialize(&event)?);
		}
		self.data_tree.apply_batch(batch)?;
		for protocol in protocols_covered.iter() {
			self.header_tree.merge(bincode::serialize(&protocol)?, bincode::serialize(&block_range)?)?;
		}
		Ok(())
	}

	fn check_time_range(&self, protocol: Protocol, block_range: Blocks) -> Result<bool> {
		match self.data_tree.get(bincode::serialize(&protocol)?)? {
			None => Ok(false),
			Some(ranges_bytes) => {
				let old_ranges : Blocks = bincode::deserialize(&ranges_bytes[..])?;
				Ok(block_range - old_ranges == Ranges::new())
			}
		}
	}

	/// outer Result is for general errors while doing the thing. 
	/// inner option is for "ya dun goofed, events weren't ingested for this time range first"
	fn get_time_range(&self, protocol_filter: HashSet<Protocol>, block_range: Blocks) -> Result<Option<Vec<(Timestamp, Event)>>> {
		// first, check real quick if anything's not on disk! then we should go get it- don't want any incomplete data.
		for protocol in protocol_filter.iter() {
			if !self.check_time_range(*protocol, block_range)? {
				return Ok(None)
			}
		}
		// next iterate over all of the blocks and all of their event timestamps in the range
		Ok(Some(block_range.iter().map(|block| self.data_tree.scan_prefix(block).collect()).collect().flatten()))
	}
}
