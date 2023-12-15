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
    address ZER0_ADDRESS = 0x0000000000000000000000000000000000000000;

    mapping(bytes => bool) usedSignatures;
    mapping(bytes32 => mapping(address => uint256)) public canisters; //keccak256(principal) => tokenAddress => amountDeposited

    event FundsDeposited(string canisterId, address indexed account, uint amount, string chain, address token);
    event FundsWithdrawn(string canisterId, address indexed account, uint amount, string chain, address token);
    event WithdrawCanceled(string canisterId, address indexed account, uint amount, string chain, address token);
    event UpdateRemittanceCanister(address remittanceCanister);

    function depositToken(string calldata _canisterId) public nonReentrant payable {
        uint256 _amount = msg.value;
        address _token = ZER0_ADDRESS;

        require(bytes(_canisterId).length == 27, "INVALID_CANISTERID");
        require(_amount > 0, "amount == 0");

        canisters[keccak256(bytes(_canisterId))][_token] += _amount;

        emit FundsDeposited(_canisterId, msg.sender, _amount, chainId, _token);
    }

    function initialize(address _remittanceCanister, string calldata _chainId) public initializer {
        __Ownable_init();
        __UUPSUpgradeable_init();
        __ReentrancyGuard_init();

        remittanceCanister = _remittanceCanister;
        chainId = _chainId;
        initialized = true;
    }

    function depositFunds(string calldata _canisterId, uint256 _amount, address _token) public nonReentrant payable returns(bool) {
        require(bytes(_canisterId).length == 27, "INVALID_CANISTERID");
        require(_amount > 0, "amount == 0");

        canisters[keccak256(bytes(_canisterId))][_token] += _amount;

        emit FundsDeposited(_canisterId, msg.sender, _amount, chainId, _token);
        bool response = IERC20Upgradeable(_token).transferFrom(msg.sender, address(this), _amount);

        return response;
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
    ) public nonReentrant returns(bool) {
        bool success = withdrawTo(_canisterId, _token, _nonce, _amount, _signature, msg.sender);
        return success;
    }

    function withdrawTo(
        string calldata _canisterId,
        address _token,
        uint _nonce,
        uint _amount,
        bytes calldata _signature,
        address _recipient
    ) public nonReentrant returns(bool) {
        require(initialized, "CONTRACT_UNINITIALIZED");
        require(getBalance(_canisterId, _token) >= _amount, "WITHDRAW_AMOUNT > CONTRACT_BALANCE");
        require(!usedSignatures[_signature], "USED_SIGNATURE");

        bytes32 dataHash = keccak256(abi.encodePacked(_nonce, _amount, msg.sender, chainId, _canisterId, _token));
        require(validateSignature(dataHash, _signature), "INVALID_SIGNATURE");

        usedSignatures[_signature] = true;

        emit FundsWithdrawn(_canisterId, msg.sender, _amount, chainId, _token);
        bool success = IERC20Upgradeable(_token).transfer(_recipient, _amount);
        return success;
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
        emit WithdrawCanceled(_canisterId, msg.sender, _amount,chainId, _token);
    }

    /// @dev required by the OZ UUPS module
    function _authorizeUpgrade(address) internal override onlyOwner {}
}
