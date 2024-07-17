import { _SERVICE as _DC_SERVICE } from '@/declarations/data_collection/data_collection.did';
import { _SERVICE as _PDC_SERVICE } from '@/declarations/protocol_data_collection/protocol_data_collection.did';
import { _SERVICE as _R_SERVICE } from '@/declarations/remittance/remittance.did';
import { ActorSubclass } from '@dfinity/agent';
import { Principal } from '@dfinity/principal';

import {
	getDCCanister,
	getPDCCanister,
	getRemittanceCanister,
} from '../../utils/functions';

const CANISTER_NAME = 'data_collection canister';

describe('Data Collection Canister', () => {
	let DC_CANISTER: ActorSubclass<_DC_SERVICE>;
	let R_CANISTER: ActorSubclass<_R_SERVICE>;
	let R_CANISTER_ID: String;
	let PDC_CANISTER: ActorSubclass<_PDC_SERVICE>;

	beforeAll(async () => {
		// fetch all the canisters
		const { canister: dcCanister} = getDCCanister();
		const { canister: pdcCanister } = getPDCCanister();
		const { canister: rCanister, id: rCanisterId } = getRemittanceCanister();

		// register teh remittance canister in both dc and pdc
		await dcCanister.set_remittance_canister(Principal.from(rCanisterId));
		await pdcCanister.set_remittance_canister(Principal.from(rCanisterId));

		// save all the canisters fro future use
		DC_CANISTER = dcCanister;
		PDC_CANISTER = pdcCanister;
		R_CANISTER = rCanister;
		R_CANISTER_ID = rCanisterId;
	});


	test('It Should return the canister name', async () => {
		const response = await DC_CANISTER.name();
		expect(response).toBe(CANISTER_NAME);
	});

	test('It can set remittance canister', async () => {
		await DC_CANISTER.set_remittance_canister(Principal.from(R_CANISTER_ID));

		const rCanisterDetails = await DC_CANISTER.get_remittance_canister();
		expect(rCanisterDetails.canister_principal).toEqual(
			Principal.from(R_CANISTER_ID),
		);
	});
});
