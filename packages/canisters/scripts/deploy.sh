#!/bin/bash

# deploy.sh
dfx generate
dfx stop
dfx start --background --clean
dfx deploy