import { ethers as hEthers, network, upgrades } from 'hardhat';

import { writeJSONToFileOutside } from '../test/utils/functions';
import { deploySignerLibrary } from './utils/functions';

const CANISTER_ADDRESS = '0x57c1D4dbFBc9F8cB77709493cc43eaA3CD505432'; //random address
const NETWORK = `${network.name}:${network.config.chainId}`;

export async function deployLocker() {
	const [signer] = await hEthers.getSigners();
	const lib = await deploySignerLibrary();
	// deploy contract
	const LockerContract = await hEthers.getContractFactory('Locker', {
		signer: signer,
		libraries: {
			VerifySignature: lib.address,
		},
	});
	const lockerContract = await upgrades.deployProxy(
		LockerContract,
		[CANISTER_ADDRESS, NETWORK],
		{ unsafeAllowLinkedLibraries: true }
	);
	await lockerContract.deployed();
	const lockerContractAddress = lockerContract.address;
	console.log(
		`lockerContractAddress deployed to ${lockerContractAddress};with parameters`,
		{
			CANISTER_ADDRESS,
			NETWORK,
		}
	);
	// deploy contract

	// write contract to json file
	await writeJSONToFileOutside(
		{
			lockerContractAddress,
		},
		'address.json'
	);
	// write contract to json file

	return lockerContract;
}

deployLocker().catch((error) => {
	console.error(error);
	process.exitCode = 1;
});
