use sled;
use anyhow::Result;
use rangemap::RangeMap;

/// this is the tree key for the headers tree, which tells us which data is stored in the sled database.
/// the structure of the DB is as follows. there's a headers tree, which has keys of protocol IDs, and RangeSets of blocks that these protocols are synced to disk for
/// then there's another sled tree for all the protocols' events during the relevant ranges. 
/// these store timestamped events, meant to be iterated over. they're timestamped by block and then an index of the event inside the block.
/// this event timestamp is the key in the protocol btree- you iterate over them in order, obviously.
/// and the data inside (the event) contains a descriptor of which protocol the event pertains to, as well as the event data for you to do what you want with.

const HEADERS_TREE_KEY: b"HEADERS_TREE"
const DATA_TREE_KEY: b"DATA_TREE"

struct SledHandle {
	db: sled::Db,
	header_tree: sled::Tree,
	data_tree: sled::Tree,
}

impl SledHandle {
	/// opens/creates the database at the requested sled path. ensures all requested protocols are in the headers table.
	fn new(sled_path: &str) -> Result<Self> {
		let handle = SledHandle{ db: sled::open(path)?, header_tree: todo!(), data_tree: todo!()});
		// this should set header tree merge to be the rangemap merge
		handle.header_tree.set_merge_operator(merge_operator: todo!())?;
		Ok(handle)
	}

	fn add_time_range(&self, protocols_covered: Vec<Protocol>, block_range: BlockRange, events: Vec<(Timestamp, Event)>) -> Result<()> {
		// FIXME THIS IS NOT ATOMIC!!!! can fail at bad times in bad ways .... no good
		let mut batch = sled::Batch::default();
		for (ts, event) in events.iter() {
			batch.insert(ts, event);
		}
		self.data_tree.apply_batch(batch)?
		for protocol in protocols_covered.iter() {
			self.header_tree.merge(protocol, block_range)?;
		}
		Ok(())
	}

	fn get_time_range(protocols_covered: Vec<Protocol>, block_range: BlockRange) -> Result<Result<Vec<(Timestamp, Event)>>> {

	}
}
