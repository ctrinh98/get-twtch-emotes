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
async fn main() -> Result<()> {

    // walfie below

    color_eyre::install()?;
    tracing_subscriber::fmt::fmt().init();

    let mut args = Args::from_args();
    if let Some(prefix) = args.prefix.strip_prefix('_') {
        warn!(%prefix, "Ignoring leading underscore in prefix");
        args.prefix = prefix.to_owned();
    }

    // Find output path based on platform
    let root_path = if let Some(ref path) = args.out {
        path.clone()
    } else if let Some(home) = dirs::home_dir() {
        if cfg!(target_os = "macos") {
            home.join("Library")
                .join("Application Support")
                .join("com.elgato.StreamDeck")
                .join("ProfilesV2")
                .to_path_buf()
        } else if cfg!(target_os = "windows") {
            home.join("AppData")
                .join("Roaming")
                .join("Elgato")
                .join("StreamDeck")
                .join("ProfilesV2")
                .to_path_buf()
        } else {
            bail!("No output path specified")
        }
    } else {
        bail!("Could not find home directory")
    };

    // Parse HTML file to get list of emotes
    // let html = if args.html_file.to_str() == Some("-") {
    //     let mut buf = String::new();
    //     std::io::stdin().read_to_string(&mut buf)?;
    //     buf
    // } else {
    //     fs::read_to_string(&args.html_file)
    //         .with_context(|| format!("Failed to read file {:?}", &args.html_file))?
    // };

    // walfie above



    let url = format!(
        // "https://api.twitch.tv/helix/users?login=koragi_ch",
        "https://api.twitch.tv/helix/chat/emotes?broadcaster_id=594387440",
        // go check out her latest album. It's ????
        // query = "koragi_ch"
    );


    let mut emotes;
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
            let mut apiRe: APIResponse;
            match response.json::<APIResponse>().await {
                Ok(parsed) => apiRe = parsed,
                Err(_) => println!("Hm, the response didn't match the shape we expected."),
            };
            let emotes = print_emotes(apiRe.data.iter().collect());
        }
        reqwest::StatusCode::UNAUTHORIZED => {
            println!("Need to grab a new token");
        }
        other => {
            panic!("Uh oh! Something unexpected happened: {:?}", other);
        }
    };

    // more walfie below

    // Generate profiles
    let profiles = ProfilesWithImages::new(
        args.profile_uuid
            .unwrap_or_else(|| profile::uuid_v5(&args.name, 0)),
        args.model,
        args.device_uuid,
        args.name,
        emotes,
        &args.prefix,
        args.include_labels,
    )
    .await?;

    // Write profiles to filesystem
    let mut root_profiles_path = root_path.clone();
    let mut current_path = root_path;
    let mut depth = 0;

    let copy_options = CopyOptions {
        overwrite: true,
        copy_inside: true,
        ..Default::default()
    };

    for (uuid, manifest) in profiles.manifests {
        let sd_profile_dir = format!("{}.sdProfile", uuid.to_string().to_uppercase());

        if depth == 0 {
            root_profiles_path = current_path.join(&sd_profile_dir).join("Profiles");
        } else {
            // Nested profiles have an additional `Profiles` directory
            current_path.push("Profiles");
        }

        current_path.push(&sd_profile_dir);
        info!(path = ?current_path, "Creating profile directory");

        // After the initial profile installation, the Stream Deck application un-nests the
        // directories. The app seems to ignore changes that we make to the un-nested profiles, so
        // we have to move the directories back to the nested structure to make changes.
        if depth >= 2 {
            let src = root_profiles_path.join(&sd_profile_dir);
            if let Err(e) = fs_extra::dir::move_dir(&src, &current_path, &copy_options) {
                if !matches!(e.kind, fs_extra::error::ErrorKind::NotFound) {
                    warn!(error = %e, "Failed to move existing nested profile");
                }
            } else {
                info!(?src, dest = ?current_path, "Moved existing nested profile");
            }
        }

        fs::create_dir_all(&current_path)
            .with_context(|| format!("Failed to create path {:?}", &current_path))?;

        let manifest_path = current_path.join("manifest.json");
        let mut json = serde_json::to_value(&manifest)?;

        if !args.no_merge {
            if let Err(e) = merge_manifests_if_exists(&mut json, &manifest_path) {
                warn!(error = %e, path = ?manifest_path, "Failed to merge existing manifest file");
            }
        }

        fs::write(&manifest_path, serde_json::to_vec(&json)?)
            .with_context(|| format!("Failed to write file {:?}", &manifest_path))?;

        for (position, action) in manifest.actions.iter() {
            let img_path = current_path
                .join(format!("{},{}", position.x, position.y))
                .join("CustomImages");

            fs::create_dir_all(&img_path)
                .with_context(|| format!("Failed to create path {:?}", &img_path))?;

            let img_file_path = img_path.join("state0.png");
            if let Some(bytes) = &action.image {
                fs::write(&img_file_path, bytes)
                    .with_context(|| format!("Failed to write image {:?}", &img_file_path))?;
            }
        }

        depth += 1;
    }

    if args.restart {
        restart_stream_deck().context("Failed to restart Stream Deck application")?;
    }

    Ok(())

}

fn merge_manifests_if_exists(new_manifest: &mut Value, existing_path: &PathBuf) -> Result<()> {
    let string = match fs::read_to_string(existing_path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e).context("Could not read existing manifest file"),
        Ok(s) => s,
    };

    let old_manifest = serde_json::from_str::<Value>(&string).context("Invalid JSON")?;

    let old_actions = if let Some(actions) = old_manifest
        .pointer("/Actions")
        .and_then(|json| json.as_object())
    {
        actions
    } else {
        bail!("Existing manifest file has invalid `Actions` field");
    };

    let new_actions = if let Some(actions) = new_manifest
        .pointer_mut("/Actions")
        .and_then(|json| json.as_object_mut())
    {
        actions
    } else {
        bail!("New manifest file has invalid `Actions` field");
    };

    for (pos, action) in old_actions.into_iter() {
        if !new_actions.contains_key(pos) {
            new_actions.insert(pos.to_owned(), action.clone());
        }
    }

    Ok(())
}

fn restart_stream_deck() -> Result<()> {
    if !cfg!(target_os = "macos") {
        warn!(
            "The --restart flag is currently only supported on macOS. \
            See https://github.com/walfie/streamdeck-youtube-emotes/issues/1"
        );
        return Ok(());
    }

    info!("Restarting Stream Deck application");

    let stop_result = Command::new("pkill").arg("Stream Deck").status();

    if let Err(e) = stop_result {
        warn!(error = %e, "Failed to stop Stream Deck");
    }

    let start_result = Command::new("open")
        .arg("/Applications/Stream Deck.app")
        .status();

    if let Err(e) = start_result {
        warn!(error = %e, "Failed to start Stream Deck");
    }

    Ok(())
}

#[derive(StructOpt)]
pub struct Args {
    /// Path to an HTML file containing the memberships page for a channel.
    /// E.g., Download the following page in a browser while logged in:
    /// https://www.youtube.com/channel/UCP4nMSTdwU1KqYWu3UH5DHQ/memberships
    ///
    /// Use - to read from stdin.
    #[structopt(parse(from_os_str), long)]
    pub html_file: PathBuf,

    /// The emote prefix (also known as "family name"). For example, if the channel has an emote
    /// `:_pomuSmall9cm:`, the emote prefix would be `pomu`. For some channels, there is no prefix,
    /// so this option can be omitted.
    #[structopt(default_value = "", long)]
    pub prefix: String,

    /// Name of the Stream Deck profile. Note that if the `profile-uuid` argument is unspecified, this name will
    /// be used to determine the name of the output profile directory.
    #[structopt(long)]
    pub name: String,

    /// Device UUID for the Stream Deck
    #[structopt(default_value = "", long)]
    pub device_uuid: String,

    /// Override the UUID for the profile
    #[structopt(long)]
    pub profile_uuid: Option<Uuid>,

    /// Whether to include the name of the emote on each key
    #[structopt(long)]
    pub include_labels: bool,

    /// Overwrite existing manifest files instead of merging them.
    #[structopt(long)]
    pub no_merge: bool,

    /// Output path to save the profile to. If unspecified, profiles will be saved to the default
    /// Stream Deck profile location (depending on platform).
    #[structopt(long)]
    pub out: Option<PathBuf>,

    /// List of emotes that should appear first, before all others (case-insensitive)
    #[structopt(long)]
    pub prioritize: Vec<String>,

    /// List of emotes that should appear last, after all others (case-insensitive)
    #[structopt(long)]
    pub deprioritize: Vec<String>,

    /// The Stream Deck model to generate the profile for
    #[structopt(long, possible_values = &["standard", "xl", "mini"])]
    pub model: DeviceModel,

    /// Restart the Stream Deck application after creating the profile
    #[structopt(long)]
    pub restart: bool,
}

fn print_emotes(emotes: Vec<&EmoteRaw>) -> Vec<Emote> {
    let mut v_refs: Vec<Emote> = Vec::new(); 
    // let mut emote_iter = Emote;

    for emote in emotes {
        println!("???? {}", emote.name);
        println!("???? {}", emote.images.url_2x);
        println!("---------");
        let mut emote_iter = Emote {
            name: emote.name.clone(),
            url: emote.images.url_2x.clone(),
        };
        v_refs.push(emote_iter);


    }
    println!("{:?}", v_refs);
    v_refs
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