const localCanisterIds = require('../../../.dfx/local/canister_ids.json');


export const ACTOR_ONE = '0x9C81E8F60a9B8743678F1b6Ae893Cc72c6Bc6840';
export const ACTOR_TWO = '0x1AE26a1F23E2C70729510cdfeC205507675208F2';
export const TOKEN = '0xB24a30A3971e4d9bf771BDc81435c25EA69A445c';
export const CANISTER_ID = localCanisterIds.data_collection.local;
export const CHAIN = 'ethereum:5';

export const DEPOSIT_AMOUNT = 500000;
export const WITHDRAW_AMOUNT = 100000;
export const ADJUST_AMOUNT = 10000;

export const SAMPLE_DEPOSIT_EVENT = {
	event_name: 'FundsDeposited',
	canister_id: CANISTER_ID,
	account: ACTOR_ONE,
	amount: DEPOSIT_AMOUNT,
	chain: CHAIN,
	token: TOKEN,
};

export const SAMPLE_WITHDRAW_DETAILS = {
	signature:
		'0xc1f88bc447b9ab9783f25fb5e88c5eefec0b563e4a60316e007834b506490ed25b21d1d6827a5c965738aba8869d7ab08b6e7b9f4a6bce6cf0f3f577037d9fdb1c',
	account: SAMPLE_DEPOSIT_EVENT.account,
	amount: WITHDRAW_AMOUNT,
};

export const SAMPLE_WITHDRAW_EVENT = {
	event_name: 'FundsWithdrawn',
	canister_id: CANISTER_ID,
	account: ACTOR_ONE,
	amount: WITHDRAW_AMOUNT,
	chain: CHAIN,
	token: TOKEN,
};

export const SAMPLE_CANCEL_EVENT = {
	event_name: 'WithdrawCanceled',
	canister_id: CANISTER_ID,
	account: ACTOR_ONE,
	amount: WITHDRAW_AMOUNT,
	chain: CHAIN,
	token: TOKEN,
};

export const SAMPLE_ADJUST_EVENTS = [
	{
		event_name: 'BalanceAdjusted',
		canister_id: CANISTER_ID,
		account: ACTOR_ONE,
		amount: -ADJUST_AMOUNT,
		chain: CHAIN,
		token: TOKEN,
	},
	{
		event_name: 'BalanceAdjusted',
		canister_id: CANISTER_ID,
		account: ACTOR_TWO,
		amount: ADJUST_AMOUNT,
		chain: CHAIN,
		token: TOKEN,
	},
];

export const SAMPLE_ADJUST_EVENTS_NOT_RESOLVES_TO_ZERO = [
	{
		event_name: 'BalanceAdjusted',
		canister_id: CANISTER_ID,
		account: ACTOR_ONE,
		amount: -ADJUST_AMOUNT,
		chain: CHAIN,
		token: TOKEN,
	},
	{
		event_name: 'BalanceAdjusted',
		canister_id: CANISTER_ID,
		account: ACTOR_TWO,
		amount: ADJUST_AMOUNT + 50,
		chain: CHAIN,
		token: TOKEN,
	},
];