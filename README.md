# Tahti Radio Discord bot

A Discord bot that joins a voice channel named **Tahti Radio** in every server it is added to and plays a 24/7 radio stream. Listeners can queue tracks with `/play`.

Source: [github.com/janiluuk/tahti-radio-discord-bot](https://github.com/janiluuk/tahti-radio-discord-bot). Part of the Tahti platform ([tahti.live](https://tahti.live)).

## Discord Developer Portal

Leave these blank — this bot uses the Gateway, not HTTP interaction endpoints:

- Interactions Endpoint URL
- Linked Roles Verification URL

Use Tahti’s public legal pages:

- Privacy Policy URL: https://tahti.live/privacy
- Terms of Service URL: https://tahti.live/terms

Invite the bot with scopes `bot` and `applications.commands`, and permissions **View Channels**, **Manage Channels**, **Connect**, and **Speak**.

## Run locally

Needs Rust 1.97.1 (see `rust-toolchain.toml`), `ffmpeg`, `yt-dlp`, and (for voice) `cmake` plus `libopus`.

```sh
cp .env.example .env.local
# set DISCORD_CLIENT_ID and DISCORD_TOKEN
cargo run
```

| Variable            | Required | Description                                      |
| ------------------- | -------- | ------------------------------------------------ |
| `DISCORD_CLIENT_ID` | yes      | Application ID from General Information          |
| `DISCORD_TOKEN`     | yes      | Bot token from the Bot page                      |

`.env.local` and `.env` are gitignored. The bot loads `.env.local` first, then `.env`.

## Deploy

The app runs on Fly.io (`tahti-radio`, Amsterdam, single 256MB VM). Push to `master` runs tests, then deploys when the `FLY_API_TOKEN` GitHub Actions secret is set. If that secret is missing, deploy is skipped and CI still passes.

```sh
gh secret set FLY_API_TOKEN --repo janiluuk/tahti-radio-discord-bot
```

Create the token with `fly tokens create deploy` while logged into the Fly org that owns `tahti-radio`. Without that secret, Tests still run and Deploy is skipped (CI stays green).

## License

AGPL-3.0. See [LICENSE](LICENSE).
