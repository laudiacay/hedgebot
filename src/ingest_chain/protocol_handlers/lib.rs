pub trait ProtocolStorageAdapter {
	// need to be able to take this from a txn, get the interesting data out, turn it back into a transaction to replay if needed.
	type ProtocolInternalStoredTransaction: Serialize + Deserialize;
	// this one returns which addresses are involved in a certain contract so we can watch them!
	fn mainnet_addresses_involved(&self) -> Vec<H160>;
	// need to be able to figure out when a given transaction touches a given contract
	// probably use foundry callgraph stuff here.
	fn is_hooked(&self, addresses_touched: Vec<H160>) -> bool;
	fn scrape_out_relevant_data(&self,  execution_info: forge::call_tracing::ExecutionInfo) -> ProtocolInternalStoredTransaction;
	fn dump_to_transaction(&self, ProtocolInternalStoredTransaction) -> ethers_rs::types::Transaction;
}