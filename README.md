# get-twtch-emotes

Basic CLI tool to grab Twitch emotes from a channel with high quality images.

## Example

The program extracts the list of emotes obtainable from the Twitch API for a specific channel ID. We use the `reqwest` library primarily to call the API and retrieve the API response; two API calls are included in `main.rs` (one to retrieve the channel ID for a channel name, and another to retreive the channel emotes for a channel ID), but only the API call for the channel emotes is currently active.

Assuming you've obtained the channel ID named `594387440` for koragi_ch and you have valid API keys, you can run the following:

```sh
cargo run 594387440
```

This command will:

* Generate a list of emotes in the following format of <emote name> followed by <emote image url> and separated with hyphens, as shown below
* Turn each emote into an `Emote` struct and add a reference to each into a `Vec` array

``` 
✏️ koragiShock
🖼️ https://static-cdn.jtvnw.net/emoticons/v2/307107604/static/light/2.0
---------
✏️ koragiPeek
🖼️ https://static-cdn.jtvnw.net/emoticons/v2/emotesv2_4e8aa60e4ea94caa8b326a4b79aab168/static/light/2.0
---------
✏️ koragiTreat
🖼️ https://static-cdn.jtvnw.net/emoticons/v2/307909747/static/light/2.0
---------
```

## Future

This project served as a quick way to solve a problem I was having in obtaining emotes from channels to use for promotional purposes with respect to the channel owner. I hope to add the following features in the future:
* learn how .env works in Rust and allowing users to place their API keys in there instead of `main.rs` (top priority)
* expanding the CLI to allow for users to search for emotes by passing in the channel name as an argument instead of the channel ID, and two API calls will be done in sequence in order to retrieve the channel ID and then the emotes immediately after
* adding support for the Elgato Stream Deck, which will show the channel's emotes on the deck; clicking one emote button should type in the corresponding emote in the chat