use crate::ingest_chain::Protocol;
use anyhow::{anyhow, Result};
use core::ops::Bound;
use ranges::{GenericRange, Ranges};
use serde::{Deserialize, Serialize};
use sled;
use std::collections::HashSet;
use std::ops::RangeBounds;

type BlockNumber = u64;

// FIXME reorg this file you degenerate

#[derive(Clone, Serialize, Deserialize)]
struct Timestamp {
    block_number: BlockNumber,
    // preserving my boyfriend's "code" here in a comment from when he "helped" me "work"
    // 	hack hack hack hack hack hack hack hack hack (hellow, owrld);
    // 	{hack ,hack hack} (hellow, world}
    // 		if ur are a noob
    // 		then l0l i h4xx u8:
    // }
    log_id: u64, // this is wrong and sucks and won't work. fix it later.
}

#[derive(Clone, Serialize, Deserialize)]
struct Event {
    protocol: Protocol,
    event_data: Vec<u8>, //sucks but works for now
}

type Blocks = Ranges<BlockNumber>;

#[derive(Serialize, Deserialize)]
enum MyBlockBound {
    Included(BlockNumber),
    Excluded(BlockNumber),
}

impl From<Bound<&BlockNumber>> for MyBlockBound {
    fn from(b: Bound<&BlockNumber>) -> MyBlockBound {
        match b.cloned() {
            Bound::Included(a) => MyBlockBound::Included(a),
            Bound::Excluded(a) => MyBlockBound::Excluded(a),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct StoreBlocks {
    serdeable_ranges: Vec<(MyBlockBound, MyBlockBound)>,
}

impl From<Ranges<BlockNumber>> for StoreBlocks {
    // serde can eat me
    fn from(range: Ranges<BlockNumber>) -> StoreBlocks {
        StoreBlocks {
            serdeable_ranges: range
                .as_ref()
                .iter()
                .map(|generic_range| {
                    (
                        generic_range.start_bound().into(),
                        generic_range.end_bound().into(),
                    )
                })
                .collect(),
        }
    }
}

impl From<StoreBlocks> for Ranges<BlockNumber> {
    // serde can eat me
    fn from(range: StoreBlocks) -> Ranges<BlockNumber> {
        todo!();
        //range.as_ref().iter().map(|generic_range| (generic_range.start_bound().into(), generic_range.end_bound().into())).collect()
    }
}

// TODO turn all the from_utf8 and serde from_str bullshit into something real at some point :|

fn process_single_event(
    db_out: core::result::Result<(sled::IVec, sled::IVec), sled::Error>,
) -> Result<(Timestamp, Event)> {
    let (timestamp_bytes, entry_bytes) = db_out?;
    let timestamp: Timestamp = serde_json::from_str(std::str::from_utf8(&timestamp_bytes)?)?;
    let event: Event = serde_json::from_str(std::str::from_utf8(&entry_bytes)?)?;
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
        handle.header_tree.set_merge_operator(|_, _, _| todo!());
        Ok(handle)
    }

    fn add_time_range(
        &self,
        protocols_covered: Vec<Protocol>,
        block_range: Blocks,
        events: Vec<(Timestamp, Event)>,
    ) -> Result<()> {
        // FIXME THIS IS NOT ATOMIC!!!! can fail at bad times in bad ways .... no good
        // FIXME make sure this short-circuits if needed.
        let mut batch = sled::Batch::default();
        for (ts, event) in events.iter() {
            batch.insert(
                str::as_bytes(&serde_json::to_string(&ts)?),
                str::as_bytes(&serde_json::to_string(&event)?),
            );
        }
        self.data_tree.apply_batch(batch)?;
        for protocol in protocols_covered.iter() {
            self.header_tree.merge(
                serde_json::to_string(&protocol)?,
                serde_json::to_string(&StoreBlocks::from(block_range))?,
            )?;
        }
        Ok(())
    }

    fn check_time_range(&self, protocol: Protocol, block_range: &Blocks) -> Result<bool> {
        match self.data_tree.get(serde_json::to_string(&protocol)?)? {
            None => Ok(false),
            Some(ranges_bytes) => {
                let old_ranges: StoreBlocks =
                    serde_json::from_str(std::str::from_utf8(&ranges_bytes[..])?)?;
                Ok(*block_range - Ranges::from(old_ranges) == Ranges::new())
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
