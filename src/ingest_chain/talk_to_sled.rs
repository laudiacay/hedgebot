use super::blocks::*;
use super::db_types::*;
use crate::ingest_chain::Protocol;
use anyhow::Result;
use ranges::GenericRange;
use sled;
use std::collections::HashSet;

// FIXME reorg this file you degenerate

// TODO turn all the from_utf8 and serde from_str bullshit into something real at some point :|

fn process_single_event(
    db_out: core::result::Result<(sled::IVec, sled::IVec), sled::Error>,
) -> Result<(Timestamp, Event)> {
    let (timestamp_bytes, entry_bytes) = db_out?;
    let timestamp: Timestamp = bincode::deserialize(&timestamp_bytes)?;
    let event: Event = bincode::deserialize(&entry_bytes)?;
    Ok((timestamp, event))
}

/// this is the tree key for the headers tree, which tells us which data is stored in the sled database.
/// the structure of the DB is as follows. there's a headers tree, which has keys of protocol IDs, and RangeSets of blocks that these protocols are synced to disk for
/// then there's another sled tree for all the protocols' events during the relevant ranges.
/// these store timestamped events, meant to be iterated over. they're timestamped by block and then an index of the event inside the block.
/// this event timestamp is the key in the protocol btree- you iterate over them in order, obviously.
/// and the data inside (the event) contains a descriptor of which protocol the event pertains to, as well as the event data for you to do what you want with.
const HEADERS_TREE_KEY: &[u8] = b"HEADERS_TREE";
const DATA_TREE_KEY: &[u8] = b"DATA_TREE";

struct SledHandle {
    db: sled::Db,
    header_tree: sled::Tree,
    data_tree: sled::Tree,
}

// this can fail. what should i do about that?
fn range_merge(_key: &[u8], old_range: Option<&[u8]>, new_range: &[u8]) -> Option<Vec<u8>> {
    match old_range {
        None => Some(new_range.to_vec()),
        Some(old_range) => {
            let old_range = Blocks::try_from(old_range).unwrap();
            let new_range = Blocks::try_from(new_range).unwrap();

            let both = old_range.union(new_range);
            let bytes: Vec<u8> = (&both).try_into().unwrap();

            Some(bytes)
        }
    }
}

impl SledHandle {
    /// opens/creates the database at the requested sled path. ensures all requested protocols are in the headers table.
    fn new(sled_path: &str) -> Result<Self> {
        let db = sled::open(sled_path)?;
        let handle = SledHandle {
            db,
            header_tree: db.open_tree(HEADERS_TREE_KEY)?,
            data_tree: db.open_tree(DATA_TREE_KEY)?,
        };
        // this should set header tree merge to be the rangemap merge
        handle.header_tree.set_merge_operator(range_merge);
        Ok(handle)
    }

    fn add_time_range(
        &self,
        protocols_covered: Vec<Protocol>,
        block_range: &Blocks,
        events: Vec<(Timestamp, Event)>,
    ) -> Result<()> {
        // FIXME THIS IS NOT ATOMIC!!!! can fail at bad times in bad ways .... no good
        // FIXME make sure this short-circuits if needed.
        let mut batch = sled::Batch::default();
        for (ts, event) in events.iter() {
            batch.insert(bincode::serialize(&ts)?, bincode::serialize(&event)?);
        }
        self.data_tree.apply_batch(batch)?;
        for protocol in protocols_covered.iter() {
            let block_bytes: Vec<u8> = block_range.try_into()?;

            self.header_tree
                .merge(bincode::serialize(&protocol)?, block_bytes)?;
        }
        Ok(())
    }

    fn check_time_range(&self, protocol: Protocol, block_range: &Blocks) -> Result<bool> {
        match self.data_tree.get(bincode::serialize(&protocol)?)? {
            None => Ok(false),
            Some(ranges_bytes) => {
                Ok(*block_range - Blocks::try_from(&ranges_bytes[..])? == Blocks::empty())
            }
        }
    }

    // FIXME: this is horrendous and clones absolutely everywhere. fix it.
    fn get_contiguous_time_range(
        &self,
        protocol_filter: HashSet<Protocol>,
        block_range: GenericRange<BlockNumber>,
    ) -> Result<Vec<(Timestamp, Event)>> {
        Ok(block_range
            .into_iter()
            .map(|block| {
                self.data_tree
                    .scan_prefix(block.to_be_bytes())
                    .map(|x| process_single_event(x))
                    .collect::<Result<Vec<(Timestamp, Event)>>>()
            })
            .collect::<Result<Vec<Vec<(Timestamp, Event)>>>>()?
            .concat())
    }

    /// outer Result is for general errors while doing the thing.
    /// inner option is for "ya dun goofed, events weren't ingested for this time range first"
    // FIXME: this is horrendous and clones absolutely everywhere. fix it. you are SHAMELESS

    // FIXME this doesnt filter protocols
    pub fn get_time_range(
        &self,
        protocol_filter: HashSet<Protocol>,
        block_ranges: &Blocks,
    ) -> Result<Option<Vec<(Timestamp, Event)>>> {
        // first, check real quick if anything's not on disk! then we should go get it- don't want any incomplete data.
        for protocol in protocol_filter.iter() {
            if !self.check_time_range(*protocol, block_ranges)? {
                return Ok(None);
            }
        }
        // next iterate over all of the blocks and all of their event timestamps in the range
        Ok(Some(
            block_ranges
                .as_ref()
                .into_iter()
                .map(|block_range| self.get_contiguous_time_range(protocol_filter, *block_range))
                .collect::<Result<Vec<Vec<(Timestamp, Event)>>>>()?
                .concat(),
        ))
    }
}
