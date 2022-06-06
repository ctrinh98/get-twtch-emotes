use crate::profile::Emote;
use color_eyre::eyre::{bail, ContextCompat, Result, WrapErr};
use serde_json::Value;

pub fn parse_emotes(channel_id: &str) -> Result<Vec<Emote>> {
    
}