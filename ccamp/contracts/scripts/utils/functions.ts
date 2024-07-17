import { ethers as hEthers, network, upgrades } from 'hardhat';

export async function deploySignerLibrary() {
	// deploy libs
	const Lib = await hEthers.getContractFactory('VerifySignature');
	const lib = await Lib.deploy();
	await lib.deployed();
	// deploy libs

    return lib;
}
