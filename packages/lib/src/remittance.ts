import {
	canisterId,
	createActor,
} from '@ccamp/canisters/src/declarations/remittance';
import { HttpAgent } from '@dfinity/agent';
import { Secp256k1KeyIdentity } from '@dfinity/identity-secp256k1';
import fetch from 'isomorphic-fetch';
import { createRequire } from 'node:module';

const localRequire = createRequire(import.meta.url);
const localCanisterIds = localRequire(
	'@ccamp/canisters/.dfx/local/canister_ids.json'
);

const LOCALHOST = 'http://127.0.0.1:4943';

export async function requestRemittance(
	ethereumPrivateKey: string,
	{ host = LOCALHOST } = {} as { host?: string }
) {
	// validate the string is a public key

	// Convert the private key to a Buffer and generate a keypair
	const privateKey = Buffer.from(ethereumPrivateKey, 'hex');
	const identity = Secp256k1KeyIdentity.fromSecretKey(privateKey);

	// inistantiate an agent and actor with this remittance id
	// use process.env.CANISTER_ID_REMITTANCE ||process.env.REMITTANCE_CANISTER_ID; if available
	// or fall back to locally deployed instance
	const effectiveCanisterId =
		canisterId?.toString() ?? localCanisterIds.remittance.local;
	const agent = new HttpAgent({
		identity: identity,
		host: host,
		fetch,
	});
	const actor = createActor(effectiveCanisterId, {
		agent,
	});

	// actually make the request to get remitted funds
	const response = await actor.request();
	return response;
}
