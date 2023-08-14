import { ethers } from "ethers";

// go through each event and create a model response
export function mapEvent({ event, args }: ethers.Event) {
  if (!args) return;
  args = { ...args };

  return {
    eventName: event,
    canisterId: args.canisterId,
    account: args.account,
    amount: args.amount,
    signatureHash: args.signatureHash || "",
    recipient: args.recipient || "",
  };
}
