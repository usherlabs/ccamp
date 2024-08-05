import { getRemittanceCanister } from '../../utils/functions';

const CANISTER_NAME = 'remittance canister';
const { canister } = getRemittanceCanister();

describe('Remittance Canister', () => {
	test('It Should return the canister name', async () => {
		const response = await canister.name();
		expect(response).toBe(CANISTER_NAME);
	});
});
