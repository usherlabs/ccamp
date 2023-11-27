import { BigNumberish } from 'ethers';

export type DeployedAddressType = Record<
	string,
	{ lockerContractAddress: string }
>;

export type EventType = {
	canisterId: string;
	account: string;
	amount: BigNumberish;
	chain: string;
	token: string;
};
