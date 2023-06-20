import { ethers as hEthers, upgrades } from 'hardhat';
import { writeJSONToFileOutside } from '../test/utils/functions';

const CANISTER_ADDRESS = '0x57c1D4dbFBc9F8cB77709493cc43eaA3CD505432';//random address
async function main() {
	const [signer] = await hEthers.getSigners();

	// deploy libs
	const Lib = await hEthers.getContractFactory('VerifySignature');
	const lib = await Lib.deploy();
	await lib.deployed();
	// deploy libs

	// deploy contract
	const LockerContract = await hEthers.getContractFactory('Locker', {
		signer: signer,
		libraries: {
			VerifySignature: lib.address,
		},
	});

	const lockerContract = await upgrades.deployProxy(
		LockerContract,
		[CANISTER_ADDRESS],
		{ unsafeAllowLinkedLibraries: true }
	);
	await lockerContract.deployed();
	const lockerContractAddress = lockerContract.address;
	console.log(`lockerContractAddress deployed to ${lockerContractAddress}`, {
		CANISTER_ADDRESS,
	});
	// deploy contract

	const deployedAddresses = {
		lockerContractAddress,
	};
	await writeJSONToFileOutside(deployedAddresses, 'address.json');
	return lockerContract;
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
	console.error(error);
	process.exitCode = 1;
});
