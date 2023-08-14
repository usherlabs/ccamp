import { ethers } from "ethers";

// go through each event and create a model response
export function mapEvent({ event, args }: ethers.Event) {
  if (!args) return;
  args = { ...args };

  return {
    event_name: event,
    canister_id: args.canisterId,
    account: args.account,
    amount: +args.amount,
    chain: args.chain,
    token: args.token,
  };
}
