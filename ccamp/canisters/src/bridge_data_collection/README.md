# CCAMP: Cross-Chain Liquidity Bridge

## Overview

This example illustrates the CCAMP protocol's application in enabling seamless liquidity movement between EVM Chains and the Internet Computer Protocol (ICP). It provides a detailed process for transferring liquidity across these blockchains.

### Key Components

1. **Locker Contract**:
   Initiates liquidity transfer by depositing tokens into an EVM-deployed smart contract. Emits events detailing depositor and DC canister information.
2. **Relayer Indexer Node**:
   Doubles as a Indexer and Relayer. Indexes events from the Locker contract for storage in a database. Validates stored events using the Log Store Network, pushing validated events with ECDSA signatures to ICP canisters.
3. **Protocol Data Collection (PDC) Canister**:
   Receives and validates events from the Relayer Node, updating the remittance canister which maintains protocol balances.
4. **Bridge Data Collection Canister**:
   Allows `mint` operations (transferring `ccMatic` tokens to users’ ICP accounts) and `burn` operations (enabling fund withdrawal).
5. **Remittance Canister**:
   Facilitates withdrawals based on protocol balances, providing necessary transaction parameters.

### Highlight: Bridge Data Collection Canister & ICRC Token

The Bridge Data Collection (BDC) Canister in the CCAMP protocol is a custom-built DC Canister, specifically designed to govern the management of an ICRC token (ccMATIC) in response to new deposits and withdrawals. The BDC Canister **acts as a reference model for developers**, illustrating asset and liquidity management within the CCAMP framework. Its primary function is to mint a counterparty or derivative token on the Internet Computer when liquidity is provided. This enables the use of the ICRC counterpart in DeFi protocols across the Internet Computer ecosystem. When bridging the ICRC token back to the original chain, or a compatible chain like Polygon, the BDC manages the token burn process and updates CCAMP accordingly, ensuring users can withdraw their initial deposits in proportion to the burnt tokens.

### Internal Architecture Note

Since the CCAMP Protocol is ERC20 token compatible, a token address might be mentioned in several places. When dealing with native tokens such as Ether, Matic or which ever token is native to the blockchain the Locker contract is deployed on, the token address to be used should be the zero address which is:

`0x0000000000000000000000000000000000000000`
