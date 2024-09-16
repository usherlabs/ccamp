#[tokio::main()]
async fn main() -> Result<(), reqwest::Error> {
    let json: serde_json::Value = reqwest::Client::new()
        .post("https://jsonplaceholder.typicode.com/posts")
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
