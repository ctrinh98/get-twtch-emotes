extern crate dotenv;

use dotenv::dotenv;
use reqwest;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE, ACCEPT, AUTHORIZATION,};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize, Deserialize, Debug)]
struct Images {
    url_1x: String,
    url_2x: String,
    url_4x: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct EmoteRaw {
    id: String,
    name: String,
    images: Images,
    tier: String,
    emote_type: String,
    emote_set_id: String
}

#[derive(Serialize, Deserialize, Debug)]
struct Items<T> {
    items: Vec<T>,
}

#[derive(Serialize, Deserialize, Debug)]
struct APIResponse {
    data: Vec<EmoteRaw>,
}

#[derive(Serialize, Deserialize, Debug)]
struct APIResponse2 {
    data: Vec<Channel>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Channel {
    id: String,
    login: String,
    display_name: String,
    broadcaster_type: String,
    profile_image_url: String,
    created_at: String
}

// tokio let's us use "async" on our main function
#[tokio::main]
async fn main() {
    dotenv().ok();
    let args: Vec<String> = env::args().collect();
    let search_query = &args[1];
    let url = format!(
        // "https://api.twitch.tv/helix/users?login=koragi_ch",
        "https://api.twitch.tv/helix/chat/emotes?broadcaster_id={query}",
        // query = "594387440"
        query = search_query
    );

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .headers(construct_headers())
        .send()
        .await
        .unwrap();
    match response.status() {
        reqwest::StatusCode::OK => {
            // on success, parse our JSON to an APIResponse
            match response.json::<APIResponse>().await {
                Ok(parsed) => print_emotes(parsed.data.iter().collect()),
                Err(_) => println!("Hm, the response didn't match the shape we expected."),
            };
        }
        reqwest::StatusCode::UNAUTHORIZED => {
            println!("Need to grab a new token");
        }
        other => {
            panic!("Uh oh! Something unexpected happened: {:?}", other);
        }
    };
}

fn print_emotes(emotes: Vec<&EmoteRaw>) {
    let mut v_refs: Vec<Emote> = Vec::new(); 

    for emote in emotes {
        println!("✏️ {}", emote.name);
        println!("🖼️ {}", emote.images.url_2x);
        println!("---------");
        let emote_iter = Emote {
            name: emote.name.clone(),
            url: emote.images.url_2x.clone(),
        };
        v_refs.push(emote_iter);


    }
    println!("{:?}", v_refs);
}

#[derive(Debug, Clone)]
pub struct Emote {
    pub name: String,
    pub url: String,
}

fn construct_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_static("Bearer <SECRET>"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

    headers.insert(HeaderName::from_static("client-id"), HeaderValue::from_static("<CLIENT_ID>"));
    headers
}