# hedgebot

fast strategy backtester and fuzzer.

### so what should it do what do i need to build????

well first we need to figure out what exactly we're testing. 

so that's gonna involve a few contracts deployed to like a fork of mainnet or something (we like having, like, DAI, and stuff). 
 - need to have the ability to fork mainnet and easily pick out useful tokens (using foundry's ability to ENS and recognize popular tickers is useful here)
 - need to be able to take solidity directories like uniswap/v3-core or something, compile those using just a location of where they are on disk, and deploy them to our little fork at a given block in the simulation

and that's also going to be some sort of parsing and cleaning and storing of the transactions that interact with a given contract.
 - define what a given simulation needs out of a given contract. these might correspond to certain events emitted, but they're for like data analysis purposes. for uniswap, that might be a log of transaction size
 - we're going to need to sync the chain for this. run a transaction and see if it calls into uniswap, i suppose. then store it in a form where it can be repointed at an arbitrary subset-of-uniswap-interface "uniswap deployment"

fuzzing up and reapplying transactions. need to basically write a deserializer to turn a uniswap swap/lp/something into a data structure that captures its essence of what exactly the user did. then we need to write a reserializer that can turn it back into a new transaction after it's been mutated, either in terms of price or timing or in terms of what contract it's headed to.

we need some kind of queueable event loop/registry at different spots in the chain playback (deploy contract here, run this there, run this after each occurrence of x/y/z to get data or do a strategy thing). would some low-level component of tokio be useful for this, or ought we roll our own?

should be able to specify rules that should be called at given spots, as if we were running a strategy... i think this should be doing like a turn-based strategy game, i guess?

### starter roadmap...

two tasks to parallelize. these are like basic legos for the rest of the system. 
1. get a forked mainnet and deploy probably like uniswap v2, uniswap v3, and hegicv8888 to start testing against. have some kind of machine ready to test against. use foundry/forge as a library for this. have it working in a nice modular function- create_machine, set_machine_to_block, deploy_contract (returns address)... all of these probably exist directly in foundry. convenient! also figure out how to do configs. do we want to use the foundry config for this? that would be convenient :) @julian wanna do this
2. sync the chain- or subsets of it filtered by contracts that the transaction interacts with. figure out what is a sufficient level of abstraction for storing a uniswap transaction to disk in a nice mess-with-able form. @i can start here?