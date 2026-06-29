use std::{collections::HashMap, env};

use qdrant_client::{
    Payload, Qdrant,
    qdrant::{
        CreateCollectionBuilder, Distance, Document, PointStruct, UpsertPointsBuilder,
        VectorParamsBuilder,
    },
};
use reqwest;
use scraper::{Html, Selector};

static COLLECTION_NAME: &'static str = "lecturers";

static LECTURER_SLUGS: &[&'static str] = &[
    "narsimlu-kemsaram",
    "zati",
    "ckloo-um",
    "nuruljapar",
    "erma",
    "cs-chan",
    "cswoo",
    "shahreeza",
];

static TEMPLATE: &'static str = r#"
# LECTURER: {}

## Biography:
{}

## Area of Expertise:
{}
"#;

pub struct InsertionData {
    pub id: u64,
    pub slug: String,
    pub name: String,
    pub biography: String,
    pub areas_of_expertise: String,
    pub text: String,
}

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
    let qdrant_api_key = env::var("QDRANT_API_KEY").unwrap_or_else(|_| "not_needed".to_string());
    let openrouter_api_key =
        env::var("OPENROUTER_API_KEY").unwrap_or_else(|_| "not_needed".to_string());

    let qdrant_client = Qdrant::from_url(cluster_endpoint.as_str())
        .api_key(qdrant_api_key)
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
                    .vectors_config(VectorParamsBuilder::new(4096, Distance::Cosine)),
            )
            .await
            .unwrap();
    }

    let mut data_for_insertion: Vec<InsertionData> = Vec::new();
    for (i, element) in LECTURER_SLUGS.iter().enumerate() {
        let slug = *element;

        let reqwest_client = reqwest::Client::builder().build().unwrap();
        let request = reqwest_client.request(
            reqwest::Method::GET,
            format!("https://umexpert.um.edu.my/{}.html", slug).as_str(),
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
            .collect::<Vec<_>>()[0]
            .trim();

        let biography_selector = Selector::parse(
            "body > article > div > div.resume-body > div.row.cv-module-content > div > div",
        )
        .unwrap();
        let biography = document
            .select(&biography_selector)
            .next()
            .unwrap()
            .text()
            .collect::<Vec<_>>();
        let biography = unescape(biography.join(""));
        let biography = biography.trim();

        let expertise_selector =
            Selector::parse("body > article > div > div.resume-body > section > div > ul > li")
                .unwrap();
        let expertise_title_selector = Selector::parse(
            "body > article > div > div.resume-body > section > div > ul > li > div.resume-degree",
        )
        .unwrap();
        let expertise_item_selector = Selector::parse(
            "body > article > div > div.resume-body > section > div > ul > li > div.area-item",
        )
        .unwrap();
        let areas_of_expertise = document.select(&expertise_selector).map(|element| {
            let title = element
                .select(&expertise_title_selector)
                .next()
                .unwrap()
                .text()
                .collect::<Vec<_>>();
            let title = title.join("");

            let item = element
                .select(&expertise_item_selector)
                .next()
                .unwrap()
                .text()
                .collect::<Vec<_>>();
            let item = item.join("");

            return format!("{}: {}", title, item);
        });
        let areas_of_expertise: Vec<String> = areas_of_expertise
            .map(|item| format!("- {}", item))
            .collect();
        let areas_of_expertise = areas_of_expertise.join("\n");
        let areas_of_expertise = areas_of_expertise.as_str();

        let markdown = TEMPLATE
            .replacen("{}", name, 1)
            .replacen("{}", biography, 1)
            .replacen("{}", areas_of_expertise, 1);

        data_for_insertion.push(InsertionData {
            id: i as u64,
            slug: slug.into(),
            text: markdown,
            name: name.to_string(),
            biography: biography.to_string(),
            areas_of_expertise: areas_of_expertise.to_string(),
        });
        println!("Preparing slug {} for insertion", slug);
    }

    let points: Vec<PointStruct> = data_for_insertion
        .iter()
        .map(|p| {
            let mut payload = Payload::default();
            payload.insert("name", p.name.clone());
            payload.insert("biography", p.biography.clone());
            payload.insert("areas_of_expertise", p.areas_of_expertise.clone());

            return PointStruct::new(
                p.id,
                Document {
                    text: p.text.clone(),
                    model: "openrouter/qwen/qwen3-embedding-8b".into(),
                    options: HashMap::new(),
                },
                payload,
            );
        })
        .collect();
    qdrant_client
        .with_header("openrouter-api-key", openrouter_api_key)
        .upsert_points(UpsertPointsBuilder::new(COLLECTION_NAME, points).wait(true))
        .await
        .unwrap();

    println!("Seed is done!");
}
