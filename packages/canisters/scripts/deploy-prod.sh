#!/bin/bash

# deploy.sh
# dfx generate
# dfx stop
# dfx start --background --clean

# deploy all canisters seperately in order to pass in the environment parameter to the canister
dfx deploy protocol_data_collection  --argument '(opt variant { Production } )'  --network ic 
dfx deploy data_collection --network ic 
dfx deploy remittance --argument '(opt variant { Production } )' --network ic
dfx deploy bridge_data_collection --network ic
dfx deploy token --network ic