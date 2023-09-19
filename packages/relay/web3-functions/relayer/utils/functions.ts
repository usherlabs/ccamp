import ky from "ky";
import { ethers } from "ethers";
import { LogstorePayload } from "../types";

// go through each event and create a model response
export function mapEvent({
  event,
  args,
  blockNumber,
}: ethers.Event): LogstorePayload {
  if (!args) throw new Error("INVALID_EVENT_PARSED");
  args = { ...args };

  return {
    event_name: String(event),
    canister_id: String(args.canisterId),
    account: String(args.account),
    amount: +args.amount,
    chain: String(args.chain),
    token: String(args.token),
    blockNumber,
  };
}

export async function publishEvent(
  eventData: LogstorePayload,
  publisherURL: string,
  bearerToken: string
): Promise<Boolean> {
  const expectedOkResponse = "OK";
  const { blockNumber, ...jsonPayload } = eventData;
  const KYInstance = ky.create({
    headers: { Authorization: `Bearer ${bearerToken}` },
  });
  try {
    const response = await KYInstance.post(publisherURL, {
      json: {
        ...jsonPayload,
      },
    }).text();
    console.log(`Event:${JSON.stringify(eventData)} published with response:${response}`)
    return response === expectedOkResponse;
  } catch (err: unknown) {
    console.log(`There was an error publishing your event:${err}`);
    return false;
  }
}
