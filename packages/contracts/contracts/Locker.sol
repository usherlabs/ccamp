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
    event WithdrawCanceled(address indexed account, uint amount, bytes signatureHash);

    function initialize(address _remittanceCanister, string calldata _chainId) public initializer {
        __Ownable_init();
        __UUPSUpgradeable_init();
        __ReentrancyGuard_init();

        remittanceCanister = _remittanceCanister;
        chainId = _chainId;
        initialized = true;
    }

    //TODO make this function compatible with erc20
    function depositFunds(string calldata _canisterId) payable public {
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

    function getBalance(string calldata _canisterId) public view returns (uint256 balance) {
        balance = canisters[keccak256(bytes(_canisterId))];
    }

    function withdraw(string calldata _canisterId, uint nonce, uint amount, bytes calldata signature) public nonReentrant {
        withdrawTo(_canisterId, nonce, amount, signature, msg.sender);
    }

    function withdrawTo(string calldata _canisterId, uint _nonce, uint _amount, bytes calldata _signature, address _recipient) public {
        require(initialized, "CONTRACT_UNINITIALIZED");
        require(getBalance(_canisterId) >= _amount, "AMOUNT > CONTRACT_BALANCE");
        require(!usedSignatures[_signature], "USED_SIGNATURE");

        bytes32 dataHash = keccak256(abi.encodePacked(_nonce, _amount, msg.sender, chainId,_canisterId));
        require(validateSignature(dataHash, _signature), "INVALID_SIGNATURE");

        usedSignatures[_signature] = true;
        payable(_recipient).transfer(_amount);
        emit FundsWithdrawn(msg.sender, _recipient,  _amount);
    }

    function cancelWithdraw(string calldata _canisterId, uint _nonce, uint _amount, bytes calldata _signature) public {
        require(initialized, "CONTRACT_UNINITIALIZED");
        require(!usedSignatures[_signature], "USED_SIGNATURE");

        // validate the signature
        bytes32 dataHash = keccak256(abi.encodePacked(_nonce, _amount, msg.sender, chainId, _canisterId));
        require(validateSignature(dataHash, _signature), "INVALID_SIGNATURE");

        // mark signature as used
        usedSignatures[_signature] = true;
        emit WithdrawCanceled(msg.sender, _amount, _signature);
    }

    /// @dev required by the OZ UUPS module
    function _authorizeUpgrade(address) internal override onlyOwner {}
}
