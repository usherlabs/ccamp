#!/bin/bash

# deploy.sh
dfx generate
dfx stop
dfx start --background --clean

# deploy all canisters seperately in order to pass in the environment parameter to the canister
dfx deploy protocol_data_collection
dfx deploy data_collection
dfx deploy remittance --argument '(opt variant { Staging } )'