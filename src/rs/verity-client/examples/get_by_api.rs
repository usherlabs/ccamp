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
        .json(&serde_json::json!({
            "userId": 1000,
            "firstName": "John",
            "lastName": "Smith",
            "fullName": "John Smith",
            "favoriteActor": "Johnny Depp"
        }))
        .send()
        .await?
        .json()
        .await?;

    println!("{:#?}", json);

    Ok(())
}
