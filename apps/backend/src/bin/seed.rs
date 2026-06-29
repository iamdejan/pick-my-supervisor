use std::env;

use dotenvy;
use qdrant_client::{
    Qdrant,
    qdrant::{CreateCollectionBuilder, Distance, VectorParamsBuilder},
};
use reqwest;

static COLLECTION_NAME: &'static str = "lecturers";

static LECTURER_SLUGS: &[&str] = &["narsimlu-kemsaram", "zati", "ckloo-um", "sawsn"];

#[tokio::main]
async fn main() {
    dotenvy::dotenv().unwrap();

    println!("Seed is running!");

    let cluster_endpoint =
        env::var("QDRANT_CLUSTER_ENDPOINT").unwrap_or_else(|_| "http://127.0.0.1:6334".to_string());
    let api_key = env::var("QDRANT_API_KEY").unwrap_or_else(|_| "not_needed".to_string());

    let qdrant_client = Qdrant::from_url(cluster_endpoint.as_str())
        .api_key(api_key)
        .build()
        .unwrap();
    let collection_exist = qdrant_client
        .collection_exists(COLLECTION_NAME)
        .await
        .unwrap();
    if !collection_exist {
        qdrant_client
            .create_collection(
                CreateCollectionBuilder::new(COLLECTION_NAME)
                    .vectors_config(VectorParamsBuilder::new(1024, Distance::Cosine)),
            )
            .await
            .unwrap();
    }

    let reqwest_client = reqwest::Client::builder().build().unwrap();
    let request = reqwest_client.request(
        reqwest::Method::GET,
        format!("https://umexpert.um.edu.my/{}.html", LECTURER_SLUGS[0]).as_str(),
    );
    let response = request.send().await.unwrap();
    let body = response.text().await.unwrap();
    let body = body.trim();

    println!("Seed is done!");
}
