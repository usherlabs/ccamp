#[tokio::main()]
async fn main() -> Result<(), reqwest::Error> {
    println!("Proving a POST request by calling prover's API endpoint...");

    let json: serde_json::Value = reqwest::Client::new()
        .post("http://127.0.0.1:8080/proxy")
        .header("T-PROXY-URL", "https://jsonplaceholder.typicode.com/posts")
        .header(
            "T-REDACTED",
            String::from("req:body:firstName, res:body:firstName"),
        )
        .json(&serde_json::json!({
            "userId": 1000,
            "firstName": "John",
            "lastName": "Smith",
            "fullName": "John Smith",
            "favoriteActor": "Johnny Depp"
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    println!("{:#?}", json);

    Ok(())
}
