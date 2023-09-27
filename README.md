# Cross-chain Asset Management Protocol - CCAMP

## Overview
CCAMP which is short for Cross chain asset management protocol, is a data based asset management protocol, which enables assets to be effectively managed and reallocated among several parties using a data driven approach.
It consists of smart contracts which live on the EVM blockchain, three canisters which communicate with each other in order to maintain a state which consists of balances and a data relayer which basically serves as a middle man between the smart contracts and canisters.

CCAMP is a general-purpose, modular, and custom data-driven Cross-chain Asset Management Protocol powered by:

- the Internet Computer for on-chain computation.
- the Log Store Network for highly-available and cryptographically pure event data.
- Web3 Functions (ie. Chainlink) for lean data relay.
- Locker Contracts (in Solidity for EVM, Rust, etc.)


## Pre-requisites
-   [ ] Download and [install the IC SDK](https://internetcomputer.org/docs/current/developer-docs/setup/index.md) if you do not already have it.

Alternatively you can run the following command at the root of the repo
`npm run setup` which would install the dfx cli tool if you are on Mac/Linux.


## Setting up
- [ ] Clone the repo.
- [ ] install the dependencies by running `yarn install` 

## Core components

### [Canisters](https://github.com/usherlabs/ccamp/tree/main/packages/canisters)
A detailed overview of rust canisters can be found [here](https://internetcomputer.org/docs/current/developer-docs/backend/rust/).
The canisters serve as the main point of interaction for users of the protocol. There are three canisters which serve as the backbone of the protocol and they are the remittance canister, the protocol data collection canister (PDC) and the data collection canister.

### [Smart Contracts](https://github.com/usherlabs/ccamp/tree/main/packages/contracts)
The smart contract consists of a [`Locker`](https://github.com/usherlabs/ccamp/blob/main/packages/contracts/contracts/Locker.sol) contract who's main purpose is to serve as a means to deposit funds into the protocol, withdraw funds from the protocol and cancel a pending withdrawal request made from the canister

### [Relayer](https://github.com/usherlabs/ccamp/tree/main/packages/relay)
The relayer is the middle man connecting the smart contracts to the PDC canister, it is powered by [gelato functions](https://docs.gelato.network/developer-services/web3-functions/writing-web3-functions).
The relayer mechanism serves as an indexer which queries events from the blockchain and publishes it to a logstore stream which via https. Logstore serves as a time series database, that can then be queried by the PDC via http to get the latest events published by the locker smart contract.

By [Usher Labs](https://usher.so)
