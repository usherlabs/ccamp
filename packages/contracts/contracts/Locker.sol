// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.9;

import {IERC20Upgradeable} from "@openzeppelin/contracts-upgradeable/token/ERC20/IERC20Upgradeable.sol";
import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {ReentrancyGuardUpgradeable} from "@openzeppelin/contracts-upgradeable/security/ReentrancyGuardUpgradeable.sol";
import {ERC20Upgradeable} from "@openzeppelin/contracts-upgradeable/token/ERC20/ERC20Upgradeable.sol";
import {VerifySignature} from "./lib/VerifySignature.sol";

// Uncomment this line to use console.log
import "hardhat/console.sol";

contract Locker is Initializable, UUPSUpgradeable, OwnableUpgradeable, ReentrancyGuardUpgradeable {
    bool initialized;
    string public chainId;
    address public remittanceCanister;
    mapping(bytes => bool) usedSignatures;
    mapping(bytes32 => uint256) public canisters; //keccak256(principal) => amountDeposited

    event FundsDeposited(address indexed sender, uint amount, string canisterId);
    event FundsWithdrawn(address indexed account, address indexed recipient, uint amount);
    event WithdrawCanceled(address indexed account, uint amount, bytes32 signatureHash);

    function initialize(address _remittanceCanister, string calldata _chainId) public initializer {
        __Ownable_init();
        __UUPSUpgradeable_init();
        __ReentrancyGuard_init();

        remittanceCanister = _remittanceCanister;
        chainId = _chainId;
        initialized = true;
    }

    // make this function compatible with erc20
    function depositFunds(string calldata _canisterId) public payable {
        require(bytes(_canisterId).length == 27, "INVALID_CANISTERID");
        require(msg.value > 0, "MSG.VALUE == 0");

        canisters[keccak256(bytes(_canisterId))] += msg.value;

        emit FundsDeposited(msg.sender, msg.value, _canisterId);
    }

    function setRemittanceCanisterAddress(address _remittanceCanister) public onlyOwner {
        remittanceCanister = _remittanceCanister;
    }

    function validateSignature(bytes32 dataHash, bytes calldata signature) internal view returns (bool isValid) {
        isValid = VerifySignature.verify(remittanceCanister, dataHash, signature);
    }

    function getBalance() public view returns (uint256) {
        return address(this).balance;
    }

    function withdraw(uint nonce, uint amount, bytes calldata signature) public nonReentrant {
        withdrawTo(nonce, amount, signature, msg.sender);
    }

    // test
    function withdrawTo(uint nonce, uint amount, bytes calldata signature, address recipient) public {
        //TODO instead require that the contract has been initialized
        require(initialized, "CONTRACT_UNINITIALIZED");
        require(getBalance() >= amount, "AMOUNT > CONTRACT_BALANCE");
        require(!usedSignatures[signature], "USED_SIGNATURE");

        bytes32 dataHash = keccak256(abi.encodePacked(nonce, amount, msg.sender, chainId));
        require(validateSignature(dataHash, signature), "INVALID_SIGNATURE");

        usedSignatures[signature] = true;
        payable(recipient).transfer(amount);
        emit FundsWithdrawn(msg.sender, recipient, amount);
    }

    // test

    function cancelWithdraw(uint nonce, uint amount, bytes calldata signature) public {
        require(initialized, "CONTRACT_UNINITIALIZED");
        require(!usedSignatures[signature], "USED_SIGNATURE");

        // validate the signature
        bytes32 dataHash = keccak256(abi.encodePacked(nonce, amount, msg.sender, chainId));
        emit hash(dataHash);
        require(validateSignature(dataHash, signature), "INVALID_SIGNATURE");

        // mark signature as used
        usedSignatures[signature] = true;
        bytes32 signatureHash = keccak256(signature);
        // console.log(signatureHash);
    }

    /// @dev required by the OZ UUPS module
    function _authorizeUpgrade(address) internal override onlyOwner {}
}
