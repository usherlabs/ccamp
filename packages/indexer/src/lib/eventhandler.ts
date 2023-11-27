import { DBEventPayload, NotificationResponseMessage } from '@/types';
import { parseTransactionReciept } from '@/utils/functions/events';

import { PostgresClient } from './postgres';

export default async function (data: NotificationResponseMessage) {
	const { payload } = data;
	try {
		if (!(this instanceof PostgresClient)) throw Error('INVALID_CONTEXT');

		const client = this as PostgresClient;

		client.log(`Recieved a payload of :${JSON.stringify(data)}`);

		const eventPayload = JSON.parse(payload) as DBEventPayload;
		const blockDetails = await client.getBlockByNumber(eventPayload.block$);

		const parsedTxnPayload = await parseTransactionReciept(
			eventPayload,
			blockDetails.data.transaction_receipts
		);
		// console.log(parsedTxnPayload);
		// console.dir(
		// 	blockDetails.data.transaction_receipts[
		// 		+parsedTxnPayload.index.toString()
		// 	],
		// 	{ depth: null }
		// );
	} catch (err) {
		console.log(`There was an error:${err.message}`);
		console.log(err);
	}
}
