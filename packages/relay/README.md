## [Relayer](https://github.com/usherlabs/ccamp/tree/main/packages/relay)
The relayer is the middle man connecting the smart contracts to the PDC canister, it is powered by [gelato functions](https://docs.gelato.network/developer-services/web3-functions/writing-web3-functions).
The relayer mechanism serves as an indexer which queries events from the blockchain and publishes it to a logstore stream which via https. Logstore serves as a time series database, that can then be queried by the PDC via http to get the latest events published by the locker smart contract.

#### Running the relayer
The relayer can be run locally by running the command
`npm run run-relayer:goerli`

#### Deploying the relayer
The relayer can be deployed by running
`npm run deploy-relayer`



## More on Web3 Function <!-- omit in toc -->

Use this template to write, test and deploy Web3 Functions.

### What are Web3 Functions?

Web3 Functions are decentralized cloud functions that work similarly to AWS Lambda or Google Cloud, just for web3. They enable developers to execute on-chain transactions based on arbitrary off-chain data (APIs / subgraphs, etc) & computation. These functions are written in Typescript, stored on IPFS and run by Gelato.

### Documentation

You can find the official Web3 Functions documentation [here](https://docs.gelato.network/developer-services/web3-functions).

### Private Beta Restriction

Web3 Functions are currently in private Beta and can only be used by whitelisted users. If you would like to be added to the waitlist, please reach out to the team on [Discord](https://discord.com/invite/ApbA39BKyJ) or apply using this [form](https://form.typeform.com/to/RrEiARiI).

### Table of Content

- [Relayer](#relayer)
    - [Running the relayer](#running-the-relayer)
    - [Deploying the relayer](#deploying-the-relayer)
  - [What are Web3 Functions?](#what-are-web3-functions)
  - [Documentation](#documentation)
  - [Private Beta Restriction](#private-beta-restriction)
  - [Table of Content](#table-of-content)
  - [Project Setup](#project-setup)
  - [Hardhat Config](#hardhat-config)
  - [Write a Web3 Function](#write-a-web3-function)
  - [Test your web3 function](#test-your-web3-function)
    - [Calling your web3 function](#calling-your-web3-function)

### Project Setup

1. Install project dependencies

```
yarn install
```

2. Configure your local environment:

- Copy `.env.example` to init your own `.env` file

```
cp .env.example .env
```

- Complete your `.env` file with your private settings

```
ALCHEMY_ID=
PRIVATE_KEY=
```

### Hardhat Config

In `hardhat.config.ts`, you can set up configurations for your Web3 Function runtime.

  - `rootDir`: Directory which contains all web3 functions directories.
  - `debug`: Run your web3 functions with debug mode on.
  - `networks`: Provider of these networks will be injected into web3 function's multi chain provider.

```ts
  w3f: {
    rootDir: "./web3-functions",
    debug: false,
    networks: ["mumbai", "goerli", "baseGoerli"],
  },
```

### Write a Web3 Function

- Go to `web3-functions/index.ts`
- Write your Web3 Function logic within the `Web3Function.onRun` function.
- Example:

```typescript
import {
  Web3Function,
  Web3FunctionContext,
} from "@gelatonetwork/web3-functions-sdk";
import { Contract } from "@ethersproject/contracts";
import ky from "ky"; // we recommend using ky as axios doesn't support fetch by default

const ORACLE_ABI = [
  "function lastUpdated() external view returns(uint256)",
  "function updatePrice(uint256)",
];

Web3Function.onRun(async (context: Web3FunctionContext) => {
  const { userArgs, gelatoArgs, multiChainProvider } = context;

  const provider = multiChainProvider.default();

  // Retrieve Last oracle update time
  const oracleAddress = "0x71B9B0F6C999CBbB0FeF9c92B80D54e4973214da";
  const oracle = new Contract(oracleAddress, ORACLE_ABI, provider);
  const lastUpdated = parseInt(await oracle.lastUpdated());
  console.log(`Last oracle update: ${lastUpdated}`);

  // Check if it's ready for a new update
  const nextUpdateTime = lastUpdated + 300; // 5 min
  const timestamp = (await provider.getBlock("latest")).timestamp;
  console.log(`Next oracle update: ${nextUpdateTime}`);
  if (timestamp < nextUpdateTime) {
    return { canExec: false, message: `Time not elapsed` };
  }

  // Get current price on coingecko
  const currency = "ethereum";
  const priceData: any = await ky
    .get(
      `https://api.coingecko.com/api/v3/simple/price?ids=${currency}&vs_currencies=usd`,
      { timeout: 5_000, retry: 0 }
    )
    .json();
  price = Math.floor(priceData[currency].usd);
  console.log(`Updating price: ${price}`);

  // Return execution call data
  return {
    canExec: true,
    callData: [{to: oracleAddress, data: oracle.interface.encodeFunctionData("updatePrice", [price])}],
  };
});
```

- Each Web3 Function has a `schema.json` file to specify the runtime configuration. In later versions you will have more optionality to define what resources your Web3 Function requires.

```json
{
  "web3FunctionVersion": "2.0.0",
  "runtime": "js-1.0",
  "memory": 128,
  "timeout": 30,
  "userArgs": {}
}
```

### Test your web3 function

#### Calling your web3 function

- Use `npx hardhat w3f-run W3FNAME` command to test your function (replace W3FNAME with the folder name containing your web3 function)

- Options:

  - `--logs` Show internal Web3 Function logs
  - `--debug` Show Runtime debug messages
  - `--network [NETWORK]` Set the default runtime network & provider. 

If your web3 function has arguments, you can define them in [`hardhat.config.ts`](./hardhat.config.ts).

Example:<br/> `npx hardhat w3f-run oracle --logs`

Output:

```
Web3Function building...

Web3Function Build result:
✓ Schema: web3-functions/examples/oracle/schema.json
✓ Built file: ./.tmp/index.js
✓ File size: 2.46mb
✓ Build time: 255.66ms

Web3Function user args validation:
✓ currency: ethereum
✓ oracle: 0x71B9B0F6C999CBbB0FeF9c92B80D54e4973214da

Web3Function running...

Web3Function Result:
✓ Return value: { canExec: false, message: 'Rpc call failed' }

Web3Function Runtime stats:
✓ Duration: 0.41s
✓ Memory: 65.65mb
✓ Rpc calls: 2
```