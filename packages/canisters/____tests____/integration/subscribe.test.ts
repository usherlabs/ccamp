import { Principal } from '@dfinity/principal';

import {
	getDCCanister,
	getPDCCanister,
	getRemittanceCanister,
} from '../utils/functions';

describe('PUB/SUB', () => {
	it('The remittance canister should be able to subscribe to the Data Collection Canister', async () => {
		const { canister: dcCanister, id: dcCanisterId } = getDCCanister();
		const { canister: rCanister, id: rCanisterId } = getRemittanceCanister();

		// use the remittance canister to subscribe to the DC canister
		await rCanister.subscribe_to_dc(Principal.from(dcCanisterId));
		// check from the dc canister if the subscription was succesfull
		const response = await dcCanister.is_subscribed(
			Principal.from(rCanisterId)
		);
		// confirm from the dc canister if the r-canister is subscribed to it
		expect(response).toBe(true);
	});

	it('The remittance canister should be able to subscribe to the Protocol Data Collection Canister', async () => {
		const { canister: pdcCanister, id: pdcCanisterId } = getPDCCanister();
		const { canister: rCanister, id: rCanisterId } = getRemittanceCanister();

		// use the remittance canister to subscribe to the DC canister
		await rCanister.subscribe_to_pdc(Principal.from(pdcCanisterId));
		// check from the dc canister if the subscription was succesfull
		const response = await pdcCanister.is_subscribed(
			Principal.from(rCanisterId)
		);
		// confirm from the dc canister if the r-canister is subscribed to it
		expect(response).toBe(true);
	});
});
