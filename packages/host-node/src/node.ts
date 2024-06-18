import { HttpAgent } from "@dfinity/agent";
import { canisterId, createActor } from "@ccamp/canisters/src/declarations/ic_af/index.js";
import { Secp256k1KeyIdentity } from "@dfinity/identity-secp256k1";
import { createRequire } from "node:module";

import * as fs from 'fs';

export async function start() {
	const tls_data = fs.readFileSync('./fixtures/twitter_proof.json');

	// Completely insecure seed phrase. Do not use for any purpose other than testing.
	// Resolves to "rwbxt-jvr66-qvpbz-2kbh3-u226q-w6djk-b45cp-66ewo-tpvng-thbkh-wae"
	const seed = "test test test test test test test test test test test test";

	const identity = Secp256k1KeyIdentity.fromSeedPhrase(seed);

	// Require syntax is needed for JSON file imports
	const require = createRequire(import.meta.url);
	const localCanisterIds = require("@ccamp/canisters/.dfx/local/canister_ids.json");

	// Use `process.env` if available provoded, or fall back to local
	const effectiveCanisterId =
	canisterId?.toString() ?? localCanisterIds.ic_af.local;

	const agent = new HttpAgent({
	identity: identity,
	host: "http://127.0.0.1:4943",
	fetch,
	});

	const actor = createActor(effectiveCanisterId, {
	agent,
	});

	const result = await actor.verify_tls_proof(
		tls_data.toString()
	);

	console.log(result);
}
