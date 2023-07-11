import { getDCCanister } from '../../utils/functions';

const CANISTER_NAME = 'data_collection canister';

describe('Data Collection Canister', () => {
	test('It Should return the canister name', async () => {
		const { canister } = getDCCanister();

		const response = await canister.name();
		expect(response).toBe(CANISTER_NAME);
	});
});
