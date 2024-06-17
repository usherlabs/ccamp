import { BlockParams, TransactionReceiptParams } from 'ethers';

export type EthereumBlockRow = {
	hash: Buffer;
	number: string | number;
	parent_hash: Buffer;
	data: {
		block: BlockParams;
		transaction_receipts: TransactionReceiptParams[];
	};
};

export type DBEventPayload = {
	vid: number;
	block$: number;
	id: string;
	canister_id: string;
	account: string;
	amount: number;
	chain: string;
	token: string;
};
