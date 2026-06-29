use std::env;

use dotenvy;
use qdrant_client::Qdrant;

fn main() {
    dotenvy::dotenv().unwrap();

    println!("Seed is running!");

    let cluster_endpoint =
        env::var("QDRANT_CLUSTER_ENDPOINT").unwrap_or_else(|_| "http://127.0.0.1:6333".to_string());

    let api_key = env::var("QDRANT_API_KEY").unwrap_or_else(|_| "not_needed".to_string());
    let client = Qdrant::from_url(cluster_endpoint.as_str())
        .api_key(api_key)
        .build()
        .unwrap();
    let does_collection_exist = async || {
        client.collection_exists("{collection_name}").await
    };
    if does_collection_exist {
        
    }

    println!("Seed is done!")
}
