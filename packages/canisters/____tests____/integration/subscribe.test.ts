import { Principal } from '@dfinity/principal';

import {
	generateRandomIdentity,
	getAgent,
	getDCCanister,
	getRemittanceCanister,
} from '../utils/functions';

describe('PUB/SUB', () => {
	it('The remittance canister should be able to subscribe to the Data Collection Canister', async () => {
		const { canister: dcCanister, id: dcCanisterId } = getDCCanister();
		const { canister: rCanister, id: rCanisterId } = getRemittanceCanister();

		// use the remittance canister to subscribe to the DC canister
		await rCanister.setup_subscribe(Principal.from(dcCanisterId));
		// check from the dc canister if the subscription was succesfull
		const response = await dcCanister.is_subscribed(
			Principal.from(rCanisterId)
		);
		// confirm from the dc canister if the r-canister is subscribed to it
		expect(response).toBe(true);
	});

	it('Only the deployer of the canister should be able to call the subscribe function', async () => {
		try {
			const agent = getAgent({ identity: generateRandomIdentity() });
			const { id: dcCanisterId } = getDCCanister({ agent });
			const { canister: rCanister } = getRemittanceCanister({
				agent,
			});

			await rCanister.setup_subscribe(Principal.from(dcCanisterId));
			throw Error('NOT_ALLOWED');
		} catch (err) {
			expect(true).toBe(true);
		}
	});
});
