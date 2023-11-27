import eventHandler from '@/lib/eventhandler';
import { EthereumBlockRow, NotificationResponseMessage } from '@/types';
import {
	buildBlockNumberQuery,
	CREATE_TRIGGER_QUERY,
	LISTEN_TO_TRIGGER_QUERY,
} from '@/utils/queries';
import { Client, QueryResult } from 'pg';

export class PostgresClient {
	private _client: Client;

	constructor(connectionString: string) {
		this._client = new Client(connectionString);
		this._client.connect();
	}

	public async listen() {
		await this._client.query(CREATE_TRIGGER_QUERY);
		await this._client.query(LISTEN_TO_TRIGGER_QUERY);

		this._client.on('notification', (data: NotificationResponseMessage) => {
			eventHandler.call(this, data);
		});

		this.log('Listening for inserts into the postgres database');
	}

	public async getBlockByNumber(
		blockNumber: string | number
	): Promise<EthereumBlockRow> {
		const queryString = buildBlockNumberQuery(blockNumber.toString());
		const response = await this._query(queryString);

		if (!response.rowCount)
			throw new Error(
				`BLOCK_NOT_FOUND: Block with number ${blockNumber} not found`
			);

		return response.rows[0];
	}

	private async _query(queryString: string): Promise<QueryResult<any>> {
		return this._client.query(queryString);
	}

	public async log(text: string) {
		// TODO change to actual logger
		console.log(text);
	}
}
