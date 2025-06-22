# Steam Randomiser

This is CLI tool to select a game from your steam library.

To use you will need to get a Steam API key, [instructions can be found here](https://steamcommunity.com/dev).
You will also need your steam id which can be found following the [instructions here](https://help.steampowered.com/en/faqs/view/2816-BE67-5B69-0FEC).

The program is written in rust so you will need Rust installed to use it. There is no executable shared anywhere.

Simplest approach is to build/run it using `cargo`.

To generate an achievement use the following command:

```rust
cargo build
cargo run <KEY> <STEAM-ID> A game name
```

Replacing `<KEY> <STEAM-ID>` with the relevant items.

It will search for all games which match the title. 
If you give a title that you have multiple games that contain it, i.e. a series and giving the only the root name like Anno, then it will tell you it found multiple but take the first one found.
The order found is based off the API.

It will then randomly generate an achievement, and tell you how many you have left to achieve. 
It will give the achievement description as well, if it has one.

If the game has no achievements then it will not fail gracefully. Test with a game that has achievements first.

The Steam Web API can be flaky, there is no automatic retry, just run it again.

## Issues?

Raise an issue directly in the repo, happy to take a look! 

## Why Rust??

This is not a good fit for Rust, it is calling an API. I wanted to learn Rust, hence it not handling many corner cases. Any changes will be further experimentation.
