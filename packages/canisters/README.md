## ICP canisters
Welcome to your new canisters project and to the internet computer development community. By default, creating a new project adds this README and some template files to your project directory. You can edit these template files to customize your project and to include your own code to speed up the development cycle.

To get started, you might want to explore the project directory structure and the default configuration file. Working with this project in your development environment will not affect any production deployment or identity tokens.

To learn more before you start working with canisters, see the following documentation available online:

- [Quick Start](https://internetcomputer.org/docs/quickstart/quickstart-intro)
- [SDK Developer Tools](https://internetcomputer.org/docs/developers-guide/sdk-guide)
- [Rust Canister Devlopment Guide](https://internetcomputer.org/docs/rust-guide/rust-intro)
- [ic-cdk](https://docs.rs/ic-cdk)
- [ic-cdk-macros](https://docs.rs/ic-cdk-macros)
- [Candid Introduction](https://internetcomputer.org/docs/candid-guide/candid-intro)
- [JavaScript API Reference](https://erxue-5aaaa-aaaab-qaagq-cai.raw.icp0.io)

If you want to start working on your project right away, you might want to try the following commands:

```bash
cd canisters/
dfx help
dfx canister --help
```

## Running the project locally

If you want to test your project locally, you can use the following commands:

```bash
# Starts the replica, running in the background
dfx start --background

# Deploys your canisters to the replica and generates your candid interface
dfx deploy
```

Once the job completes, your application will be available at `http://localhost:4943?canisterId={asset_canister_id}`.

If you have made changes to your backend canister, you can generate a new candid interface with

```bash
npm run generate
```

at any time. This is recommended before starting the frontend development server, and will be run automatically any time you run `dfx deploy`.

If you are making frontend changes, you can start a development server with

```bash
npm start
```

Which will start a server at `http://localhost:8080`, proxying API requests to the replica at port 4943.


## [Canisters](https://github.com/usherlabs/ccamp/tree/main/packages/canisters)
A detailed overview of rust canisters can be found [here](https://internetcomputer.org/docs/current/developer-docs/backend/rust/).
The canisters serve as the main point of interaction for users of the protocol. There are three canisters which serve as the backbone of the protocol and they are the remittance canister, the protocol data collection canister (PDC) and the data collection canister.

#### Canisters Overview
**- Protocol Data Collection Canister**: This can be described as the "admin canister", it aggregates data about deposits, withdrawals and withdrawal cancelations from the smart contract's events and passes it onto the remittance canister.
**- Remittance canister**: This canister can be described as the "brain", it is the canister which stores the state of the protocol, which includes the balances of users across several tokens and chains. It is responsible for generating parameters which can be used to facilitate a withdrawal of allocated tokens from the smart contracts.
**- Data Collection Canister**: This canister serves as a "reallocator", it reallocates balances between users, it is the canister which is responsible for the reallocation/redistribution of assets across several users.
Note: The data collection canister can only reallocate balances which have already been created by the **PDC canister**

### Canisters Setup
`These commands should be run at the root of the canister folder.`

- [ ] `dfx start --clean` : This starts a local version of the internet computer's blockchain to which canisters can be deployed to.
- [ ] `dfx deploy` : This deploys an instance of all three canisters to the local instance of the blockchain that was started in the previous step. This step would output three different canister addresses/principals which we will use as placeholders for the rest of this documentation.
```
# sample canister deployment output
dc canister: bkyz2-fmaaa-aaaaa-qaaaq-cai
r canister: be2us-64aaa-aaaaa-qaabq-cai
pdc canister: bd3sg-teaaa-aaaaa-qaaba-cai
```
- [ ] Register the remittance canister principal to the PDC and DC canisters
```
dfx canister call --network ic protocol_data_collection set_remittance_canister '(principal "be2us-64aaa-aaaaa-qaabq-cai")'
dfx canister call --network ic data_collection set_remittance_canister '(principal "be2us-64aaa-aaaaa-qaabq-cai")'
```

- [ ] Register the PDC and DC canisters to remittance canister
```
dfx canister call remittance subscribe_to_dc '(principal "bkyz2-fmaaa-aaaaa-qaaaq-cai")'
dfx canister call remittance subscribe_to_pdc '(principal "bd3sg-teaaa-aaaaa-qaaba-cai")'
```
- [ ] Register the details of the data source, which is a Logstore Query URL and a Logstore Query Token as well as the timestamp to start querying logstore from, more information on logstore can be found [here](https://logstore.usher.so/), and a snippet of how to generate a query token can be found here
```
dfx canister call --network ic protocol_data_collection initialise_logstore '(0, "https://broker-eu-1.logstore.usher.so/streams/${STREAM_ID}", "SUPER_SECRET_QUERY_TOKEN")'
```
- [ ] Validate that the PDC and DC canisters are successfully registered with the remittance canister.
```
dfx canister call --network ic protocol_data_collection is_subscribed '(principal "be2us-64aaa-aaaaa-qaabq-cai")' dfx canister call --network ic data_collection is_subscribed '(principal "be2us-64aaa-aaaaa-qaabq-cai")'
```
If all previous steps have been completed then the canisters have been successfully setup and are ready for use.

### Canisters Commands
Note: The cli calls have the parameter `--network ic` to indicate they are for the main net, to run the commands against the local instance of the blockchain, the parameter and its value can be safely taken out.


#### Data Collection Canister
- Register a remittance canister to the DC canister
```
dfx canister call data_collection set_remittance_canister '(principal "be2us-64aaa-aaaaa-qaabq-cai")' --network ic

**parameters**
"be2us-64aaa-aaaaa-qaabq-cai": The address of the remittance canister.
```

- Manually publish event data to the registered remittance canister
```
dfx canister call data_collection manual_publish '[{"event_name":"BalanceAdjusted","canister_id":"bkyz2-fmaaa-aaaaa-qaaaq-cai","account":"0x9C81E8F60a9B8743678F1b6Ae893Cc72c6Bc6840","amount":-100000,"chain":"ethereum:5","token":"0xB24a30A3971e4d9bf771BDc81435c25EA69A445c"},{"event_name":"BalanceAdjusted","canister_id":"bkyz2-fmaaa-aaaaa-qaaaq-cai","account":"0x9C81E8F60a9B8743678F1b6Ae893Cc72c6Bc6840","amount":100000,"chain":"ethereum:5","token":"0xB24a30A3971e4d9bf771BDc81435c25EA69A445c"}]' --network ic

**Parameters**
A stringified json object following the above format, which represents an event that occured in the smart contract.
```

- Get the registered data collection canister.
```
dfx canister call data_collection get_remittance_canister --network ic
```

- Get if the remittance canister is successfully subscribed to the DC canister
```
dfx canister call data_collection is_subscribed '(principal "be2us-64aaa-aaaaa-qaabq-cai")' --network ic

**parameters*
"be2us-64aaa-aaaaa-qaabq-cai": The address of the remittance canister.
```

#### Protocol Data Collection Canister


- Register a remittance canister to the PDC.
```
dfx canister call protocol_data_collection set_remittance_canister '(principal "be2us-64aaa-aaaaa-qaabq-cai")' --network ic

**parameters**
"be2us-64aaa-aaaaa-qaabq-cai": The address of the remittance canister.
```

- whitelist a publisher to be able to push the PDC.
```
dfx canister call protocol_data_collection add_publisher '(principal "be2us-64aaa-aaaaa-qaabq-cai")' --network ic

**parameters**
"be2us-64aaa-aaaaa-qaabq-cai": The principal we want to whitelist to push events
```

- publish ethereum events and logstore validations
```
dfx canister call protocol_data_collection process_event '("{"source":{}, "validations":[]}")' --network ic

**parameters**
"be2us-64aaa-aaaaa-qaabq-cai": The principal we want to whitelist to push events
```

- Manually publish event data to the registered remittance canister.
```
dfx canister call protocol_data_collection manual_publish '[{"event_name":"FundsDeposited","canister_id":"bkyz2-fmaaa-aaaaa-qaaaq-cai","account":"0x9C81E8F60a9B8743678F1b6Ae893Cc72c6Bc6840","amount":100000,"chain":"ethereum:5","token":"0xB24a30A3971e4d9bf771BDc81435c25EA69A445c"}]' --network ic

**Parameters**
A stringified json object following the above format, which represents an event that occured in the smart contract.
```

- Set parameters of data source to fetch information from.
```
dfx canister call protocol_data_collection initialise_logstore '(0, "https://broker-us-1.logstore.usher.so/streams/${STREAM_ID}", "SUPER_SECRET_QUERY_TOKEN")' --network ic

**parameters**
"0": This is the starting timestamp, it is the timestamp from which we want to query data from the logstore for, it can be left as 0 for a new deployment.
"https://broker-us-1.logstore.usher.so/streams/${STREAM_ID}": This is the URL which would be polled for data in intervals of a minute, the "STREAM_ID" should be changed to the created stream id.
"SUPER_SECRET_QUERY_TOKEN": This is a token which is used to authenticate query requests to logstore.
```

- Start the poller which queries the provided logstore stream for the latest information every minute starting from events registered at the specified timestamp.
```
dfx canister call protocol_data_collection poll_logstore --network ic
```

- Stop the poller which queries the provided logstore stream for the latest information every minute starting from events registered at the specified timestamp.
```
dfx canister call protocol_data_collection halt_logstore_poll --network ic
```


- Get the registered data collection canister.
```
dfx canister call protocol_data_collection get_remittance_canister --network ic
```

- Manually trigger the process to fetch the latest data from logstore and push to the remittance canister
```
dfx canister call protocol_data_collection update_data --network ic
```

- Get the registered timestamp which we have fetched events up to.
```
dfx canister call protocol_data_collection last_queried_timestamp --network ic
```

- Get the registered query url for the data source
```
dfx canister call protocol_data_collection get_query_url --network ic
```

- Get the registered query token for the data source
```
dfx canister call protocol_data_collection get_query_token --network ic
```

- Get if the remittance canister is successfully subscribed to the PDC canister.
```
dfx canister call protocol_data_collection is_subscribed '(principal "be2us-64aaa-aaaaa-qaabq-cai")' --network ic

**parameters*
"be2us-64aaa-aaaaa-qaabq-cai": The address of the remittance canister.
```

#### Remittance Canister
- Get public key of remittance canister.
```
dfx canister call remittance public_key --network ic
```

- Subscribe to a data collection canister.
```
dfx canister call remittance subscribe_to_dc '(principal "bkyz2-fmaaa-aaaaa-qaaaq-cai")' --network ic

**parameters**
bkyz2-fmaaa-aaaaa-qaaaq-cai: Principal of the remittance canister
```

- Subscribe to a Protocol data collection canister.
```
dfx canister call remittance subscribe_to_pdc '(principal "bd3sg-teaaa-aaaaa-qaaba-cai")' --network ic

**parameters**
bd3sg-teaaa-aaaaa-qaaba-cai: Principal of the remittance canister
```

- Get the balance of an address.
```
dfx canister call remittance get_available_balance '("0xB24a30A3971e4d9bf771BDc81435c25EA69A445c","ethereum:5","0x1AE26a1F23E2C70729510cdfeC205507675208F2", principal "bkyz2-fmaaa-aaaaa-qaaaq-cai")' --network ic

**parameters**
"0xB24a30A3971e4d9bf771BDc81435c25EA69A445c": Address of the token which the user wants to check their balance of.
"0x1AE26a1F23E2C70729510cdfeC205507675208F2": Address of the user.
"ethereum:5": The Chain which the funds allocated to this user exists on.
"bkyz2-fmaaa-aaaaa-qaaaq-cai": The principal of the data collection canister responsible for managing funds of the user
```

- Get the balance of a data collection canister.
```
dfx canister call remittance get_canister_balance '("0xB24a30A3971e4d9bf771BDc81435c25EA69A445c","ethereum:5", principal "bkyz2-fmaaa-aaaaa-qaaaq-cai")' --network ic

**parameters**
"0xB24a30A3971e4d9bf771BDc81435c25EA69A445c": Address of the token.
"ethereum:5": The Chain which the funds allocated to this user exists on.
"bkyz2-fmaaa-aaaaa-qaaaq-cai": The principal of the data collection canister responsible for managing funds of the user
```

- Request a signature for withdrawal.
```
dfx canister call remittance remit '("0xB24a30A3971e4d9bf771BDc81435c25EA69A445c","ethereum:5","0x9C81E8F60a9B8743678F1b6Ae893Cc72c6Bc6840",principal "bkyz2-fmaaa-aaaaa-qaaaq-cai",100000,"0xc1f88bc447b9ab9783f25fb5e88c5eefec0b563e4a60316e007834b506490ed25b21d1d6827a5c965738aba8869d7ab08b6e7b9f4a6bce6cf0f3f577037d9fdb1c")' --network ic

**parameters**
"0xB24a30A3971e4d9bf771BDc81435c25EA69A445c": The address of the token.
"ethereum:5": The Chain which the funds allocated to this user exists on.
"0x9C81E8F60a9B8743678F1b6Ae893Cc72c6Bc6840": The address of the user.
"bkyz2-fmaaa-aaaaa-qaaaq-cai": The principal of the data collection canister responsible for managing funds of the user.
"100000": The amount to withdraw.
"0xc1f88bc447...": A signature of the amount to withdraw.
```

- Get a receipt for a valid withdrawal.
```
dfx canister call remittance get_reciept '(principal "bkyz2-fmaaa-aaaaa-qaaaq-cai", 12095196426242356980)' --network ic

**parameters**
"bkyz2-fmaaa-aaaaa-qaaaq-cai": The principal of the data collection canister responsible for managing funds of the user.
"12095196426242356980": The nonce provided when a withdrawal was requested.
```


# CCAMP Example: Cross-Chain Liquidity

## Overview

This is an application of the CCAMP protocol, an example demonstrating the seamless movement of liquidity between EVM Chains and the Internet Computer Protocol (ICP) using smart contracts and canisters.
In this example, we showcase a comprehensive solution for transferring liquidity across different blockchains, specifically between EVM Chains and ICP. The goal is to provide developers with a robust understanding of the mechanisms involved in managing deposits, bridging tokens, and facilitating withdrawals.

### Key Components

1. **Locker Contract:**
   Developers initiate the liquidity movement by depositing tokens into a smart contract which has been deployed and initialised on the Ethereum Virtual Machine (EVM). Events are emitted, capturing essential details such as the responsible DC (Data Center) canister and depositor information.

2. **Graph Indexer:**
 The Graph Indexer node plays a crucial role in indexing events emitted from the corresponding locker contract and storing it to a database.

3. **Relayer Indexer Node:**
  Indexed events stored in the postgres database are validated using the Log Store Network programs. When there are enough validations for a particular event, this node is responsible for pushing events and their validations which include ECDSA that can be verified by the ICP canisters.

4. **Protocol Data Collection canister:**
   The PDC canister is responsible for receiving and validating the events pushed by the Relayer Indexer node, and then updating the remittance canister which is responsible for maintaining the balance of each use and canister on the protocol.

5. **Bridge Data Collection canister:**
  Using this canister, one can either perform a *mint* operation after funds have been deposited into the protocol. Performing a mint operation would transfer some *ccMatic tokens* to the ICP account of the user performing the mint operation while taking ownership of their available funds on the protocol, and when a *burn* operation is performed, the locked funds on the protocol would be available to the user for withdrawal.

6. **Remittance canister:**
 Provided the user has some balance on the protocol, a request for withdrawal can be made. This request returns several parameters which can be used to facilitate a withdrawal from the locker contract.

### Internal Architecture Note
 Since the CCAMP Protocol is ERC20 token compatible, a token address might be mentioned in several places. When dealing with native tokens such as Ether, Matic or which ever token is native to the blockchain the locker contract is deployed on, the token address to be used should be the zero address which is:
`0x0000000000000000000000000000000000000000`


## Running the code

### Prerequisites

Before you begin, ensure you have the following:

- A deployed locker contract to an EVM network.
- A deployed Remittance and Protocol Data Collection canister.
- A running graph node responsible for indexing events in the locker contract.
- A running indexer relayer node linked to the graph node.

### Deploying and configuring the canisters
`Note: to deploy these canisters in production environment, add --network ic` to the end of the dfx commands. There are two canisters to be deployed which are the Bridge Data Collection canister and the Token canister.
The Bridge Data Collection canister is responsible for minting and burning tokens created by the Token canister using liquidity from the CCAMP protocol as collateral, and the token ccMatic is an IERC20 compliant token on the ICP blockchain.

#### Deploying the canisters
All relevant canisters can be deployed by calling the command
```
dfx deploy
```

#### Configuring the Token Canister
Configure the token canister we just deployed by calling a function to set an admin DC canister which is capable of minting and burning tokens.
```
dfx canister call token set_dc_canister '(principal "BRIDGE_DC_CANISTER_PRINCIPAL")'
```

#### Configuring the DC Canister
Configuration of the DC canister involves syncing it with the remittance canister and setting the value of the token canister we want to mint and burn from.

**Setting the principal of the token canister**
 ```
 dfx canister call bridge_data_collection set_token_principal '(principal "TOKEN_CANISTER_PRINCIPAL")'
```

**Syncing the remittance canister with the Bridge DC Canister**
```
dfx canister call data_collection set_remittance_canister '(principal "REMITTANCE_CANISTER_PRINCIPAL")'

dfx canister call remittance subscribe_to_dc '(principal "BRIDGE_DC_CANISTER_PRINCIPAL")'
```
**Validating the sync**
In order to validate the sync with the remittance canister was successfull, this function call should return `true`.
```
 dfx canister call bridge_data_collection is_subscribed '(principal "REMITTANCE_CANISTER_PRINCIPAL")'
```

### Step-by-Step Approach to using the canister

1. **Deposit on the EVM Chain :**

   Developers can deposit tokens on the EVM chain and mint ccMatic tokens by calling the `depositTokens` method

2. **Minting Tokens from the BDC(Bridge Data Collection) Canister:**
 After the deposit on ethereum has reflected on the remittance canister, they can then be used as collateral for minting ccMatic tokens by calling the mint method and specifying the amount to use as collateral to mint tokens.
   ```
   dfx canister call bridge_data_collection mint '("EVM_PUBLIC_ADDRESS",    "ECDSA_SIGNATURE_OF_AMOUNT_TO_MINT",AMOUNT_TO_MINT)' 
   ```

3. **Checking your balance on the token canister:**
 After tokens have been minted on the BDC canister, your token balance can be confirmed on the token canister by running the command
   ```
   dfx canister call token balance 
   ```

4. **Burning Tokens from the BDC(Bridge Data Collection) Canister:**
 Tokens can be burned and the collateral deposited would be returned to the balance of the user on the remittance canister, after which a withdrawal request can be made on the remittance canister.
   ```
   dfx canister call bridge_data_collection burn '("EVM_PUBLIC_ADDRESS",    "ECDSA_SIGNATURE_OF_AMOUNT_TO_BURN",AMOUNT_TO_BURN)'
   ```
  
5. **Requesting withdrawal from the remittance canister:**
The zero address is used to represent the native token of the chain the locker contract is on 
   ```
   dfx canister call remittance remit '("0x0000000000000000000000000000000000000000","CHAIN_ID","EVM_ADDRESS",principal "BRIDGE_DC_CANISTER_PRINCIPAL",WITHDRAWAL_AMOUNT,"ECDSA_SIGNATURE_OF_WITHDRAWAL_AMOUNT")'
   ```

6. **Withdrawal from the EVM Chain :**

   Developers can withdraw tokens on the EVM chain and get back their native token by calling the `withdrawTokens` method and providing the parameters returned by the `remit` call to the remittance canister, and the token would be withdrawn into the specified  wallet address.
  
  

## Next Steps

Congratulations! You've successfully moved liquidity across different blockchains using ICP and Ethereum. Explore further customization options or integrate this process into your own projects.
