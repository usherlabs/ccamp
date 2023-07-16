#!/bin/bash

# deploy.sh
dfx stop
dfx start --background --clean
dfx deploy
dfx generate