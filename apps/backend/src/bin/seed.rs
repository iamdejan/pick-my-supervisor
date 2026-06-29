use std::env;

use dotenvy;
use qdrant_client::{
    Qdrant,
    qdrant::{CreateCollectionBuilder, Distance, VectorParamsBuilder},
};
use reqwest;
use scraper::{Html, Selector};

static COLLECTION_NAME: &'static str = "lecturers";

static LECTURER_SLUGS: &[&str] = &["narsimlu-kemsaram", "zati", "ckloo-um", "sawsn"];

fn unescape(raw_input: String) -> String {
    raw_input
        .replace(r"\n", "\n")
        .replace(r"\t", "\t")
        .replace(r"\\", "\\")
}

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
    let document = Html::parse_document(body);
    let name_selector =
        Selector::parse("#personaldetails > div > div.profile-main > div.prof-name-row > div")
            .unwrap();
    let name = document
        .select(&name_selector)
        .next()
        .unwrap()
        .text()
        .collect::<Vec<_>>()[0];
    println!("name = {}", name);

    let biography_selector = Selector::parse("body > article > div > div.resume-body.ps-3.pe-4.pb-4.ms-4.pt-5 > div.row.cv-module-content.mr-4.mt-0.mb-0 > div > div").unwrap();
    let biography = document
        .select(&biography_selector)
        .next()
        .unwrap()
        .text()
        .collect::<Vec<_>>();
    let biography = unescape(biography.join(""));
    let biography = biography.trim();
    println!("biography = {}", biography);

    let expertise_selector = Selector::parse(
        "body > article > div > div.resume-body.ps-3.pe-4.pb-4.ms-4.pt-5 > section > div > ul > li",
    )
    .unwrap();
    let areas_of_expertise = document
        .select(&expertise_selector)
        .map(|element| element.text().collect::<String>());
    let areas_of_expertise: Vec<String> = areas_of_expertise.collect();
    println!("areas_of_expertise = {:?}", areas_of_expertise);

    println!("Seed is done!");
}
