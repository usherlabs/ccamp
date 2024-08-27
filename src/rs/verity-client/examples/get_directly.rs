#[tokio::main()]
async fn main() -> Result<(), reqwest::Error> {
    let json: serde_json::Value = reqwest::Client::new()
        .get("https://jsonplaceholder.typicode.com/posts/98")
        .send()
        .await?
        .json()
        .await?;

    println!("{:#?}", json);

    Ok(())
}
