import { getAgent } from '.';
import {
	canisterId,
	createActor,
} from '../../../src/declarations/protocol_data_collection';

const localCanisterIds = require('../../../.dfx/local/canister_ids.json');

export function getPDCCanister({ agent } = { agent: getAgent() }) {
	// obtain the dc canister id from env variables
	const effectiveCanisterId =
		canisterId?.toString() ?? localCanisterIds.protocol_data_collection.local;
	const PDCCanister = createActor(effectiveCanisterId, {
		agent,
	});

	return { canister: PDCCanister, id: effectiveCanisterId };
}
