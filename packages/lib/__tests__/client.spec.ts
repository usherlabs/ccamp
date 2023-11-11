import { ethers } from 'ethers';

import { CCAMPClient } from '../src/lib';
import { CANISTER_TYPES } from '../src/utils/constants';

const privateKey = '';
const chain = 'goerli';
const infuraKey = '';
const testTokenAddress = '0xB24a30A3971e4d9bf771BDc81435c25EA69A445c';
const chainIdentifier = 'ethereum:5';

describe('Remittance Canister', () => {
	test('It Should work', async () => {
		const client = new CCAMPClient(privateKey);

		const infuraProvider = new ethers.providers.InfuraProvider(
			chain,
			infuraKey,
		);
		const wallet = new ethers.Wallet(privateKey, infuraProvider);

		// const dcCanister = client.getCanisterInstace(
		// 	CANISTER_TYPES.DATA_COLLECTION,
		// );
		// console.log({ dcCanister})
		// const approval = await client.approveLockerContract(
		// 	testTokenAddress,
		// 	'100000',
		// 	wallet,
		// );
		// const approve = await client.approveLockerContract(
		// 	testTokenAddress,
		// 	10000000,
		// 	wallet,
		// );
		// console.log({ approve });
		// const deposit = await client.deposit(10000000, testTokenAddress, wallet);
		// console.log(deposit);

		// const deposit = await client.withdraw(
		// 	100,
		// 	testTokenAddress,
		// 	wallet,
		// 	chainIdentifier,
		// );
		// console.log({
		// 	deposit,
		// });
	});
});
