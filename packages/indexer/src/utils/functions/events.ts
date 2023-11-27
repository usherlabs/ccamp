import { DBEventPayload } from '@/types';
import { DeployedAddressType } from '@/types/contract';
import addresses from '@ccamp/contracts/address.json';
import { TransactionReceiptParams } from 'ethers';

import environment from '../../config/env';
import { formatHexString, parseEventLog } from './utilities';

/**
 *  given a certain payload and the reciepts from the transactions find
 *  return the index of teh reciept which contains the actual event
 * @param eventPayload
 * @param transactionReciepts
 */
export async function parseTransactionReciept(
	eventPayload: DBEventPayload,
	transactionReciepts: TransactionReceiptParams[]
): Promise<LogStorePayloadType> {
	let allAddresses = addresses as DeployedAddressType;
	const lockerContractAddress =
		allAddresses[environment.chainId].lockerContractAddress;

	const event = formatPayloadHexFields(eventPayload);

	// filter all events that do not come from this address
	const relevantReciepts = transactionReciepts.filter(
		(reciept) =>
			reciept.to.toLowerCase() === lockerContractAddress.toLowerCase()
	);
	const [foundReciept] = await relevantReciepts
		.map((reciept) => reciept.logs.map(parseEventLog))
		.flat()
		.filter(Boolean)
		.filter((parsedLog) => {
			const { eventParameters: logsEventPayload } = parsedLog;
			// make sure all the events parameters match
			return (
				logsEventPayload.account?.toLowerCase() ===
					event.account?.toLowerCase() &&
				logsEventPayload.canisterId === event.canister_id &&
				logsEventPayload.amount?.toString() === event.amount?.toString() &&
				logsEventPayload.chain === event.chain &&
				logsEventPayload.token?.toLowerCase() === event.token?.toLowerCase()
			);
		});

	if (!foundReciept)
		throw new Error(
			`RECIEPT_NOT_FOUND: event not found in block:${eventPayload.block$} `
		);

	return {
		logStoreStreamId: '0x4178babe9e5148c6d5fd431cd72884b07ad855a0/lsan-events',
		logStoreChainId: '8997',
		logStoreChannelId: 'evm-validate',

		address: foundReciept.address,
		blockHash: foundReciept.blockHash,
		data: foundReciept.raw.data,
		index: foundReciept.transactionIndex,
		topics: foundReciept.raw.topics as string[],
		transactionHash: foundReciept.transactionHash,
	};
}

// properly parse all the hex strings from the DB
export const formatPayloadHexFields = (payload: DBEventPayload) => ({
	...payload,
	id: formatHexString(payload.id),
	account: formatHexString(payload.account),
	token: formatHexString(payload.token),
});
