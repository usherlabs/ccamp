export interface LogstorePayload {
  event_name: string;
  canister_id: string;
  account: string;
  amount: number;
  chain: string;
  token: string;
  blockNumber: number;
}
