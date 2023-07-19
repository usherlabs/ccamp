import assert from 'assert';
import { expect } from 'chai';
import { BigNumberish, Contract, ethers, Signer } from 'ethers';
import { ethers as hEthers } from 'hardhat';

import { chainId, ERROR_MESSAGES } from './utils/constants';
import {
	generateHashAndSignature,
	loadLockerContract,
} from './utils/functions';

describe('Locker', function () {
	let lockerContract: Contract;
	let allSigners: Signer[];
	let adminSigner: Signer;
	let canisterSigner: Signer;

	beforeEach(async () => {
		allSigners = await hEthers.getSigners();
		adminSigner = allSigners[0];
		canisterSigner = allSigners[5];

		const canisterAddress = await canisterSigner.getAddress();
		lockerContract = await loadLockerContract(canisterAddress, adminSigner);
	});

	async function sendEtherToLocker(amount: BigNumberish, signer: Signer) {
		// deposit some funds into the contract
		await signer.sendTransaction({
			to: lockerContract.address,
			value: amount,
		});
		// deposit some funds into the contract
	}

	it('should deposit funds and emit event', async () => {
		const depositedAmount = ethers.utils.parseEther('1.0');
		await sendEtherToLocker(depositedAmount, adminSigner);
		// filter for the last event sent from this contract
		const eventFilter = lockerContract.filters.FundsDeposited();
		const events = await lockerContract.queryFilter(eventFilter, -1);
		const { sender, amount } = { ...events[events.length - 1].args } as {
			sender: string;
			amount: number;
		};
		// filter for the last event sent from this contract

		// validate the parameters
		const adminAddress = await adminSigner.getAddress();
		expect(depositedAmount.toString()).to.equal(amount.toString());
		expect(sender).to.equal(adminAddress);
		// validate the parameters
	});

	it('should unlock funds with valid signature and emit event', async () => {
		const amount = ethers.utils.parseEther('1.0');
		const recipient = await adminSigner.getAddress();
		const userPreBalance = await adminSigner.getBalance();
		await sendEtherToLocker(amount, allSigners[11]);

		// generate request and signature to unlock funds
		const { signature } = await generateHashAndSignature(
			0,
			amount,
			recipient,
			chainId,
			canisterSigner
		);
		// make request
		await lockerContract.withdraw(0, amount, signature);
		// validate the balance of the contract is 0
		const contractBalance = await lockerContract.getBalance();
		assert.equal(+contractBalance, 0);
		// validate the balance of the recipient is increased
		const userPostBalance = await adminSigner.getBalance();
		expect(userPostBalance).greaterThan(userPreBalance);
	});

	it('should revert when unlocking funds with invalid signature from wrong amount', async () => {
		const amount = ethers.utils.parseEther('1.0');
		const recipient = await adminSigner.getAddress();

		await sendEtherToLocker(amount, allSigners[11]);

		const { signature } = await generateHashAndSignature(
			0,
			amount,
			recipient,
			chainId,
			canisterSigner
		);
		await expect(
			lockerContract.withdraw(0, ethers.BigNumber.from(10), signature)
		).to.revertedWith(ERROR_MESSAGES.INVALID_SIGNATURE);

		// Check contract balance remains unchanged
		const contractBalance = await lockerContract.getBalance();
		assert.equal(contractBalance.toString(), amount.toString());
	});

	it('should revert when unlocking funds with invalid signature to wrong recipient', async () => {
		const amount = ethers.utils.parseEther('1.0');
		const recipient = await canisterSigner.getAddress();

		await sendEtherToLocker(amount, allSigners[11]);

		const { signature } = await generateHashAndSignature(
			0,
			amount,
			recipient,
			chainId,
			canisterSigner
		);
		await expect(lockerContract.withdraw(0, amount, signature)).to.revertedWith(
			ERROR_MESSAGES.INVALID_SIGNATURE
		);

		// Check contract balance remains unchanged
		const contractBalance = await lockerContract.getBalance();
		assert.equal(contractBalance.toString(), amount.toString());
	});

	it('should revert when unlocking funds with amount greater than contract balance', async () => {
		const amountToDeposit = ethers.utils.parseEther('1.0');
		const amountToWithdraw = ethers.utils.parseEther('2.0');
		const recipient = await adminSigner.getAddress();

		await sendEtherToLocker(amountToDeposit, allSigners[11]);

		const { signature } = await generateHashAndSignature(
			0,
			amountToWithdraw,
			recipient,
			chainId,
			canisterSigner
		);
		await expect(
			lockerContract.withdraw(0, amountToWithdraw, signature)
		).to.be.revertedWith(ERROR_MESSAGES.INVALID_AMOUNT);

		// Check contract balance remains unchanged
		const contractBalance = await lockerContract.getBalance();
		assert.equal(contractBalance.toString(), amountToDeposit.toString());
	});

	it('should revert when a signature is used multiple times with same nonce', async () => {
		const nonce = 0;
		const withdrawAmount = ethers.utils.parseEther('1.0');
		const depositAmount = ethers.utils.parseEther('2.0');
		const recipient = await adminSigner.getAddress();
		const userPreBalance = await adminSigner.getBalance();
		await sendEtherToLocker(withdrawAmount, allSigners[11]);


		// generate request and signature to unlock funds
		const { signature } = await generateHashAndSignature(
			0,
			withdrawAmount,
			recipient,
			chainId,
			canisterSigner
		);

		// make request
		await lockerContract.withdraw(nonce, withdrawAmount, signature);
		await expect(
			lockerContract.withdraw(nonce, withdrawAmount, signature)
		).to.revertedWith(ERROR_MESSAGES.USED_SIGNATURE);
	});
});
