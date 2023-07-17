import { utils } from 'ethers';

import { getRemittanceCanister } from '../../utils/functions';

const CANISTER_NAME = 'remittance canister';
const { canister } = getRemittanceCanister();

describe('Remittance Canister', () => {
	test('It Should return the canister name', async () => {
		const response = await canister.name();
		expect(response).toBe(CANISTER_NAME);
	});

	test('It Can generate pubblic key', async () => {
		const response = await canister.public_key();
		if (!('Ok' in response)) return;

		const { sec1_pk: sec1PK, etherum_pk: ethereumPK } = response.Ok;
		const computedEthereumPK = utils.computeAddress(`0x${sec1PK}`);

		expect(ethereumPK.toLowerCase()).toEqual(computedEthereumPK.toLowerCase());
	});
});
