mod profile;

use reqwest;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE, ACCEPT, AUTHORIZATION,};
use serde::{Deserialize, Serialize};
// use std::env;
use crate::profile::{DeviceModel, ProfilesWithImages};
use color_eyre::eyre::{bail, Result, WrapErr};
use fs_extra::dir::CopyOptions;
use serde_json::Value;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;
use structopt::StructOpt;
use tracing::{info, warn};
use uuid::Uuid;


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
    let url = format!(
        // "https://api.twitch.tv/helix/users?login=koragi_ch",
        "https://api.twitch.tv/helix/chat/emotes?broadcaster_id=594387440",
        // go check out her latest album. It's 🔥
        // query = "koragi_ch"
    );
    // the rest is the same as before!
    // let mut headers = reqwest::header::HeaderMap::new();
    // let name: HeaderName = "Authorization".parse().unwrap();
    // headers.insert(name, "Client-ID 3j7qlg41ffyzbi20g1nqq6lu1zmopu".parse().unwrap());

    // let name1: HeaderName = "Authorization".parse().unwrap();
    // headers.insert(name1, "Bearer toioibff8pu2ommc47i4d5lek4ym6o".parse().unwrap());

    // let name2: HeaderName = "Content_Type".parse().unwrap();
    // headers.insert(name2, "application/json".parse().unwrap());

    // let name3: HeaderName = "Accept".parse().unwrap();
    // headers.insert(name3, "application/json".parse().unwrap());

    // .header(AUTHORIZATION, "Bearer toioibff8pu2ommc47i4d5lek4ym6o")
    // .header(CONTENT_TYPE, "application/json")
    // .header(ACCEPT, "application/json")
    // .header(CLIENT_ID, "3j7qlg41ffyzbi20g1nqq6lu1zmopu")



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



    //     .send()
    //     .await
    //     .expect("failed to get response")
    //     .text()
    //     .await
    //     .expect("failed to get payload");
    // println!("Success! {:?}", response)
}

fn print_emotes(emotes: Vec<&EmoteRaw>) {
    let mut v_refs: Vec<Emote> = Vec::new(); 
    // let mut emote_iter = Emote;

    for emote in emotes {
        println!("🔥 {}", emote.name);
        println!("💿 {}", emote.images.url_2x);
        println!("---------");
        let mut emote_iter = Emote {
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
    headers.insert(AUTHORIZATION, HeaderValue::from_static("Bearer toioibff8pu2ommc47i4d5lek4ym6o"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

    headers.insert(HeaderName::from_static("client-id"), HeaderValue::from_static("3j7qlg41ffyzbi20g1nqq6lu1zmopu"));
    headers
}