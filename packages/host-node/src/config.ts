export const environment = {
    dev : {
        agentUrl : "http://127.0.0.1:4943",
        canisterIdPath : "@ccamp/canisters/.dfx/local/canister_ids.json"
    },
    release : {
        agentUrl : "https://icp-api.io",
        canisterIdPath : "@ccamp/canisters/canister_ids.json"
    }
}