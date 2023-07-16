echo "installing the internet computer's cli tools"
sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)"

echo "installing rustup dependencies"
rustup target add wasm32-unknown-unknown