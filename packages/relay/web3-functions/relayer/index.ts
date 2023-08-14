import { Contract, EventFilter, ethers } from "ethers";
import {
  Web3Function,
  Web3FunctionContext,
} from "@gelatonetwork/web3-functions-sdk";
import contractAddresses from "@ccamp/contracts/address.json";
import { abi as LockerABI } from "@ccamp/contracts/artifacts/contracts/Locker.sol/Locker.json";
import { BLOCK_STORAGE_KEY, START_BLOCK_NUM } from "./utils/constants";
import { mapEvent } from "./utils/functions";
import { LogStoreClient, CONFIG_TEST } from "@logsn/client";

const MAX_RANGE = 100; // limit range of events to comply with rpc providers
const MAX_REQUESTS = 9; // limit number of requests on every execution to avoid hitting timeout

Web3Function.onRun(async (context: Web3FunctionContext) => {
  const { userArgs, storage, multiChainProvider, secrets } = context;
  const evmPk = await secrets.get("EVM_PK");

  const { startBlock: defaultStartBlock, streamId } = userArgs;

  // get the details about the contract
  const provider = multiChainProvider.default();
  const { chainId } = await provider.getNetwork();
  const contractAddress =
    contractAddresses[String(chainId) as keyof typeof contractAddresses]
      .lockerContractAddress;
  const lockerContract = new Contract(contractAddress, LockerABI, provider);

  // get the last processed block
  let lastProcessedBlock = parseInt(
    (await storage.get(BLOCK_STORAGE_KEY)) ?? `${defaultStartBlock}`
  );
  console.log(`Last processed block: ${lastProcessedBlock}`);

  // fetch the current block
  const currentBlock = (await provider.getBlockNumber()) - 1;

  // Fetch recent logs in range of 100 blocks
  const filter = [
    lockerContract.filters.FundsDeposited(),
    lockerContract.filters.FundsWithdrawn(),
    lockerContract.filters.WithdrawCanceled(),
  ] as EventFilter;

  let nbRequests = 0;
  let totalEvents: ethers.Event[] = [];
  while (lastProcessedBlock < currentBlock && nbRequests < MAX_REQUESTS) {
    nbRequests++;
    const fromBlock = lastProcessedBlock + 1;
    const toBlock = Math.min(fromBlock + MAX_RANGE, currentBlock);

    console.log(`Fetching log events from blocks ${fromBlock} to ${toBlock}`);

    try {
      const events = await lockerContract.queryFilter(
        filter,
        fromBlock,
        toBlock
      );
      totalEvents = [...totalEvents, ...events];
      lastProcessedBlock = toBlock;
    } catch (err) {
      return {
        canExec: false,
        message: `Rpc call failed: ${(err as Error).message}`,
      };
    }
  }

  // push the events to the predefied stream in logstore
  const parsedEvent = totalEvents.map(mapEvent);
  console.log(
    `${parsedEvent.length} events were fetched in total to block:${lastProcessedBlock}  `
  );

  if (parsedEvent.length === 0) {
    return {
      canExec: false,
      message: `No new event found`,
    };
  }

  console.log(parsedEvent);

  // TODO use an http interface to publish events to a stream
  // const logStoreClient = new LogStoreClient({
  //   ...CONFIG_TEST,
  //   auth: {
  //     privateKey: String(evmPk),
  //   },
  // });
  // // create the stream
  // const stream = await logStoreClient.getOrCreateStream({
  //   id: String(streamId),
  // });

  // const response = await stream.publish(parsedEvent);

  // Update storage for next run
  await storage.set(BLOCK_STORAGE_KEY, lastProcessedBlock.toString());
  return {
    canExec: true,
    callData: [],
  };
});
