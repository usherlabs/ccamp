
# CCAMPClient Documentation

The `CCAMPClient` is a TypeScript class designed to streamline interactions with various CCAMP canisters, including the Remittance canister, the Data collection canister, and the Protocol data collection canister. This class enables users to engage with Data Collection, Protocol Data Collection, and Remittance canisters on the IC network using Ethereum-based private keys.

## Table of Contents

- [Installation](https://chat.openai.com/c/073de091-84c7-4ff6-aded-3955335ad8a3#installation)
- [Usage](https://chat.openai.com/c/073de091-84c7-4ff6-aded-3955335ad8a3#usage)
  - [Constructor](https://chat.openai.com/c/073de091-84c7-4ff6-aded-3955335ad8a3#constructor)
  - [getCanisterInstance](https://chat.openai.com/c/073de091-84c7-4ff6-aded-3955335ad8a3#getcanisterinstance)
  - [approveLockerContract](https://chat.openai.com/c/073de091-84c7-4ff6-aded-3955335ad8a3#approvelockercontract)
  - [deposit](https://chat.openai.com/c/073de091-84c7-4ff6-aded-3955335ad8a3#deposit)
  - [withdraw](https://chat.openai.com/c/073de091-84c7-4ff6-aded-3955335ad8a3#withdraw)

## Installation

Ensure you have the necessary dependencies installed:

```bash
npm install
```

## Usage

The `CCAMPClient` is versatile, supporting both development and production environments. This flexibility is achieved by providing an optional `options` parameter to the constructor. The `env` property within this parameter allows users to specify the environment, determining which network the CCAMP canisters will be instantiated on. By default, the environment is set to `prod` for production, but users can easily switch to `local` for development. Example usage:

### Constructor

```typescript
constructor(ethereumPrivateKey: string, options?: { env?: Environment })
```

- `ethereumPrivateKey`: Private key for the Ethereum account.
- `options.env`: Environment (default is `ENV.prod`). Options: `prod` or `local`.

```typescript
const ccampClient = new CCAMPClient('your_ethereum_private_key', { env: ENV.prod });
```

### getCanisterInstance

```typescript
getCanisterInstance(canisterType: CanisterType, overrides?: { canisterId?: string }): any
```

- `canisterType`: Type of the IC canister (e.g., `CANISTER_TYPES.DATA_COLLECTION`).
- `overrides.canisterId`: Override the default canister ID.

```typescript
const dataCollectionCanister = ccampClient.getCanisterInstance(CANISTER_TYPES.DATA_COLLECTION);
```

### approveLockerContract

Approve the locker contract to spend tokens on your behalf.

```typescript
approveLockerContract(erc20TokenAddress: string, amountToApprove: ethers.BigNumberish, signer: ethers.Wallet, overrides?: { lockerContract?: string }): Promise<any>
```

- `erc20TokenAddress`: Ethereum address of the ERC20 token.
- `amountToApprove`: Amount to approve for the locker contract.
- `signer`: Ethereum Wallet signer.
- `overrides.lockerContract`: Override the default locker contract address.

```typescript
await ccampClient.approveLockerContract('token_address', amount, signer);
```

### deposit

Deposit funds into the protocol.

```typescript
deposit(amount: ethers.BigNumberish, tokenAddress: string, signer: ethers.Wallet, overrides?: { lockerContract?: string; dcCanister?: string }): Promise<Transaction>
```

- `amount`: Amount to deposit.
- `tokenAddress`: Ethereum address of the token.
- `signer`: Ethereum Wallet signer.
- `overrides.lockerContract`: Override the default locker contract address.
- `overrides.dcCanister`: Override the default data collection canister ID.

```typescript
await ccampClient.deposit(amount, 'token_address', signer);
```

### withdraw

Withdraw funds from the network.

```typescript
withdraw(amount: ethers.BigNumberish, tokenAddress: string, chain: string ,signer: ethers.Wallet, overrides?: { lockerContract?: string; dcCanister?: string; remittanceCanister?: string }): Promise<Transaction>
```

- `amount`: Amount to withdraw.
- `tokenAddress`: Ethereum address of the token.
- `chain`: Blockchain identifier.
- `signer`: Ethereum Wallet signer.
- `overrides.lockerContract`: Override the default locker contract address.
- `overrides.dcCanister`: Override the default data collection canister ID.
- `overrides.remittanceCanister`: Override the default remittance canister ID.

```typescript
await ccampClient.withdraw(amount, 'token_address', signer, 'ethereum');
```

## Types

Below are the types used in the `CCAMPClient` class:

### Environment

```typescript
export type Environment = 'prod' | 'dev';
```

- `prod`: Production environment.
- `dev`: Development environment.

### CanisterType

```typescript
export type CanisterType = 'dataCollection' | 'protocolDataCollection' | 'remittance';
```

- `dataCollection`: Type for Data Collection canister.
- `protocolDataCollection`: Type for Protocol Data Collection canister.
- `remittance`: Type for Remittance canister.

### Canister Instances

```typescript
export type DataCollectionCanister;
export type ProtocolDataCollectionCanister;
export type RemittanceCanister;
```

- `DataCollectionCanister`: Type for Data Collection canister instance.
- `ProtocolDataCollectionCanister`: Type for Protocol Data Collection canister instance.
- `RemittanceCanister`: Type for Remittance canister instance.

These types are integral to the proper functioning of the `CCAMPClient.getCanisterInstance` method, providing a more specific interface for the different canister types.

#
This documentation provides a brief overview of the `CCAMPClient` class and its methods. Refer to the inline comments in the code for more detailed explanations of each method and its parameters.