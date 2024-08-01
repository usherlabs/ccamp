import { getAgent } from '.';
import { canisterId, createActor } from '../../../src/declarations/remittance';

const localCanisterIds = require('../../../.dfx/local/canister_ids.json');

export function getRemittanceCanister({ agent } = { agent: getAgent() }) {
	// obtain the dc canister id from env variables
	const effectiveCanisterId =
		canisterId?.toString() ?? localCanisterIds.remittance.local;
	const DCCanister = createActor(effectiveCanisterId, { agent });

	return { canister: DCCanister, id: canisterId };
}
