### [Smart Contracts](https://github.com/usherlabs/ccamp/tree/main/packages/contracts)
The smart contract consists of a [`Locker`](https://github.com/usherlabs/ccamp/blob/main/packages/contracts/contracts/Locker.sol) contract who's main purpose is to serve as a means to deposit funds into the protocol, withdraw funds from the protocol and cancel a pending withdrawal request made from the canisters.

#### Smart Contracts Overview
#### Initialise contract
```
function initialize(address _remittanceCanister, string calldata _chainId)

**parameters*
address _remittanceCanister: This is the public key of the remittance canister we wish to use to request for withdrawals.
string _chainId: This is the chain id of the contract which is a combination of the chain name and the chain id(if applicable). e.g "ethereum:1", "ethereum:5", "polygon:137"
```

#### Get Canister Balance
```
function getBalance(string calldata _canisterId, address _token) 

**parameters*
string _canisterId: This is a string representation of the principal of the data collection canister we want to fetch the balance of.
address _token: This is the address of the token which we want to get the balance of the canister of
```

#### Withdraw token
Withdraw tokens from the smart contract with parameters obtained from the canisters
```
function withdraw(
  string calldata _canisterId,
  address _token,
  uint _nonce,
  uint _amount,
  bytes calldata _signature
)

**parameters*
string _canisterId: This is a string representation of the principal of the data collection canister we want to fetch the balance of.
address _token: This is the address of the token which we want to get the balance of the canister of.
uint _nonce: This is a value gotten from the smart contract and is provided into this method.
uint _amount: This is the amount to withdraw.
bytes calldata _signature: the signature provided by the canister when a request for withdrawal is made
```

#### Withdraw token to address
Withdraw tokens from the smart contract with parameters obtained from the canisters
```
function withdrawTo(
  string calldata _canisterId,
  address _token,
  uint _nonce,
  uint _amount,
  bytes calldata _signature,
  address _recipient
)

**parameters*
string _canisterId: This is a string representation of the principal of the data collection canister we want to fetch the balance of.
address _token: This is the address of the token which we want to get the balance of the canister of.
uint _nonce: This is a value gotten from the smart contract and is provided into this method.
uint _amount: This is the amount to withdraw.
bytes calldata _signature: the signature provided by the canister when a request for withdrawal is made
address _recipient: The address of the intended recipient
```


#### Cancel token withdrawal request
Cancel a withdrawal request which was made from the remittance canister.
```
function cancelWithdraw(
  string calldata _canisterId,
  address _token,
  uint _nonce,
  uint _amount,
  bytes calldata _signature
)

**parameters*
string _canisterId: This is a string representation of the principal of the data collection canister we want to fetch the balance of.
address _token: This is the address of the token which we want to get the balance of the canister of.
uint _nonce: This is a value gotten from the smart contract and is provided into this method.
uint _amount: This is the amount to withdraw.
bytes calldata _signature: the signature provided by the canister when a request for withdrawal is made