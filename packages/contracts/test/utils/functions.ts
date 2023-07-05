import { BigNumberish, ContractTransaction, Signer } from 'ethers';
import fs from 'fs';
import { ethers as hEthers, upgrades } from 'hardhat';
import path from 'path';

export async function generateHashAndSignature(
	nonce: number,
	amount: BigNumberish,
	recipient: string,
	signer: Signer
) {
	// generate the has from the amount and hash
	const encodedData = hEthers.utils.solidityPack(
		['uint256', 'uint256', 'address'],
		[nonce, amount, recipient]
	);
	const dataHash = hEthers.utils.keccak256(encodedData);
	// sign the hash recieved
	const signature = await signer.signMessage(hEthers.utils.arrayify(dataHash));
	// retuen both the signature and amount
	console.log({signature, dataHash, addr: await signer.getAddress()});;
	return { hash: dataHash, signature };
}

export const getChainId = async () =>
	await hEthers.provider
		.getNetwork()
		.then((n: { chainId: number }) => n.chainId);

export async function loadLockerContract(
	canisterAddress: string,
	signer: Signer
) {
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
		[canisterAddress],
		{ unsafeAllowLinkedLibraries: true }
	);
	// deploy contract

	return lockerContract;
}

export async function fetchEventArgsFromTx(
	tx: ContractTransaction,
	eventName: string
) {
	const receipt = await tx.wait();
	const foundEvent = receipt.events?.find((x) => x.event === eventName);
	return foundEvent?.args;
}

export async function writeJSONToFileOutside(
	inputJsonData: Record<string, string>,
	filename: string
) {
	// Specify the absolute path of the directory outside the current directory where you want to write the file.
	const targetDirectory = path.join(__dirname, '../..');

	// Combine the target directory and the filename to get the full file path.
	const filePath = path.join(targetDirectory, filename);
	// attach the chainId
	const chainId = await getChainId();
	const jsonData = { [chainId]: inputJsonData };
	// Check if the file exists.
	fs.access(filePath, fs.constants.F_OK, (err) => {
		if (err) {
			// If the file does not exist, create it.
			fs.writeFile(filePath, JSON.stringify(jsonData), (err) => {
				if (err) {
					console.error(err);
					return;
				}
				console.log(`JSON addresses data written to ${filePath}`);
			});
		} else {
			// If the file already exists, append the JSON data to it.
			fs.readFile(filePath, (err, data) => {
				if (err) {
					console.error(err);
					return;
				}
				const existingData = JSON.parse(data.toString());
				const newData = Object.assign(existingData, jsonData);
				fs.writeFile(filePath, JSON.stringify(newData, null, 2), (err) => {
					if (err) {
						console.error(err);
						return;
					}
					console.log(`JSON addresses data appended to ${filePath}`);
				});
			});
		}
	});
}
