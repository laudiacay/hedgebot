use anyhow::Result;
use core::ops::Add;
use core::ops::Bound;
use core::ops::Sub;
use ranges::GenericRange;
use ranges::Ranges;
use serde::{Deserialize, Serialize};
use std::ops::RangeBounds;

pub type BlockNumber = u64;

#[derive(PartialEq)]
pub struct Blocks {
    ranges: Ranges<BlockNumber>,
}

impl Blocks {
    fn new(ranges: Ranges<BlockNumber>) -> Self {
        Blocks { ranges }
    }
    pub fn empty() -> Self {
        Blocks {
            ranges: Ranges::new(),
        }
    }

    pub fn union(&self, rhs: Blocks) -> Self {
        Blocks {
            ranges: self.ranges.union(rhs.ranges),
        }
    }
}

impl AsRef<Vec<GenericRange<BlockNumber>>> for Blocks {
    fn as_ref(&self) -> &Vec<GenericRange<BlockNumber>> {
        self.ranges.as_ref()
    }
}

impl Sub<Blocks> for Blocks {
    type Output = Blocks;
    fn sub(self, rhs: Blocks) -> Blocks {
        Blocks::new(self.ranges - rhs.ranges)
    }
}

impl Add<GenericRange<BlockNumber>> for Blocks {
    type Output = Blocks;
    fn add(self, rhs: GenericRange<BlockNumber>) -> Blocks {
        Blocks::new(self.ranges + rhs)
    }
}

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
            Bound::Unbounded => panic!("block ranges are always finite...."),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct StoreBlocks {
    serdeable_ranges: Vec<(MyBlockBound, MyBlockBound)>,
}

impl From<&Blocks> for StoreBlocks {
    // serde can eat me
    fn from(range: &Blocks) -> StoreBlocks {
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

impl From<StoreBlocks> for Blocks {
    // serde can eat me
    fn from(_range: StoreBlocks) -> Blocks {
        todo!();
        //range.as_ref().iter().map(|generic_range| (generic_range.start_bound().into(), generic_range.end_bound().into())).collect()
    }
}

impl TryFrom<&[u8]> for Blocks {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> Result<Blocks> {
        let sb: StoreBlocks = bincode::deserialize(&bytes)?;
        Ok(sb.into())
    }
}

impl TryInto<Vec<u8>> for &Blocks {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<Vec<u8>> {
        Ok(bincode::serialize(&StoreBlocks::from(self))?)
    }
}
