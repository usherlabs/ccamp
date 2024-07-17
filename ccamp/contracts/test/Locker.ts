import assert from 'assert';
import { expect } from 'chai';
import { BigNumberish, Contract, ethers, Signer } from 'ethers';
import { ethers as hEthers } from 'hardhat';

import {
	chainId,
	ERROR_MESSAGES,
	nonce,
	remittanceCanisterPrincipal,
	testTokenAddress,
} from './utils/constants';
import {
	fetchEventArgsFromTx,
	generateHashAndSignature,
	getERC20Token,
	loadLockerContract,
	mintTokenAndApproveLocker,
} from './utils/functions';

describe('Locker', function () {
	let lockerContract: Contract;
	let tokenContract: Contract;
	let allSigners: Signer[];
	let adminSigner: Signer;
	let canisterSigner: Signer;
	const encoder = new TextEncoder();

	beforeEach(async () => {
		allSigners = await hEthers.getSigners();
		adminSigner = allSigners[0];
		canisterSigner = allSigners[5];

		const canisterAddress = await canisterSigner.getAddress();
		lockerContract = await loadLockerContract(canisterAddress, adminSigner);

		tokenContract = getERC20Token(adminSigner);
		await mintTokenAndApproveLocker(lockerContract.address);
	});

	it('should deposit funds and emit event', async () => {
		const depositedAmount = ethers.utils.parseEther('0.5');
		const adminAddress = await adminSigner.getAddress();

		const responseTx = await lockerContract.depositFunds(
			remittanceCanisterPrincipal,
			depositedAmount,
			testTokenAddress
		);
		const depositEvent = await fetchEventArgsFromTx(
			responseTx,
			'FundsDeposited'
		);
		const [canisterId, account, amount] = depositEvent || [];

		const lockerContractTokenBalance = await tokenContract.balanceOf(
			lockerContract.address
		);

		const canisterBalance = await lockerContract.canisters(
			hEthers.utils.keccak256(encoder.encode(remittanceCanisterPrincipal)),
			testTokenAddress
		);

		expect(canisterId).to.equal(remittanceCanisterPrincipal);
		expect(+amount)
			.to.equal(+depositedAmount)
			.to.equal(+canisterBalance)
			.to.greaterThanOrEqual(+lockerContractTokenBalance);
		expect(account).to.equal(adminAddress);
	});

	it('should unlock funds with valid signature and emit event', async () => {
		const depositedAmount = ethers.utils.parseEther('0.5');
		const recipient = await adminSigner.getAddress();

		await lockerContract.depositFunds(
			remittanceCanisterPrincipal,
			depositedAmount,
			testTokenAddress
		);

		const recipientPreBalance = await tokenContract.balanceOf(recipient);

		// generate request and signature to unlock funds
		const { signature } = await generateHashAndSignature(
			nonce,
			depositedAmount,
			recipient,
			chainId,
			remittanceCanisterPrincipal,
			testTokenAddress,
			canisterSigner
		);
		// make request
		const withdrawTx = await lockerContract.withdraw(
			remittanceCanisterPrincipal,
			testTokenAddress,
			nonce,
			depositedAmount,
			signature
		);
		const withdrawEvent = await fetchEventArgsFromTx(
			withdrawTx,
			'FundsWithdrawn'
		);
		const recipientPostBalance = await tokenContract.balanceOf(recipient);
		const [canisterId, account, fundsRecipient, amount] = withdrawEvent || [];

		// validate event
		expect(canisterId).to.equal(remittanceCanisterPrincipal);
		expect(account).to.equal(recipient).equal(fundsRecipient);
		expect(amount.toString()).to.equal(depositedAmount.toString());

		// validate funds were sent to recipient
		expect(recipientPostBalance.toString()).to.equal(
			(+recipientPreBalance + +depositedAmount).toString()
		);
	});

	it('should revert when unlocking funds with invalid signature from wrong amount', async () => {
		const recipient = await adminSigner.getAddress();

		// deposit 1eth into the pool
		await lockerContract.depositFunds(
			remittanceCanisterPrincipal,
			ethers.utils.parseEther('1.0'),
			testTokenAddress
		);

		// generate a signature for 0.5eth
		const { signature } = await generateHashAndSignature(
			nonce,
			ethers.utils.parseEther('0.5'),
			recipient,
			chainId,
			remittanceCanisterPrincipal,
			testTokenAddress,
			canisterSigner
		);

		// try to withdraw 0.7eth
		await expect(
			lockerContract.withdraw(
				remittanceCanisterPrincipal,
				testTokenAddress,
				nonce,
				ethers.utils.parseEther('0.7'),
				signature
			)
		).to.revertedWith(ERROR_MESSAGES.INVALID_SIGNATURE);
	});

	it('should revert when unlocking funds with invalid signature to wrong recipient', async () => {
		const amount = ethers.utils.parseEther('1.0');
		const recipient = await canisterSigner.getAddress();

		// deposit 1eth into the pool
		await lockerContract.depositFunds(
			remittanceCanisterPrincipal,
			ethers.utils.parseEther('1.0'),
			testTokenAddress
		);

		const { signature } = await generateHashAndSignature(
			nonce,
			amount,
			recipient,
			chainId,
			remittanceCanisterPrincipal,
			testTokenAddress,
			canisterSigner
		);

		await expect(
			lockerContract.withdraw(
				remittanceCanisterPrincipal,
				testTokenAddress,
				0,
				amount,
				signature
			)
		).to.revertedWith(ERROR_MESSAGES.INVALID_SIGNATURE);
	});

	it('should revert when unlocking funds with amount greater than contract balance of provided dc canister', async () => {
		const amountToDeposit = ethers.utils.parseEther('1.0');
		const amountToWithdraw = ethers.utils.parseEther('2.0');
		const recipient = await adminSigner.getAddress();

		// deposit 1eth into the pool
		await lockerContract.depositFunds(
			remittanceCanisterPrincipal,
			amountToDeposit,
			testTokenAddress
		);

		const { signature } = await generateHashAndSignature(
			0,
			amountToWithdraw,
			recipient,
			chainId,
			remittanceCanisterPrincipal,
			testTokenAddress,
			canisterSigner
		);

		await expect(
			lockerContract.withdraw(
				remittanceCanisterPrincipal,
				testTokenAddress,
				nonce,
				amountToWithdraw,
				signature
			)
		).to.be.revertedWith(ERROR_MESSAGES.INVALID_AMOUNT);
	});

	it('should revert when a signature is used multiple times with same nonce', async () => {
		const depositedAmount = ethers.utils.parseEther('0.5');
		const recipient = await adminSigner.getAddress();

		await lockerContract.depositFunds(
			remittanceCanisterPrincipal,
			depositedAmount,
			testTokenAddress
		);

		// generate request and signature to unlock funds
		const { signature } = await generateHashAndSignature(
			nonce,
			depositedAmount,
			recipient,
			chainId,
			remittanceCanisterPrincipal,
			testTokenAddress,
			canisterSigner
		);
		// make request
		await lockerContract.withdraw(
			remittanceCanisterPrincipal,
			testTokenAddress,
			nonce,
			depositedAmount,
			signature
		);

		await expect(
			lockerContract.withdraw(
				remittanceCanisterPrincipal,
				testTokenAddress,
				nonce,
				depositedAmount,
				signature
			)
		).to.revertedWith(ERROR_MESSAGES.USED_SIGNATURE);
	});

	it('should blacklist signature after canceling withdrawal', async () => {
		const depositedAmount = ethers.utils.parseEther('0.5');
		const recipient = await adminSigner.getAddress();
		const adminAccount = await adminSigner.getAddress();

		await lockerContract.depositFunds(
			remittanceCanisterPrincipal,
			depositedAmount,
			testTokenAddress
		);

		// generate request and signature to unlock funds
		const { signature } = await generateHashAndSignature(
			nonce,
			depositedAmount,
			recipient,
			chainId,
			remittanceCanisterPrincipal,
			testTokenAddress,
			canisterSigner
		);

		// make request
		const withdrawTx = await lockerContract.cancelWithdraw(
			remittanceCanisterPrincipal,
			testTokenAddress,
			nonce,
			depositedAmount,
			signature
		);
		const onCancelEvent = await fetchEventArgsFromTx(
			withdrawTx,
			'WithdrawCanceled'
		);

		const [canisterId, account, amountCanceled, signatureHash] =
			onCancelEvent || [];

		expect(canisterId).to.equal(remittanceCanisterPrincipal);
		expect(account).to.equal(adminAccount);
		expect(amountCanceled).to.equal(depositedAmount);
		expect(signatureHash).to.equal(hEthers.utils.keccak256(signature));

		// try to withdraw using this signature again and get an error
			await expect(
				lockerContract.withdraw(
					remittanceCanisterPrincipal,
					testTokenAddress,
					nonce,
					depositedAmount,
					signature
				)
			).to.revertedWith(ERROR_MESSAGES.USED_SIGNATURE);
	});

	it('should be able to change remittance canister address', async () => {
		const newCanisterAddress = await allSigners[4].getAddress();
		const setRCanisterTx = await lockerContract.setRemittanceCanisterAddress(
			newCanisterAddress
		);

		const [setCanisterEvent] =
			(await fetchEventArgsFromTx(
				setRCanisterTx,
				'UpdateRemittanceCanister'
			)) || [];

		expect(setCanisterEvent).to.equal(newCanisterAddress);
	});
});
