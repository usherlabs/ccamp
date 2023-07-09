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

    event FundsDeposited(address indexed sender, uint amount);
    event FundsUnlocked(address indexed recipient, uint amount);
    event hash(bytes32 data);

    function initialize(address _remittanceCanister, string calldata _chainId) public initializer {
        __Ownable_init();
        __UUPSUpgradeable_init();
        __ReentrancyGuard_init();

        remittanceCanister = _remittanceCanister;
        chainId = _chainId;
        initialized = true;
    }

    receive() external payable {
        depositFunds(msg.value);
    }

    function depositFunds(uint amount) internal {
        emit FundsDeposited(msg.sender, amount);
    }

    function setRemittanceCanisterAddress(address _remittanceCanister) public onlyOwner{
        remittanceCanister = _remittanceCanister;
    }

    function unlockFunds(uint nonce, uint amount, bytes calldata signature) public nonReentrant {
        //TODO instead require that the contract has been initialized
        require(initialized, "CONTRACT_UNINITIALIZED");
        require(getBalance() >= amount, "AMOUNT > CONTRACT_BALANCE");
        require(!usedSignatures[signature], "USED_SIGNATURE");

        bytes32 dataHash = keccak256(abi.encodePacked(nonce, amount, msg.sender, chainId));
        emit hash(dataHash);
        require(validateSignature(dataHash, signature), "INVALID_SIGNATURE");

        usedSignatures[signature] = true;
        payable(msg.sender).transfer(amount);
        emit FundsUnlocked(msg.sender, amount);
    }

    function validateSignature(bytes32 dataHash, bytes calldata signature) internal view returns (bool isValid) {
        isValid = VerifySignature.verify(remittanceCanister, dataHash, signature);
    }

    function getBalance() public view returns (uint256) {
        return address(this).balance;
    }

    /// @dev required by the OZ UUPS module
    function _authorizeUpgrade(address) internal override onlyOwner {}
}
