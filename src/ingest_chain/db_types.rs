use crate::ingest_chain::blocks::BlockNumber;

use crate::ingest_chain::Protocol;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Timestamp {
    block_number: BlockNumber,
    tx_id: u64, // this may not work. fix it later.
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Event {
    protocol: Protocol,
    event_data: Vec<u8>, //sucks but works for now
}
