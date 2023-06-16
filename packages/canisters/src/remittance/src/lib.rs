#[ic_cdk::query]
fn greet(name: String) -> String {
    format!("Hello remittance canister, {}!", name)
}
