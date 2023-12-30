#!/bin/bash

# deploy.sh
# dfx generate
# dfx stop
# dfx start --background --clean
dfx deploy data_collection
dfx deploy protocol_data_collection
dfx deploy remittance