# Cross-chain Asset Management Protocol - CCAMP

## 📘 Overview

CCAMP is an asset management protocol that leverages data-driven strategies to effectively manage and reallocate assets among various parties. This protocol encompasses a well-coordinated ecosystem, including smart contracts residing on EVM blockchains (and eventually non-EVM blockchains), a trio of interconnected canisters for state maintenance, and a crucial data relayer acting as an intermediary between the smart contracts and canisters.

At its core, CCAMP is a versatile, modular, and custom data-driven Cross-chain Asset Management Protocol, harnessing the following technologies and components:

- The Internet Computer: Providing the backbone for on-chain computation, ensuring the protocol's robust execution.
- The Log Store Network: Ensuring the availability of highly reliable and cryptographically pure event data, a critical element for informed asset management.
- Web3 Functions (e.g., Gelato): Facilitating streamlined data relay, connecting the protocol with external data sources.
- Locker Contracts: These contracts, available in Solidity for EVM and other compatible languages like Rust, play an essential role in asset management and protection across various blockchains.

CCAMP's flexibility, modularity, and data-driven approach make it an ideal choice for those seeking efficient and secure asset management solutions across multiple blockchain networks. It seamlessly combines the power of the Internet Computer, Log Store Network, Web3 Functions, and Locker Contracts to create a cutting-edge protocol for cross-chain liquidity aggregation management.

### 👉 Learn more

#### Overview

- [View the Introduction](https://youtu.be/R-mPl4T_ch8)
- [View Architecture](https://github.com/usherlabs/ccamp/tree/main/assets/CCAMP-Architecture-Simple.jpeg)
- [Read the Announcement](https://forum.dfinity.org/t/introducing-ccamp-unlocking-cross-chain-defi-aggregation-on-the-internet-computer/24643)

#### Codebase

- [Step 1: Deposit](https://www.loom.com/share/18d55367509c4823bf4784ce09ed92d7?sid=423b85b8-f2b0-4500-964c-3fb247ec6491)
- [Step 2: Withdrawal](https://www.loom.com/share/90386c85e08e4128ab21ea84a76f9935?sid=86cd63f5-5bd5-4fe2-a938-cded7747c4cf)
- [Step 3: Re-allocate](https://www.loom.com/share/fdc5081b9e4a49e9afae4aaa7825b927?sid=9ea5c31d-7b43-490f-8c03-7585efbc4f79)
- [A walkthrough of the Canister Code](https://www.loom.com/share/89935ad79a9f4c079bfffd10861afb23?sid=9459291c-71d4-437b-9fc5-e5b7137265f5)

### 🚙 Roadmap

The development of CCAMP is a journey marked by continuous enhancement and innovation. Here's a glimpse into our future plans and how we intend to evolve the protocol:

- [ ] **Enhancing Data Relay**  
       Our current architecture employs Web3 Functions for a lean data relay from blockchain sources to CCAMP. This proof-of-concept approach aggregates events from diverse blockchains, however, is cyclical due to CRON execution. Our plan is to transition to a relayer network model. In this phase, each relayer will reference its own RPC nodes and blockchain gateways. This upgrade aims to ensure the confirmation of events originating from source blockchains derive from disparate gateways, decreasing risk of centralisation and enhancing data reliability and security.

- [ ] **Near Real-Time Data Ingestion**  
       We are committed to enabling near real-time DeFi capabilities within CCAMP. To achieve this, we plan to replace on-chain calls to the Log Store with a dedicated ingestion node optimised for high-frequency data ingestion. This transformation empowers CCAMP to operate as swiftly as incoming data, ushering in a new era of near real-time DeFi. The ingestion node will also support bespoke data protocols designed for Data Collection (DC) Canisters, allowing CCAMP to manage assets based on real-time real-world data.

- [ ] **Account Reconciliation**  
       As we seek to enhance the speed and efficiency of CCAMP, we recognise the limitations posed by block confirmation wait periods for every Locker Contract interaction. Our ongoing research and development efforts focus on account reconciliation and the introduction of a mechanism to flush Canister historic state transitions to the Log Store. This innovative approach will allow all incoming deposits to immediately influence the protocol's aggregated liquidity, while withdrawals will still await the block confirmation period.

- [ ] **Diverse Blockchain Compatibility**  
       At Usher Labs, we believe in inclusivity. Our future roadmap includes expanding the reach of Locker Contracts to support a diverse array of blockchains. While Locker Contracts currently cater to Solidity/EVM blockchains, we are eager to embrace improvsed diversity in cross-blockchain compatibility. If you have the expertise to contribute Locker Contracts designed for various blockchains, your efforts will be highly valued, and your contributions will be recognised within this repository and the wider Usher Labs Community.

The CCAMP roadmap is a testament to our dedication to advancing the capabilities of this innovative protocol. [Join us on this journey](https://go.usher.so/discord), and together, we can shape the future of decentralised asset management and data-driven asset reallocation.

## 🚀 Get Started

CCAMP empowers developers to create and register [Data Collection (DC) Canisters](https://github.com/usherlabs/ccamp/tree/main/packages/canisters/src/data_collection), fostering modularity in the system. These DC Canisters play a crucial role by accepting bespoke data from sources like Log Store or user inputs, and subsequently reallocating assets based on this input data. Upon deployment, each Data Collection Canister is assigned a unique identifier.

**Managing Liquidity**: To ensure the orderly flow of assets within the protocol, deposits and withdrawals with Locker contracts must reference this unique identifier, ensuring that liquidity is accurately assigned to the relevant DC Canister. This practice safeguards against DC Canisters inadvertently affecting liquidity across the entire protocol.

**Investor Liquidity Management**: Each DC Canister is responsible for managing the liquidity allocated to it by investors, providing an organised approach to asset reallocation.

Ready to create your own DC Canister and contribute to the CCAMP ecosystem? Here's how to get started:

1. **Create Your DC Canister:** Develop and deploy your Data Collection Canister, aligning it with your specific data needs.
2. **Register Your Canister:** Once your DC Canister is up and running, contact the Usher Labs team for the registration process. This step integrates your canister into the core CCAMP ecosystem, allowing it to function seamlessly alongside other protocol components.

Embark on your CCAMP journey, foster data-driven asset reallocation, and engage with the Usher Labs team for support and integration. Together, we can advance the capabilities of this innovative protocol.

Ready to explore the potential of CCAMP? [Join us on Discord](https://go.usher.so/discord).

### ⚠️ Pre-requisites

- [ ] Download and [install the IC SDK](https://internetcomputer.org/docs/current/developer-docs/setup/index.md) if you do not already have it.

Alternatively you can run the following command at the root of the repo `npm run setup` which would install the dfx cli tool if you are on Mac/Linux.

### ⚙️ Setting up

- [ ] Clone the repo.
- [ ] install the dependencies by running `yarn install`

## 📕 Core components

### [Canisters](https://github.com/usherlabs/ccamp/tree/main/packages/canisters)

For an in-depth exploration of Rust canisters, refer to the documentation available [here](https://internetcomputer.org/docs/current/developer-docs/backend/rust/). The canisters play a pivotal role as the primary interface for protocol users, with three core canisters forming the backbone of CCAMP. These are the Remittance Canister, the Protocol Data Collection Canister (PDC), and the Data Collection Canister.

### [Smart Contracts](https://github.com/usherlabs/ccamp/tree/main/packages/contracts)

The smart contract component features the `Locker` contract. Its primary purpose is to facilitate fund deposits into the protocol, fund withdrawals, and the cancellation of pending withdrawal requests initiated from the canister.

### [Relayer](https://github.com/usherlabs/ccamp/tree/main/packages/relay)

The relayer acts as the vital intermediary that links the smart contracts to the PDC canister. It is empowered by [Gelato Web3 Functions](https://docs.gelato.network/developer-services/web3-functions/writing-web3-functions). This relayer mechanism operates as an event indexer, querying blockchain events and publishing them to a Log Store stream via HTTPS. Log Store serves as a time-series database, enabling the PDC to retrieve the latest events published by the Locker smart contract through HTTP.

## Licensing

The primary license for the CCAMP is the Business Source License 1.1 (BUSL-1.1), see LICENSE. However, some files are dual licensed under GPL-2.0-or-later:

- The `packages/relay/` is licensed under GPL-2.0-or-later.

## Contributors

- [Usher Labs](https://usher.so)
