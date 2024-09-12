use k256::SecretKey;
use rand::rngs::OsRng;
use verity_client::client::{AnalysisConfig, VerityClient, VerityClientConfig};

#[tokio::main()]
async fn main() -> Result<(), reqwest::Error> {
    println!("Proving a POST request using VerityClient...");

    let secret_key = SecretKey::random(&mut OsRng);

    let config = VerityClientConfig {
        prover_url: String::from("http://127.0.0.1:8080"),
        prover_zmq: String::from("tcp://127.0.0.1:5556"),
        analysis: Some(AnalysisConfig {
            analysis_url: String::from("http://127.0.0.1:8888"),
            secret_key,
        }),
    };

    let json: serde_json::Value = VerityClient::new(config)
        .post(String::from("https://jsonplaceholder.typicode.com/posts"))
        .json(&serde_json::json!({
            "userId": 1000,
            "firstName": "John",
            "lastName": "Smith",
            "fullName": "John Smith",
            "favoriteActor": "Johnny Depp"
        }))
        .redact(String::from("req:body:firstName, res:body:firstName"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    println!("{:#?}", json);

    Ok(())
}
