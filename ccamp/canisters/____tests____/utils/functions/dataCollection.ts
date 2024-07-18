import { getAgent } from '.';
import {
	canisterId,
	createActor,
} from '../../../src/declarations/data_collection';

const localCanisterIds = require('../../../.dfx/local/canister_ids.json');

export function getDCCanister({ agent } = { agent: getAgent() }) {
	// obtain the dc canister id from env variables
	const effectiveCanisterId =
		canisterId?.toString() ?? localCanisterIds.data_collection.local;
	const DCCanister = createActor(effectiveCanisterId, {
		agent,
	});

	return { canister: DCCanister, id: effectiveCanisterId };
}
