use crate::ingest_chain::blocks::BlockNumber;

use crate::ingest_chain::Protocol;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Timestamp {
    block_number: BlockNumber,
    // preserving my boyfriend's "code" here in a comment from when he "helped" me "work"
    // 	hack hack hack hack hack hack hack hack hack (hellow, owrld);
    // 	{hack ,hack hack} (hellow, world}
    // 		if ur are a noob
    // 		then l0l i h4xx u8:
    // }
    tx_id: u64, // this may not work. fix it later.
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Event {
    protocol: Protocol,
    event_data: Vec<u8>, //sucks but works for now
}
