# CCAMP Example: Cross-Chain Liquidity

  

## Overview

  

This is an application of the CCAMP protocol, an example demonstrating the seamless movement of liquidity between EVM Chains and the Internet Computer Protocol (ICP) using smart contracts and canisters.

In this example, we showcase a comprehensive solution for transferring liquidity across different blockchains, specifically between EVM Chains and ICP. The goal is to provide developers with a robust understanding of the mechanisms involved in managing deposits, bridging tokens, and facilitating withdrawals.

  

### Key Components

  

1.  **Locker Contract:**

Developers initiate the liquidity movement by depositing tokens into a smart contract which has been deployed and initialised on the Ethereum Virtual Machine (EVM). Events are emitted, capturing essential details such as the responsible DC (Data Center) canister and depositor information.

  

2.  **Graph Indexer:**

The Graph Indexer node plays a crucial role in indexing events emitted from the corresponding locker contract and storing it to a database.

  

3.  **Relayer Indexer Node:**

Indexed events stored in the postgres database are validated using the Log Store Network. The network publishes validations for each event to be validated to a stream. When there are enough validations for a particular event, this node is responsible for pushing events and their validations which include ECDSA signatures that can be verified by the ICP canisters to ensure the data has not been tampered with.

  

1.  **Protocol Data Collection canister:**

The PDC canister is responsible for receiving and validating the events pushed by the Relayer Indexer node, and then updating the remittance canister which is responsible for maintaining the balance of each use and canister on the protocol.

  

2.  **Bridge Data Collection canister:**

Using this canister, one can either perform a *mint* operation after funds have been deposited into the protocol. Performing a mint operation would transfer some *ccMatic tokens* to the ICP account of the user performing the mint operation while taking their funds on the CCAMP protocol as collateral, and when a *burn* operation is performed, the locked funds on the protocol would be available to the user for withdrawal.

  

1.  **Remittance canister:**

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

Note: to deploy these canisters in production environment, add `--network ic` to the end of the dfx commands.

  

There are two canisters to be deployed which are the Bridge Data Collection canister and the Token canister.

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
Configuration of the DC canister involves syncing it with the remittance canister and setting the principal of the token canister we want to mint and burn from.

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

  

1.  **Deposit on the EVM Chain :**

  

Developers can deposit tokens on the EVM chain and mint ccMatic tokens by calling the `depositTokens` method

  

2.  **Minting Tokens from the BDC(Bridge Data Collection) Canister:**

After the deposit on ethereum has reflected on the remittance canister, they can then be used as collateral for minting ccMatic tokens by calling the mint method and specifying the amount to use as collateral to mint tokens.

```
dfx canister call bridge_data_collection mint '("EVM_PUBLIC_ADDRESS", "ECDSA_SIGNATURE_OF_AMOUNT_TO_MINT",AMOUNT_TO_MINT)'
```

  

3.  **Checking your balance on the token canister:**

After tokens have been minted on the BDC canister, your token balance can be confirmed on the token canister by running the command

```
dfx canister call token balance
```

  

4.  **Burning Tokens from the BDC(Bridge Data Collection) Canister:**

Tokens can be burned and the collateral deposited would be returned to the balance of the user on the remittance canister, after which a withdrawal request can be made on the remittance canister.

```

dfx canister call bridge_data_collection burn '("EVM_PUBLIC_ADDRESS", "ECDSA_SIGNATURE_OF_AMOUNT_TO_BURN",AMOUNT_TO_BURN)'

```

5.  **Requesting withdrawal from the remittance canister:**

The zero address is used to represent the native token of the chain the locker contract is on

```
dfx canister call remittance remit '("0x0000000000000000000000000000000000000000","CHAIN_ID","EVM_ADDRESS",principal "BRIDGE_DC_CANISTER_PRINCIPAL",WITHDRAWAL_AMOUNT,"ECDSA_SIGNATURE_OF_WITHDRAWAL_AMOUNT")'
```

6.  **Withdrawal from the EVM Chain :**
Developers can withdraw tokens on the EVM chain and get back their native token by calling the `withdrawTokens` method and providing the parameters returned by the `remit` call to the remittance canister, and the token would be withdrawn into the specified wallet address.
  

## Next Steps

Congratulations! You've successfully moved liquidity across different blockchains using ICP and Ethereum. Explore further customization options or integrate this process into your own projects.