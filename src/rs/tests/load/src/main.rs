use k256::ecdsa::SigningKey;
use verity_client::client::{VerityClient, VerityClientConfig};

#[tokio::main()]
async fn main(){
    let mut count = 0;
    while count < 20 {
        let _ = f(&count).await;
        count += 1;
    }
}

async fn f(count: &i32) -> Result<(), reqwest::Error> {
    println!("{} :: Proving a GET request using VerityClient...", count);

    // let mut rng = rand::thread_rng();
    // let _signing_key = SigningKey::random(&mut rng);

    let config = VerityClientConfig {
        prover_url: String::from("http://127.0.0.1:8080"),
        prover_zmq: String::from("tcp://127.0.0.1:8000"),
        analysis: None,
    };

    let client = VerityClient::new(config);

    let json: serde_json::Value = client
        .get("https://jsonplaceholder.typicode.com/posts/98")
        .redact(String::from("res:body:dolor"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    println!("{} :: Response: {:?}", count, json);

    Ok(())
}