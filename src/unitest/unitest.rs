/// Basic components for deploying and working with a forked mainnet
use evm_adapters::{
    Evm,
    evm_opts::{Env, EvmOpts, EvmType},
    sputnik::{
        cheatcodes::{
            backend::CheatcodeBackend,
            memory_stackstate_owned::MemoryStackStateOwned,
        },
		helpers::{VICINITY, TestSputnikVM},
		SputnikExecutor, Executor, PRECOMPILES_MAP,
	},
};
use ethers::types::{Address, Bytes, U256};
use eyre::Result;
use sputnik::{
	ExitReason, Config,
	backend::MemoryBackend,
    executor::stack::StackState,
};
use std::marker::PhantomData;

// FIXME no idea what the significance of this is or what it should be
static CFG: Config = Config::london();

/// Interface to a forked mainnet that allows us to run transactions and 
/// query the EVM state
/// Right now this is essentially a wrapper over foundry's evm_adapters 
/// interface, but we may add more functionality as time goes on
pub struct Machine<S,A> {
    evm: A,
    phantom: PhantomData<S>,
}

// Constructors for Sputnik VM Forks, may need to reimplement these as more EVM
// implementations are supported by evm-adapters
impl<'a, S: StackState<'a>> Machine<S, TestSputnikVM<'a, MemoryBackend<'a>>> {
    pub fn new_sputnik_evm_from_opts(opts: EvmOpts) -> Self {
		let mut backend = MemoryBackend::new(&*VICINITY, Default::default());
		// max out the balance of the faucet
        let faucet_account = Address::from_slice(&ethers::utils::keccak256("turbodapp faucet")[12..]);
        let faucet = backend.state_mut().entry(faucet_account).or_insert_with(Default::default);
        faucet.balance = U256::MAX;

        let executor = Executor::new_with_cheatcodes(
            backend,
            opts.env.gas_limit,
            &CFG,
            &*PRECOMPILES_MAP,
            opts.ffi,
            opts.verbosity > 2,
            opts.debug,
        );

        Self{ evm: executor, phantom: PhantomData }
    }

    /// Create a new Fork using SputnikVM and the default options
    pub fn new_sputnik_evm() -> Self {
        // TODO should set verbosity field to >2 here to enable tracing
        let opts = EvmOpts {
            env: Env { gas_limit: 18446744073709551615, chain_id: Some(99), ..Default::default() },
            initial_balance: U256::MAX,
            evm_type: EvmType::Sputnik,
            ..Default::default()
        };
        Self::new_sputnik_evm_from_opts(opts)
    }

    pub fn new_sputnik_evm_from_fork_url(url: String) -> Self {
        // TODO should set verbosity field to >2 here to enable tracing
        let opts = EvmOpts {
            env: Env { gas_limit: 18446744073709551615, chain_id: Some(99), ..Default::default() },
            initial_balance: U256::MAX,
            evm_type: EvmType::Sputnik,
            fork_url: Some(url),
            ..Default::default()
        };
        Self::new_sputnik_evm_from_opts(opts)
    }

    // TODO
    pub fn new_sputnik_evm_from_timestamp() -> Self {
        todo!("look at EvmOpts fields, or figure out a way to generate a nice fork_url")
    }
}

impl<'a, S, A: Evm<S>> Machine<S,A> {
    /// Deploy a contract
    pub fn deploy_contract(&mut self, from: Address, calldata: Bytes, value: U256) 
        -> Result<(Address, <A as Evm<S>>::ReturnReason, u64 /*gas used*/, Vec<String>)>
    {
        self.evm.deploy(from, calldata, value)
    }
}
