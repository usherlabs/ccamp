import { _SERVICE as _DC_SERVICE } from '@/declarations/data_collection/data_collection.did';
import { _SERVICE as _PDC_SERVICE } from '@/declarations/protocol_data_collection/protocol_data_collection.did';
import { _SERVICE as _R_SERVICE } from '@/declarations/remittance/remittance.did';
import { ActorSubclass } from '@dfinity/agent';
import { Principal } from '@dfinity/principal';
import { ethers } from 'ethers';

import {
	ACTOR_ONE,
	ACTOR_TWO,
	ADJUST_AMOUNT,
	SAMPLE_ADJUST_EVENTS,
	SAMPLE_ADJUST_EVENTS_NOT_RESOLVES_TO_ZERO,
	SAMPLE_CANCEL_EVENT,
	SAMPLE_DEPOSIT_EVENT,
	SAMPLE_WITHDRAW_DETAILS,
	SAMPLE_WITHDRAW_EVENT,
} from '../utils/constants';
import {
	getDCCanister,
	getPDCCanister,
	getRemittanceCanister,
} from '../utils/functions';

(BigInt.prototype as any).toJSON = function () {
	return this.toString();
};

describe('PUB/SUB', () => {
	let DC_CANISTER: ActorSubclass<_DC_SERVICE>;
	let R_CANISTER: ActorSubclass<_R_SERVICE>;
	let PDC_CANISTER: ActorSubclass<_PDC_SERVICE>;
	let CANISTER_PUBLIC_KEY: string;
	let NONCE: bigint;

	async function getAvailableBalance(account: string) {
		// confirm balance
		const { balance: availableBalance } =
			(await R_CANISTER.get_available_balance(
				SAMPLE_DEPOSIT_EVENT.token,
				SAMPLE_DEPOSIT_EVENT.chain,
				account,
				Principal.from(SAMPLE_DEPOSIT_EVENT.canister_id),
			)).Ok;

		return availableBalance;
	}

	async function getWitheldBalance() {
		// confirm balance

		const { balance: availableBalance } = (await R_CANISTER.get_withheld_balance(
			SAMPLE_DEPOSIT_EVENT.token,
			SAMPLE_DEPOSIT_EVENT.chain,
			SAMPLE_DEPOSIT_EVENT.account,
			Principal.from(SAMPLE_DEPOSIT_EVENT.canister_id),
		)).Ok;

		return availableBalance;
	}

	beforeAll(async () => {
		// fetch all the canisters
		const { canister: dcCanister, id: dcCanisterId } = getDCCanister();
		const { canister: pdcCanister, id: pdcCanisterId } = getPDCCanister();
		const { canister: rCanister, id: rCanisterId } = getRemittanceCanister();

		// register teh remittance canister in both dc and pdc
		await dcCanister.set_remittance_canister(Principal.from(rCanisterId));
		await pdcCanister.set_remittance_canister(Principal.from(rCanisterId));

		// subscribe to the dc and pdc canister from the remittance canister
		await rCanister.subscribe_to_dc(Principal.from(dcCanisterId));
		await rCanister.subscribe_to_pdc(Principal.from(pdcCanisterId));

		// // get the public key of the canister
		const canisterPKVariant = await rCanister.public_key();
		const canisterPk = canisterPKVariant.Ok.etherum_pk;

		// save all the canisters fro future use
		DC_CANISTER = dcCanister;
		PDC_CANISTER = pdcCanister;
		R_CANISTER = rCanister;
		CANISTER_PUBLIC_KEY = canisterPk;
	});

	it('The PDC Canister can deposit funds to the Remittance Canister', async () => {
		// simulate a deposit event
		await PDC_CANISTER.manual_publish(JSON.stringify([SAMPLE_DEPOSIT_EVENT]));
		const availableBalance = await getAvailableBalance(ACTOR_ONE);
		expect(+availableBalance.toString()).toBeGreaterThanOrEqual(
			+SAMPLE_DEPOSIT_EVENT.amount.toString(),
		);
	});

	it('The remittance canister can generate a correct signature which can be used to withdraw funds', async () => {
		await PDC_CANISTER.manual_publish(JSON.stringify([SAMPLE_DEPOSIT_EVENT]));

		const initialAvailableBalance = await getAvailableBalance(ACTOR_ONE);
		// try to generate an event from the address used to as recipient from the deposit event
		let remittanceResponse = await R_CANISTER.remit(
			SAMPLE_DEPOSIT_EVENT.token,
			SAMPLE_DEPOSIT_EVENT.chain,
			SAMPLE_DEPOSIT_EVENT.account,
			Principal.from(SAMPLE_DEPOSIT_EVENT.canister_id),
			BigInt(SAMPLE_WITHDRAW_DETAILS.amount),
			SAMPLE_WITHDRAW_DETAILS.signature,
		);
		const {
			signature: canisterSignature,
			hash: dataHash,
			nonce,
			amount: withdrawalAmount,
		} = remittanceResponse.Ok;

		NONCE = nonce;

		// validate the signature produced
		// generate the has from the amount and hash
		const encodedData = ethers.utils.solidityPack(
			['uint256', 'uint256', 'address', 'string', 'string', 'address'],
			[
				nonce,
				withdrawalAmount,
				SAMPLE_DEPOSIT_EVENT.account,
				SAMPLE_DEPOSIT_EVENT.chain,
				SAMPLE_DEPOSIT_EVENT.canister_id,
				SAMPLE_DEPOSIT_EVENT.token,
			],
		);
		const derivedDataHash = ethers.utils.keccak256(encodedData);
		const recoveredSigner = ethers.utils.verifyMessage(
			Buffer.from(derivedDataHash.slice(2), 'hex'),
			canisterSignature,
		);

		const preAvailableBalance = await getAvailableBalance(ACTOR_ONE);
		const preWitheldBalance = await getWitheldBalance();
		// validate the hash returned by the canister is the same as the hash calculated based on the values
		expect(derivedDataHash.toLowerCase().replace("0x","")).toEqual(dataHash.toLowerCase().replace("0x",""));
		// validate that the address recovered from the signature and message is that of the r-canister
		// confirm why this fails at times by logging the hash and signature to confirm what is causing the failuer

		expect(recoveredSigner.toLowerCase()).toEqual(
			CANISTER_PUBLIC_KEY.toLowerCase(),
		);

		// generate a withdraw confirm payload and send to nthe r canister, then confirm the balance of the user has reduced by the amount withdrawn
		// simulate a withdraw event
		// TODO send a response back to the client for if it was successfull or not
		await PDC_CANISTER.manual_publish(JSON.stringify([SAMPLE_WITHDRAW_EVENT]));
		const postAvailableBalance = await getAvailableBalance(ACTOR_ONE);
		const postWitheldBalance = await getWitheldBalance();

		// validate that the balances are as they should be
		expect(postWitheldBalance.toString()).toEqual('0'); //make sure the witheld balance is now 0
		expect(preAvailableBalance.toString()).toEqual(
			postAvailableBalance.toString(),
		); // confirm both of the available balances are the same before and after withdrawal confirmation
		expect(+preAvailableBalance.toString()).toBeGreaterThanOrEqual(
			+(initialAvailableBalance - BigInt(withdrawalAmount)).toString(),
		); //validate the amount available left is equal to the initial amount minus the withdrawn amount
		expect(preWitheldBalance.toString()).toEqual(`${withdrawalAmount}`);
	});

	it('The remittance canister can to create a withdrawal and cancel a withdrawal', async () => {
		const initialAvailableBalance = await getAvailableBalance(ACTOR_ONE);

		// try to generate an event from the address used to as recipient from the deposit event
		await R_CANISTER.remit(
			SAMPLE_DEPOSIT_EVENT.token,
			SAMPLE_DEPOSIT_EVENT.chain,
			SAMPLE_DEPOSIT_EVENT.account,
			Principal.from(SAMPLE_DEPOSIT_EVENT.canister_id),
			BigInt(SAMPLE_WITHDRAW_DETAILS.amount),
			SAMPLE_WITHDRAW_DETAILS.signature,
		);

		const preAvailableBalance = await getAvailableBalance(ACTOR_ONE);
		const preWitheldBalance = await getWitheldBalance();
		// generate a cancel withdraw event
		await PDC_CANISTER.manual_publish(JSON.stringify([SAMPLE_CANCEL_EVENT]));
		// check balances again
		const postAvailableBalance = await getAvailableBalance(ACTOR_ONE);
		const postWitheldBalance = await getWitheldBalance();

		// validate that the balances are as they should be
		expect(postWitheldBalance.toString()).toEqual('0'); //make sure the witheld balance is now 0
		expect(postAvailableBalance.toString()).toEqual(
			(preWitheldBalance + preAvailableBalance).toString(),
		);
		expect(initialAvailableBalance.toString()).toEqual(postAvailableBalance.toString());
	});

	it('Can get the reciept of a succesfull withdrawal', async () => {
		const { token, chain, account, amount } = (await R_CANISTER.get_reciept(
			Principal.from(SAMPLE_DEPOSIT_EVENT.canister_id),
			NONCE,
		)).Ok;

		expect(token.toLowerCase()).toEqual(
			SAMPLE_DEPOSIT_EVENT.token.toLowerCase(),
		);
		expect(chain).toEqual(SAMPLE_DEPOSIT_EVENT.chain);
		expect(account.toLowerCase()).toEqual(
			SAMPLE_DEPOSIT_EVENT.account.toLowerCase(),
		);
		expect(amount.toString()).toEqual(`${SAMPLE_WITHDRAW_DETAILS.amount}`);
	});

	it('The DC Canister can adjust allocated to the Remittance Canister', async () => {
		const availableBalanceActorOnePre = await getAvailableBalance(ACTOR_ONE);
		const availableBalanceActorTwoPre = await getAvailableBalance(ACTOR_TWO);
		// simulate a deposit event
		await DC_CANISTER.manual_publish(JSON.stringify([...SAMPLE_ADJUST_EVENTS]));
		const availableBalanceActorOnePost = await getAvailableBalance(ACTOR_ONE);
		const availableBalanceActorTwoPost = await getAvailableBalance(ACTOR_TWO);

		expect(availableBalanceActorOnePost.toString()).toEqual(
			(availableBalanceActorOnePre - BigInt(ADJUST_AMOUNT)).toString(),
		);
		expect(availableBalanceActorTwoPost.toString()).toEqual(
			(availableBalanceActorTwoPre + BigInt(ADJUST_AMOUNT)).toString(),
		);
	});

	it('The DC Canister updates must always amount to zero or the balances wont be updated', async () => {
		const availableBalanceActorOnePre = await getAvailableBalance(ACTOR_ONE);
		const availableBalanceActorTwoPre = await getAvailableBalance(ACTOR_TWO);
		// simulate a deposit event
		await DC_CANISTER.manual_publish(
			JSON.stringify([...SAMPLE_ADJUST_EVENTS_NOT_RESOLVES_TO_ZERO]),
		);
		const availableBalanceActorOnePost = await getAvailableBalance(ACTOR_ONE);
		const availableBalanceActorTwoPost = await getAvailableBalance(ACTOR_TWO);

		// both should be the same since there would be no balance updates
		expect(availableBalanceActorOnePost.toString()).toEqual(availableBalanceActorOnePre.toString());
		expect(availableBalanceActorTwoPost.toString()).toEqual(availableBalanceActorTwoPre.toString());
	});
});
