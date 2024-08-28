#[tokio::main()]
async fn main() -> Result<(), reqwest::Error> {
    println!("Proving a GET request by calling prover's API endpoint...");

    let json: serde_json::Value = reqwest::Client::new()
        .get("http://127.0.0.1:8080/proxy")
        .header(
            "T-PROXY-URL",
            "https://jsonplaceholder.typicode.com/posts/98",
        )
        .header("T-REDACTED", String::from("res:body:dolor"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    println!("{:#?}", json);

    Ok(())
}
