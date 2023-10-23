#!/bin/bash

# deploy.sh
# dfx generate
# dfx stop
# dfx start --background --clean

# deploy all canisters seperately in order to pass in the environment parameter to the canister
dfx deploy --network ic protocol_data_collection
dfx deploy --network ic data_collection
dfx deploy --network ic remittance --argument '(opt variant { Production } )'