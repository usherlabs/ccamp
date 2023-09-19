import { Contract, EventFilter, ethers } from "ethers";
import {
  Web3Function,
  Web3FunctionContext,
} from "@gelatonetwork/web3-functions-sdk";
import contractAddresses from "@ccamp/contracts/address.json";
import { abi as LockerABI } from "@ccamp/contracts/artifacts/contracts/Locker.sol/Locker.json";
import { BLOCK_STORAGE_KEY } from "./utils/constants";
import { mapEvent, publishEvent } from "./utils/functions";
import ky from "ky";

const MAX_RANGE = 100; // limit range of events to comply with rpc providers
const MAX_REQUESTS = 4; // limit number of requests on every execution to avoid hitting timeout

Web3Function.onRun(async (context: Web3FunctionContext) => {
  const { userArgs, storage, multiChainProvider, secrets } = context;
  const { startBlock: defaultStartBlock, publisherURLWithStream } = userArgs;
  const bearerToken = await secrets.get("BEARER_TOKEN");

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
  // save the initial block we started from
  const initialBlock = lastProcessedBlock;
  console.log(`Last processed block: ${lastProcessedBlock} \n`);
  // fetch the current block
  const currentBlock = (await provider.getBlockNumber()) - 1;
  console.log(`Current block: ${currentBlock} \n`);
  // Fetch recent logs in range of 100 blocks
  const filter = [
    lockerContract.filters.FundsDeposited(),
    lockerContract.filters.FundsWithdrawn(),
    lockerContract.filters.WithdrawCanceled(),
  ] as EventFilter;

  // make the requests and avoid hitting api rate limit
  let nbRequests = 0;
  let totalEvents: ethers.Event[] = [];
  while (lastProcessedBlock < currentBlock && nbRequests < MAX_REQUESTS) {
    nbRequests++;
    const fromBlock = lastProcessedBlock + 1;
    const toBlock = Math.min(fromBlock + MAX_RANGE, currentBlock);

    console.log(`Fetching log events from blocks ${fromBlock} to ${toBlock}. \n`);

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
  // make the requests and avoid hitting api rate limit

  // Parse the events into a more appriate form to be broadcasted
  const parsedEvent = totalEvents.map(mapEvent);
  console.log({ parsedEvent });
  console.log(
    `${parsedEvent.length} events were fetched in total from block:${initialBlock} to block:${lastProcessedBlock}. \n`
  );
  if (parsedEvent.length === 0) {
    await storage.set(BLOCK_STORAGE_KEY, lastProcessedBlock.toString());
    return {
      canExec: false,
      message: `No new event found`,
    };
  }
  // Parse the events into a more appriate form to be broadcasted

  // iterate through and publish each event
  let publisherError = false;
  let lastSavedBlock = "0";
  for await (const singleEvent of parsedEvent) {
    const publishSuccess: Boolean = await publishEvent(
      singleEvent,
      String(publisherURLWithStream),
      String(bearerToken)
    );
    // update the last stored key to the last published event if succesfull,
    if (publishSuccess) {
      await storage.set(BLOCK_STORAGE_KEY, singleEvent.blockNumber.toString());
      lastSavedBlock = singleEvent.blockNumber.toString();
    } else {
      // otherwise break because we assume the publisher is faulty
      publisherError = false;
      break;
    }
    // iterate through and publish each event
  }

  // validate the response from logstore, and only update block storage only if we get an appropriate resposnse back from the api
  return {
    canExec: false,
    message: publisherError
      ? "Publisher error"
      : `Succesfully published events from ${initialBlock} to ${lastSavedBlock}`,
  };
});
