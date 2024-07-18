// const localRequire = createRequire(import.meta.url);
import localCanisterIds from '@ccamp/canisters/.dfx/local/canister_ids.json';
import remoteCanisterIds from '@ccamp/canisters/canister_ids.json';
import lockerContractAddresses from '@ccamp/contracts/address.json';
import { Locker__factory } from '@ccamp/contracts/typechain-types/index';
import { ActorSubclass, Agent, HttpAgent } from '@dfinity/agent';
import { Secp256k1KeyIdentity } from '@dfinity/identity-secp256k1';
import { Principal } from '@dfinity/principal';
import { ethers } from 'ethers';
import fetch from 'isomorphic-fetch';

import erc20ABI from './abi/erc20.json';
import {
	CanisterIds,
	CanisterType,
	DataCollectionCanister,
	Environment,
	ProtocolDataCollectionCanister,
	RemittanceCanister,
} from './types';
import { CANISTER_TYPES, canisterActors, ENV, HOSTS } from './utils/constants';
import { prependKeyWith0x } from './utils/functions';

export class CCAMPClient {
	public agent: Agent;
	public canisterIds: CanisterIds;
	public env: Environment;
	public identity: Secp256k1KeyIdentity;
	public actors = canisterActors;

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
		const strippedPK = safeEthereumPrivateKey.replace('0x', '');
		const privateKey = Buffer.from(strippedPK, 'hex');
		this.identity = Secp256k1KeyIdentity.fromSecretKey(privateKey);

		// use keypair to generate an agent
		const host = HOSTS[env];
		this.agent = new HttpAgent({
			identity: this.identity,
			host: host,
			fetch,
		});

		const baseFile = env === ENV.prod ? remoteCanisterIds : localCanisterIds;
		const baseFileKey = env === ENV.prod ? 'ic' : 'local';

		this.canisterIds = {
			dataCollection: baseFile['data_collection'][baseFileKey],
			protocolDataCollection: baseFile['protocol_data_collection'][baseFileKey],
			remittance: baseFile['remittance'][baseFileKey],
			icaf: baseFile['icaf'][baseFileKey],
		};
		this.env = env;
	}

	getPrincipal() {
		return this.agent.getPrincipal();
	}

	getCanisterInstance(
		canisterType: CanisterType,
		overrides: { canisterId?: string } = {},
	): ActorSubclass<any> {
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

	getCCampCanisters(): {
		pdcCanister: ProtocolDataCollectionCanister;
		remittanceCanister: RemittanceCanister;
		dcCanister: DataCollectionCanister;
	} {
		const pdcCanister = this.getCanisterInstance(
			CANISTER_TYPES.PROTOCOL_DATA_COLLECTION,
		) as ProtocolDataCollectionCanister;

		const remittanceCanister = this.getCanisterInstance(
			CANISTER_TYPES.REMITTANCE,
		);
		const dcCanister = this.getCanisterInstance(CANISTER_TYPES.DATA_COLLECTION);

		return {
			pdcCanister,
			remittanceCanister,
			dcCanister,
		};
	}

	async approveLockerContract(
		erc20TokenAddress: string,
		amountToApprove: ethers.BigNumberish,
		signer: ethers.Wallet,
		overrides: { lockerContract?: string } = {},
	): Promise<ethers.ContractTransaction> {
		let chainId = (await signer.provider.getNetwork()).chainId.toString();

		const lockerContractAddress =
			overrides.lockerContract ||
			lockerContractAddresses[chainId]?.lockerContractAddress;
		if (!lockerContractAddress)
			throw new Error(
				'CCAMPClient.approveLockerContract: provide value for  `overrideLockerContract` parameter ',
			);

		this._logger(
			`CCAMPClient.approveLockerContract: Approving Locker contract:${lockerContractAddress} for amount:${amountToApprove}`,
		);
		// get the erc20 contract
		const contract = new ethers.Contract(erc20TokenAddress, erc20ABI, signer);
		const approvalTx = await contract.approve(
			lockerContractAddress,
			amountToApprove,
		);

		return approvalTx;
	}

	async deposit(
		amount: ethers.BigNumberish,
		tokenAddress: string,
		signer: ethers.Wallet,
		overrides: { lockerContract?: string; dcCanister?: string } = {},
	): Promise<ethers.ContractTransaction> {
		let chainId = (await signer.provider.getNetwork()).chainId.toString();
		const canisterId =
			overrides.dcCanister || this.canisterIds[CANISTER_TYPES.DATA_COLLECTION];

		const lockerContractAddress = this._getLockerContractAddress(chainId, {
			lockerContract: overrides.lockerContract,
		});
		const lockerContract = Locker__factory.connect(
			lockerContractAddress,
			signer,
		);

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
		chain: string,
		signer: ethers.Wallet,
		overrides: {
			lockerContract?: string;
			dcCanister?: string;
			remittanceCanister?: string;
		} = {},
	) {
		const address = signer.address;
		let chainId = (await signer.provider.getNetwork()).chainId.toString();
		const lockerContractAddress = this._getLockerContractAddress(chainId, {
			lockerContract: overrides.lockerContract,
		});
		const amountSignature = await signer.signMessage(amount.toString());
		const dcCanisterID =
			overrides.dcCanister || this.canisterIds.dataCollection;

		// get an instance of the remittance canister
		const remittanceCanister = this.getCanisterInstance(
			CANISTER_TYPES.REMITTANCE,
			{ canisterId: overrides.remittanceCanister },
		) as RemittanceCanister;

		// get the parameters from the remittance canister
		const { signature, hash, nonce } = await remittanceCanister.remit(
			tokenAddress,
			chain,
			address,
			Principal.from(dcCanisterID),
			BigInt(String(amount)),
			amountSignature,
		);
		this._logger(
			`CCAMPClient.withdraw: Parameters requested obtained from remittance canister`,
		);

		// withdraw from the locker contract
		const lockerContract = Locker__factory.connect(
			lockerContractAddress,
			signer,
		);
		this._logger(
			`CCAMPClient.withdraw: Depositing tokens into address:${address}`,
		);
		const withdrawTx = lockerContract.withdraw(
			dcCanisterID,
			tokenAddress,
			nonce.toString(),
			amount,
			signature,
		);
		return withdrawTx;
	}

	private _getLockerContractAddress(
		chainId: string | number,
		overrides: { lockerContract?: string } = {},
	) {
		const lockerContractAddress =
			overrides.lockerContract ||
			lockerContractAddresses[chainId]?.lockerContractAddress;
		if (!lockerContractAddress)
			throw new Error(
				'CCAMPClient.approveLockerContract: provide value for  `overrideLockerContract` parameter ',
			);
		return lockerContractAddress;
	}

	private _logger(text: string) {
		console.log(text);
	}
}
