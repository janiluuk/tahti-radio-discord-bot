# Tahti Radio

A Discord bot that runs a 24/7 internet radio station. It joins a voice channel called "Tahti Radio" in every guild it's added to and continuously plays music from a curated playlist of YouTube URLs. Users can queue tracks via the `/play` slash command.

Runs as the `radio-discord-bot` service in the Tahti stack (`infra/docker-compose.stack.yml`). CI on push to `master` runs tests and builds the Docker image.

Part of the `tahti-org` collection of repositories.

## Tech stack

- Rust
- Serenity 0.12 (Discord gateway)
- Songbird 0.6 (voice connection and audio playback)
- Symphonia (PCM codec support)
- Tokio (async runtime)
- yt-dlp (resolves YouTube URLs to direct audio stream URLs)
- ffmpeg (decodes audio to raw f32le PCM at 48kHz stereo)

## Architecture

The system has three layers: source resolution, audio pipeline, and output sinks.

### Source resolution

`ytdlp.rs` wraps the yt-dlp CLI. `source.rs` parses its output into domain types. `TrackMetadata` (no stream URL, used in queue and now-playing) and `Track` (metadata + stream URL, only created at play time) live in `track.rs`.

### Audio pipeline

`Broadcast` is the core playback loop. Each iteration pops from the user queue (FIFO) or falls back to a random playlist pick, resolves the URL to a stream via yt-dlp, decodes it with ffmpeg, and writes PCM into a shared ring buffer (`AudioStream`). The buffer gives silence to readers when empty, so playback never stalls.

Consumers access `Broadcast` through `subscribe()` (now-playing watch channel), `queue()`, and `stream()`.

### Sinks

A `Sink` consumes the audio stream. `Runtime` owns all sinks, starts them, and handles shutdown.

- `DiscordSink`: On `guild_create` it finds or creates a "Tahti Radio" voice channel, joins it, and feeds the shared `AudioStream` to Songbird. The bot's activity shows "Listening to Artist - Title".

### Slash commands

One file per command under `commands/`, routed by name match in `interaction.rs`. No command manager.

- `/play url:<string>`: responds immediately, resolves metadata in a background task, pushes to queue.

### Data flow

```
User /play command -----> Queue
                              |
                              v
tracks.txt (YouTube URLs) -> Broadcast (queue first, then random playlist pick)
                              |
                              v
                         yt-dlp -> ffmpeg -> AudioStream (ring buffer)
                              |
                              v
                         Songbird -> Discord voice channel
                              |
                         Bot activity: "Listening to Artist - Title"
```

## File map

```
src/
  main.rs           Entry point. Loads config, playlist, creates Broadcast and DiscordSink.
  config.rs         Loads credentials from the Tahti API (`TAHTI_API_BASE` + `INTERNAL_SECRET`) or env.
  playlist.rs       Loads tracks.txt (embedded at compile time via include_str!).
  track.rs          TrackMetadata and Track structs, parsing, Display impls.
  ytdlp.rs          yt-dlp CLI wrapper: fetch_metadata(), fetch_metadata_and_stream_url().
  source.rs         Parsing layer: resolve() -> Track, resolve_metadata() -> TrackMetadata.
  decode.rs         Spawns ffmpeg as a child process, exposes stdout as a Read + MediaSource.
  audio_stream.rs   Shared ring buffer. The bridge between Broadcast and all sinks.
  broadcast.rs      Playback loop. Owns now-playing watch channel and queue. Unit tested.
  runtime.rs        Owns sinks, manages startup and shutdown.
  sinks/
    mod.rs          Sink trait definition.
    discord/
      mod.rs        DiscordSink + Handler. Handler holds Arc<Broadcast>.
      commands/
        mod.rs      all() and register() for slash commands.
        play.rs     /play url:<str> handler.
      handlers/
        mod.rs      Re-exports handler functions.
        ready.rs    Login -> command registration -> activity sync spawn.
        activity.rs sync() loop + activity_for_track().
        guild_create.rs  Finds/creates voice channel, joins, starts playback.
        interaction.rs   Routes slash commands to handlers by name match.
tracks.txt          Playlist of YouTube URLs, one per line.
docker-compose.yml  Standalone single-replica Compose file (local/dev). Production uses the Tahti stack.
Dockerfile          Multi-stage build: cargo-chef 0.1.78 on Rust 1.97.1 bookworm, debian bookworm-slim runtime with ffmpeg + yt-dlp.
```

## Environment variables

| Variable            | Required | Description                                                                 |
| ------------------- | -------- | --------------------------------------------------------------------------- |
| `DISCORD_CLIENT_ID` | yes*     | Discord application / client ID                                             |
| `DISCORD_TOKEN`     | yes*     | Discord bot token                                                           |
| `TAHTI_API_BASE`    | no       | Tahti API origin (e.g. `https://api.tahti.live`). When set with `INTERNAL_SECRET`, credentials are loaded from the API. |
| `INTERNAL_SECRET`   | no       | Shared secret for `GET /api/v1/internal/discord-bot/credentials`            |

\*Required unless the bot successfully loads credentials from the Tahti API.

Loaded from `.env.local` first, then `.env`, then the actual environment. Both `.env` files are gitignored. Copy `.env.example` to `.env.local`.

Board admins edit the same Client ID and token in Tahti Player → Settings → Add-ons → Radio (Tahti Radio Discord bot). That write goes to `PUT /api/admin/discord-bot` and never returns the raw token afterward.

## Key patterns and conventions

- Handler functions live in their own files under `handlers/`, named after the event. The handler module re-exports them. The `EventHandler` impl in `discord/mod.rs` delegates to these functions.
- Commands follow the same pattern: one file per command under `commands/`, routed by a match in `interaction.rs`.
- External processes (yt-dlp, ffmpeg) are spawned as child processes. `PcmSource` kills its ffmpeg child on drop.
- `yt-dlp` CLI args live in `ytdlp.rs`; parsing of its output lives in `source.rs` and `track.rs`.
- The codebase avoids `unwrap()` in fallible paths. `expect()` is used only for invariants.
- Tracing is used for logging, not `println!`.
- `Broadcast` owns shared state internally. Consumers get handles via `subscribe()`, `queue()`, and `stream()`.
- `DiscordSink` receives `Arc<Broadcast>` rather than individual handles.

## Discord setup

1. Go to the [Discord Developer Portal](https://discord.com/developers/applications) and create a new application.
2. Under **Bot**, add a bot user, then copy the token — this is `DISCORD_TOKEN`.
3. Under **General Information**, copy the **Application ID** — this is `DISCORD_CLIENT_ID`.
4. Under **Bot**, enable the **Server Members Intent** if you plan to extend the bot with member-aware features; the current build only needs the `GUILDS` and `GUILD_VOICE_STATES` gateway intents, which don't require privileged-intent toggles.
5. Under **OAuth2 > URL Generator**, select scopes `bot` and `applications.commands`, and bot permissions:
   - View Channels
   - Manage Channels (the bot creates its own "Tahti Radio" voice channel if one doesn't exist)
   - Connect
   - Speak
6. Open the generated URL and invite the bot to your server.
7. Set `DISCORD_TOKEN` and `DISCORD_CLIENT_ID` in `.env.local` (or `.env`) — see [Environment variables](#environment-variables).
8. Run the bot (`cargo run`); it joins/creates the "Tahti Radio" voice channel in every guild it's in on `guild_create`.

## Building and running

```sh
cargo run
```

Requires `yt-dlp` and `ffmpeg` on PATH.

## Deployment

This bot is **not** Fly.io and **not** part of the Fastify API. It is a sibling
of the Tahti monorepo (`../tahti`) and ships as Compose service `radio-discord-bot`
in `tahti/infra/docker-compose.stack.yml` (one replica — a second copy joins
Discord twice and plays in duplicate).

- Production: from `../tahti`, `./scripts/deploy_prod.sh` rsyncs this checkout to
  `$DEPLOY_PATH/../tahti-radio-discord-bot` (default `/srv/tahti-radio-discord-bot`)
  and builds it with api / web / worker / orchestrator. The sibling must exist
  (or set `RADIO_DISCORD_BOT_SRC`).
- Local stack: `../tahti/scripts/stack-up.sh` starts this service when this repo
  is checked out next to `tahti`.
- Credentials: prefer `TAHTI_API_BASE` + `INTERNAL_SECRET` so the process loads
  `GET /api/v1/internal/discord-bot/credentials`. On the stack network that is
  `TAHTI_API_BASE=http://api:3001` and the same `INTERNAL_SECRET` as the API.
  Board admins edit Client ID and token in Tahti Player → Settings → Add-ons →
  Radio (`PUT /api/admin/discord-bot`).
- CI on push to `master`: `cargo test --locked` and a Docker image build (no
  deploy). Discord Interactions Endpoint URL and Linked Roles URL stay blank.
