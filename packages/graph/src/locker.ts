import {
  FundsDeposited as FundsDepositedEvent,
  FundsWithdrawn as FundsWithdrawnEvent,
  WithdrawCanceled as WithdrawCanceledEvent,
  UpdateRemittanceCanister as UpdateRemittanceCanisterEvent,
} from "../generated/Locker/Locker";
import {
  DepositedFund,
  WithdrawnFund,
  CanceledWithdraw,
  UpdateRemittanceCanister,
} from "../generated/schema";


export function handleFundsDeposited(event: FundsDepositedEvent): void {
  const entity = new DepositedFund(
    event.transaction.hash.concatI32(event.logIndex.toI32())
  );
  entity.canisterId = event.params.canisterId;
  entity.account = event.params.account;
  entity.amount = event.params.amount;
  entity.chain = event.params.chain;
  entity.token = event.params.token;

  entity.save();
}

export function handleFundsWithdrawn(event: FundsWithdrawnEvent): void {
  const entity = new WithdrawnFund(
    event.transaction.hash.concatI32(event.logIndex.toI32())
  );
  entity.canisterId = event.params.canisterId;
  entity.account = event.params.account;
  entity.amount = event.params.amount;
  entity.chain = event.params.chain;
  entity.token = event.params.token;

  entity.save();
}

export function handleWithdrawCanceled(event: WithdrawCanceledEvent): void {
  const entity = new CanceledWithdraw(
    event.transaction.hash.concatI32(event.logIndex.toI32())
  );
  entity.canisterId = event.params.canisterId;
  entity.account = event.params.account;
  entity.amount = event.params.amount;
  entity.chain = event.params.chain;
  entity.token = event.params.token;

  entity.save();
}

export function handleUpdateRemittanceCanister(
  event: UpdateRemittanceCanisterEvent
): void {
  const entity = new UpdateRemittanceCanister(
    event.transaction.hash.concatI32(event.logIndex.toI32())
  );
  entity.remittanceCanister = event.params.remittanceCanister;
  entity.save();
}
