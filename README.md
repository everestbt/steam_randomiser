# Steam Randomiser

This is CLI tool to select a game from your steam library.

To use you will need to get a Steam API key, [instructions can be found here](https://steamcommunity.com/dev).
You will also need your steam id which can be found following the [instructions here](https://help.steampowered.com/en/faqs/view/2816-BE67-5B69-0FEC).

The program is written in rust so you will need Rust installed to use it. There is no executable shared anywhere. 

You will need cargo to install it:
```bash
cargo install --path ./bin/cli
steam-rand --key <KEY> --steam-id <STEAM-ID> --random-achievement --game-name <GAME-NAME>
```

Replacing `<KEY> <STEAM-ID>` with your own credentials. It will save your credentials after the first use, if you want to replace them just provide new details. Note this may cause errors in the goals as it is only designed for one user.

For all commands use:

```bash
steam-rand --help
```

The Steam Web API can be flaky, there is no automatic retry, just run it again.
