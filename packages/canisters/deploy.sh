#!/bin/bash

# deploy.sh
dfx start --background --clean
dfx generate
dfx deploy