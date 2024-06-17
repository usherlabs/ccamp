import { EventType } from "@/types";
import LockerABI from "@/abis/Locker.json";
import { Log } from "ethers";

const AbiCoder = require("web3-eth-abi");

/**
 * format an hex string properly
 * from '\\x9c81e8f60a9b8743678f1b6ae893cc72c6bc6840' to '0x9c81e8f60a9b8743678f1b6ae893cc72c6bc6840'
 * @param hexString
 * @returns
 */
export const formatHexString = (hexString: string) => {
  if(hexString.startsWith("0x")) return hexString;

  const cleanHexString = hexString.replace(/\\x/g, "");
  const byteArray = Buffer.from(cleanHexString, "hex").toString("hex");
  return `0x${byteArray}`;
};

/**
 * Convert the ethereum event log into a format that can be parsed and published to the logstore
 * @param log
 * @returns
 */
export const parseEventLog = (log: Log & { logIndex: string }) => {
  const events = LockerABI.filter(
    (e) => e.type === "event" && e.anonymous === false
  );
  const signature = log.topics[0];
  const event = events.find(
    (e) => AbiCoder.encodeEventSignature(e) === signature
  );

  if (!event) return undefined;

  const rawReturnValues = AbiCoder.decodeLog(
    event.inputs,
    log.data,
    log.topics.slice(1)
  );
  const returnValues = Object.keys(rawReturnValues)
    .filter((key) => isNaN(+key) && key !== "__length__")
    .reduce((obj, key) => ({ ...obj, [key]: rawReturnValues[key] }), {});

  return {
    event: event.name,
    signature: signature,
    address: log.address,
    blockHash: log.blockHash,
    blockNumber: log.blockNumber,
    transactionHash: log.transactionHash,
    transactionIndex: log.transactionIndex,
    logIndex: log.logIndex,
    raw: {
      data: log.data,
      topics: log.topics,
    },
    eventParameters: returnValues as EventType,
  };
};
