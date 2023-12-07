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

<!-- new commands tobe documented -->
<!-- dfx canister call protocol_data_collection a
dd_publisher '(principal "qtyrj-5d7hy-yotni-4pnny-4vxak-pgha6-6gxas-7k52m-pqxkp-qa2oz-zqe")' -->