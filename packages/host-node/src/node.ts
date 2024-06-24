import {
	canisterId,
	createActor,
} from '@ccamp/canisters/src/declarations/ic_af/index.js';
import { HttpAgent } from '@dfinity/agent';
import { Secp256k1KeyIdentity } from '@dfinity/identity-secp256k1';
import * as fs from 'fs';
import { createRequire } from 'node:module';
import path from 'path';

import { environment } from './config';

export async function start() {
	const env = environment.dev;
	// const tls_path = './fixtures/twitter_proof.json';
	const tls_path = './fixtures/proof.json';
	const tls_data = fs.readFileSync(tls_path);

	// Completely insecure seed phrase. Do not use for any purpose other than testing.
	// Resolves to "rwbxt-jvr66-qvpbz-2kbh3-u226q-w6djk-b45cp-66ewo-tpvng-thbkh-wae"
	const seed = 'test test test test test test test test test test test test';
	const identity = Secp256k1KeyIdentity.fromSeedPhrase(seed);

	// Require syntax is needed for JSON file imports
	const require = createRequire(import.meta.url);
	const localCanisterIds = require(env.canisterIdPath);
	console.log(localCanisterIds);

	// Use `process.env` if available provoded, or fall back to local
	const effectiveCanisterId =
		canisterId?.toString() ?? localCanisterIds.ic_af.local;

	const agent = new HttpAgent({
		identity: identity,
		host: env.agentUrl,
		fetch,
	});

	const actor = createActor(effectiveCanisterId, {
		agent,
	});

	const result = await actor.verify_tls_proof(tls_data.toString());

	fs.writeFileSync(
		`./fixtures/${path.basename(tls_path)}.data.json`,
		JSON.stringify(result, null, 2)
	);
	console.log(result);
}
