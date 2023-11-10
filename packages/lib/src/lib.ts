// const localRequire = createRequire(import.meta.url);
import localCanisterIds from '@ccamp/canisters/.dfx/local/canister_ids.json';
import remoteCanisterIds from '@ccamp/canisters/canister_ids.json';
import { Locker__factory } from '@ccamp/contracts';
import lockerContractAddresses from '@ccamp/contracts/address.json';
import { Agent, HttpAgent } from '@dfinity/agent';
import { Secp256k1KeyIdentity } from '@dfinity/identity-secp256k1';
import { Principal } from '@dfinity/principal';
import { ethers } from 'ethers';
import fetch from 'isomorphic-fetch';
import { createRequire } from 'node:module';

import erc20ABI from './abi/erc20.json';
import {
	CanisterIds,
	CanisterType,
	Environment,
	RemittanceCanister,
} from './types';
import { CANISTER_TYPES, canisterActors, ENV, HOSTS } from './utils/constants';
import { prependKeyWith0x } from './utils/functions';

export class CCAMPClient {
	public agent: Agent;
	public canisterIds: CanisterIds;
	public actors = canisterActors;
	public env: Environment;

	// initialise a constructor with your private key and the url of the rpc you want to connect to
	constructor(
		ethereumPrivateKey: string,
		{ env = ENV.prod } = {} as { env?: Environment },
	) {
		const safeEthereumPrivateKey = prependKeyWith0x(ethereumPrivateKey);
		// validate the string is a public key
		if (!ethers.utils.isHexString(safeEthereumPrivateKey))
			throw new Error(
				'CCAMPClient: Invalid private key provided to constructor',
			);

		// Convert the private key to a Buffer and generate a keypair
		const privateKey = Buffer.from(ethereumPrivateKey, 'hex');
		const identity = Secp256k1KeyIdentity.fromSecretKey(privateKey);

		// use keypair to generate an agent
		const host = HOSTS[env];
		this.agent = new HttpAgent({
			identity: identity,
			host: host,
			fetch,
		});

		const baseFile = env === ENV.prod ? remoteCanisterIds : localCanisterIds;
		const baseFileKey = env === ENV.prod ? 'ic' : 'local';

		this.canisterIds = {
			dataCollection: baseFile['data_collection'][baseFileKey],
			protocolDataCollection: baseFile['protocol_data_collection'][baseFileKey],
			remittance: baseFile['remittance'][baseFileKey],
		};
		this.env = env;
	}

	getCanisterInstance(
		canisterType: CanisterType,
		overrides: { canisterId?: string } = {},
	) {
		// initialise an instance of the pdc canister and return it
		const createActor = canisterActors[canisterType];
		const actor = createActor(
			overrides.canisterId || this.canisterIds[canisterType],
			{
				agent: this.agent,
			},
		);
		return actor;
	}

	async approveLockerContract(
		erc20TokenAddress: string,
		amountToApprove: ethers.BigNumberish,
		signer: ethers.Wallet,
		overrides: { lockerContract?: string } = {},
	) {
		let chainId = (await signer.provider.getNetwork()).chainId.toString();

		const lockerContractAddress =
			overrides.lockerContract ||
			lockerContractAddresses[chainId]?.lockerContractAddress;
		if (!lockerContractAddress)
			throw new Error(
				'CCAMPClient.approveLockerContract: provide value for  `overrideLockerContract` parameter ',
			);

		console.log(`Approving Logger contract:${lockerContractAddress}`);
		// get the erc20 contract
		const contract = new ethers.Contract(erc20TokenAddress, erc20ABI, signer);
		const approvalTx = await contract.approve(
			lockerContractAddress,
			amountToApprove,
		);
		console.log(`Waiting for transaction:${approvalTx.hash}`);
		const response = await approvalTx.wait();

		return response;
	}

	async deposit(
		amount: ethers.BigNumberish,
		tokenAddress: string,
		signer: ethers.Wallet,
		overrides: { lockerContract?: string; dcCanister?: string } = {},
	) {
		let chainId = (await signer.provider.getNetwork()).chainId.toString();

		const lockerContractAddress =
			overrides.lockerContract ||
			lockerContractAddresses[chainId]?.lockerContractAddress;
		if (!lockerContractAddress)
			throw new Error(
				'CCAMPClient.approveLockerContract: provide value for  `overrideLockerContract` parameter ',
			);

		const lockerContract = Locker__factory.connect(
			lockerContractAddress,
			signer,
		);
		const canisterId =
			overrides.dcCanister || this.canisterIds[CANISTER_TYPES.DATA_COLLECTION];

		const depositTx = await lockerContract.depositFunds(
			canisterId,
			amount,
			tokenAddress,
		);

		return depositTx;
	}

	async withdraw(
		amount: ethers.BigNumberish,
		tokenAddress: string,
		signer: ethers.Wallet,
		chain: string,
		overrides: {
			lockerContract?: string;
			dcCanister?: string;
			remittanceCanister?: string;
		} = {},
	) {
		const remittanceCanister = this.getCanisterInstance(
			CANISTER_TYPES.REMITTANCE,
			{ canisterId: overrides.remittanceCanister },
		) as RemittanceCanister;

		const address = signer.address;
		const amountSignature = await signer.signMessage(amount.toString());

		const dcCanisterID =
			overrides.dcCanister || this.canisterIds.dataCollection;

		const { signature, hash, nonce } = await remittanceCanister.remit(
			tokenAddress,
			chain,
			address,
			Principal.from(dcCanisterID),
			BigInt(String(amount)),
			amountSignature,
		);

		let chainId = (await signer.provider.getNetwork()).chainId.toString();

		const lockerContractAddress =
			overrides.lockerContract ||
			lockerContractAddresses[chainId]?.lockerContractAddress;
		if (!lockerContractAddress)
			throw new Error(
				'CCAMPClient.approveLockerContract: provide value for  `overrideLockerContract` parameter ',
			);
		const lockerContract = Locker__factory.connect(
			lockerContractAddress,
			signer,
		);
		console.log({ nonce: nonce.toString() });

		// withdraw from locker
		const withdrawTx = lockerContract.withdraw(
			dcCanisterID,
			tokenAddress,
			nonce.toString(),
			'100',
			signature,
		);
		return withdrawTx;
	}
}
