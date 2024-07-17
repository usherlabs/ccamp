// scripts/upgrade-box.js
import { ContractAddressOrInstance } from '@openzeppelin/hardhat-upgrades/dist/utils';
import { ethers as hEthers, network, upgrades } from 'hardhat';

import contractAddresses from '../address.json';
import { deployLocker } from './deploy-locker';
import { deploySignerLibrary } from './utils/functions';

async function upgradeLocker() {
	const [signer] = await hEthers.getSigners();

	const lib = await deploySignerLibrary();
	const lockerContract = await hEthers.getContractFactory('Locker', {
		signer: signer,
		libraries: {
			VerifySignature: lib.address,
		},
	});

	const chainId = `${network.config.chainId}` as keyof typeof contractAddresses;
	const upgradedLockerContract = await upgrades.upgradeProxy(
		contractAddresses[chainId].lockerContractAddress,
		lockerContract,
		{ unsafeAllowLinkedLibraries: true }
	);

	await upgradedLockerContract.deployed();
	console.log(`Contract upgraded successfully!!!`);
}

deployLocker().then(() => {
	upgradeLocker().catch((error) => {
		console.error(error);
		process.exitCode = 1;
	});
});
