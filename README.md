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
# set TAHTI_API_BASE + INTERNAL_SECRET, or DISCORD_CLIENT_ID + DISCORD_TOKEN
cargo run
```

| Variable            | Required | Description                                                                 |
| ------------------- | -------- | --------------------------------------------------------------------------- |
| `TAHTI_API_BASE`    | no       | Tahti API origin. With `INTERNAL_SECRET`, credentials are loaded from the API. |
| `INTERNAL_SECRET`   | no       | Shared secret for `GET /api/v1/internal/discord-bot/credentials`            |
| `DISCORD_CLIENT_ID` | yes*     | Application ID from General Information                                     |
| `DISCORD_TOKEN`     | yes*     | Bot token from the Bot page                                                 |

\*Required unless the bot successfully loads credentials from the Tahti API.

`.env.local` and `.env` are gitignored. The bot loads `.env.local` first, then `.env`.

## Deploy

Fly.io is not used. This is a long-running Discord Gateway process (ffmpeg + yt-dlp), not an HTTP service — do not fold it into the Fastify API. Run **one replica** next to the API (a second copy would play in duplicate).

On the Tahti API host:

```sh
cp .env.example .env
# set INTERNAL_SECRET to the same value the API uses
# if the bot shares the API compose network: TAHTI_API_BASE=http://api:3001
docker compose up -d --build
```

Push to `master` runs tests only. Board admins edit Client ID and token in Tahti Player → Settings → Add-ons → Radio.

## License

AGPL-3.0. See [LICENSE](LICENSE).
