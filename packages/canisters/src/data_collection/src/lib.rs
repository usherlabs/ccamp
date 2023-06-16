#[ic_cdk::query]
fn greet(name: String) -> String {
    format!("Hello data_collection canister, {}!", name)
}

    