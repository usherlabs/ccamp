# ICAF Canister Demo

**Note for MacOS**: `https://github.com/briansmith/ring/issues/1824#issuecomment-2059955073`
Please ensure `llvm` is installed and verify with `llvm-config --version`.

## 1. Execute local IC network

```bash
dfx start --clean
```

## 2. Deploy the ICAF canister

```bash
cd packages/canisters
dfx deploy ic_af
```

## 3. (Optional) Open the canister on a browser

Copy and paste the sample data `fixtures/data_package.json` and `fixtures/twitter_proof.json` we can get corresponding pairs of parsed data and its signature.

- [verify_data_proof](img/verify_data_proof.png)
- [verify_tls_proof](img/verify_tls_proof.png)

## 4. Run the Host Node Script

```bash
cd ../../packages/host-node
pnpm start
```

## 5. Review the signed JSON response

[JSON Response](../../../host-node/fixtures/twitter_data.json)
