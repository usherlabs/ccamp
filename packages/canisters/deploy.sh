# deploy.sh
dfx stop
dfx start --background --clean
dfx generate
dfx deploy