use std::env;

use dotenvy;
use qdrant_client::{
    Qdrant,
    qdrant::{CreateCollectionBuilder, Distance, VectorParamsBuilder},
};

static COLLECTION_NAME: &'static str = "lecturers";

#[tokio::main]
async fn main() {
    dotenvy::dotenv().unwrap();

    println!("Seed is running!");

    let cluster_endpoint =
        env::var("QDRANT_CLUSTER_ENDPOINT").unwrap_or_else(|_| "http://127.0.0.1:6333".to_string());
    let api_key = env::var("QDRANT_API_KEY").unwrap_or_else(|_| "not_needed".to_string());

    let client = Qdrant::from_url(cluster_endpoint.as_str())
        .api_key(api_key)
        .build()
        .unwrap();
    let collection_exist = client.collection_exists(COLLECTION_NAME).await.unwrap();
    if !collection_exist {
        let response = client
            .create_collection(CreateCollectionBuilder::new(COLLECTION_NAME).vectors_config(
                VectorParamsBuilder::new(1024, Distance::Cosine),
            )).await;
        response.unwrap();
    }

    println!("Seed is done!")
}
