use k256::ecdsa::SigningKey;
use verity_client::client::{VerityClient, VerityClientConfig};

#[tokio::main()]
async fn main() -> Result<(), reqwest::Error> {
    println!("Proving a GET request using VerityClient...");

    let mut rng = rand::thread_rng();
    let signing_key = SigningKey::random(&mut rng);

    let config = VerityClientConfig {
        prover_url: String::from("http://127.0.0.1:8080"),
        prover_zmq: String::from("tcp://127.0.0.1:8000"),
        analysis_url: String::from("http://127.0.0.1:8000"),
        signing_key,
    };

    let json: serde_json::Value = VerityClient::new(config)
        .get("https://jsonplaceholder.typicode.com/posts/98")
        .redact(String::from("res:body:dolor"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    println!("{:#?}", json);

    Ok(())
}
