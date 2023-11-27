import { PostgresClient } from '@/lib/postgres';
import environment from '@/config/env';
import { ENV_CONNECTION_STRING_NOT_SET } from '@/utils/errors';
import { isConfigValid } from '@/utils/functions/validator';


export async function startNode() {
	if(isConfigValid()) throw new Error("INVALID_CONFIG: all environment variables not set")

	const { postgresConnectionString } = environment;
	// validate that the connection string was set as an env variable
	if (!postgresConnectionString) throw new Error(ENV_CONNECTION_STRING_NOT_SET);
	const postgresClient = new PostgresClient(postgresConnectionString);

	// listen for new insertions into the databasr
	await postgresClient.listen();
}
