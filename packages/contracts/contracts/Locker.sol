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
// import "hardhat/console.sol";

contract Locker is Initializable, UUPSUpgradeable, OwnableUpgradeable, ReentrancyGuardUpgradeable {
    bool initialized;
    string public chainId;
    address public remittanceCanister;

    mapping(bytes => bool) usedSignatures;
    mapping(bytes32 => mapping(address => uint256)) public canisters; //keccak256(principal) => tokenAddress => amountDeposited

    event FundsDeposited(string canisterId, address indexed account, uint amount);
    event FundsWithdrawn(string canisterId, address indexed account, address indexed recipient, uint amount);
    event WithdrawCanceled(string canisterId, address indexed account, uint amount, bytes32 signatureHash);
    event UpdateRemittanceCanister(address remittanceCanister);

    function initialize(address _remittanceCanister, string calldata _chainId) public initializer {
        __Ownable_init();
        __UUPSUpgradeable_init();
        __ReentrancyGuard_init();

        remittanceCanister = _remittanceCanister;
        chainId = _chainId;
        initialized = true;
    }

    function depositFunds(string calldata _canisterId, uint256 _amount, address _token) public payable {
        require(bytes(_canisterId).length == 27, "INVALID_CANISTERID");
        require(_amount > 0, "amount == 0");

        IERC20Upgradeable(_token).transferFrom(msg.sender, address(this), _amount);
        canisters[keccak256(bytes(_canisterId))][_token] += _amount;

        emit FundsDeposited(_canisterId, msg.sender, _amount);
    }

    function setRemittanceCanisterAddress(address _remittanceCanister) public onlyOwner {
        remittanceCanister = _remittanceCanister;
        emit UpdateRemittanceCanister(_remittanceCanister);
    }

    function validateSignature(bytes32 dataHash, bytes calldata signature) internal view returns (bool isValid) {
        isValid = VerifySignature.verify(remittanceCanister, dataHash, signature);
    }

    function getBalance(string calldata _canisterId, address _token) public view returns (uint256 balance) {
        balance = canisters[keccak256(bytes(_canisterId))][_token];
    }

    function withdraw(
        string calldata _canisterId,
        address _token,
        uint _nonce,
        uint _amount,
        bytes calldata _signature
    ) public nonReentrant {
        withdrawTo(_canisterId, _token, _nonce, _amount, _signature, msg.sender);
    }

    function withdrawTo(
        string calldata _canisterId,
        address _token,
        uint _nonce,
        uint _amount,
        bytes calldata _signature,
        address _recipient
    ) public {
        require(initialized, "CONTRACT_UNINITIALIZED");
        require(getBalance(_canisterId, _token) >= _amount, "WITHDRAW_AMOUNT > CONTRACT_BALANCE");
        require(!usedSignatures[_signature], "USED_SIGNATURE");

        bytes32 dataHash = keccak256(abi.encodePacked(_nonce, _amount, msg.sender, chainId, _canisterId, _token));
        require(validateSignature(dataHash, _signature), "INVALID_SIGNATURE");

        usedSignatures[_signature] = true;
        IERC20Upgradeable(_token).transfer(_recipient, _amount);

        emit FundsWithdrawn(_canisterId, msg.sender, _recipient, _amount);
    }

    function cancelWithdraw(
        string calldata _canisterId,
        address _token,
        uint _nonce,
        uint _amount,
        bytes calldata _signature
    ) public {
        require(initialized, "CONTRACT_UNINITIALIZED");
        require(!usedSignatures[_signature], "USED_SIGNATURE");

        // validate the signature
        bytes32 dataHash = keccak256(abi.encodePacked(_nonce, _amount, msg.sender, chainId, _canisterId, _token));
        require(validateSignature(dataHash, _signature), "INVALID_SIGNATURE");

        // mark signature as used
        usedSignatures[_signature] = true;
        bytes32 sigHash = keccak256(_signature);
        emit WithdrawCanceled(_canisterId, msg.sender, _amount, sigHash);
    }

    /// @dev required by the OZ UUPS module
    function _authorizeUpgrade(address) internal override onlyOwner {}
}
