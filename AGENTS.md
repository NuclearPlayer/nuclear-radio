# Nuclear Radio

A Discord bot that runs a 24/7 internet radio station. It joins a voice channel called "Nuclear Radio" in every guild it's added to and continuously plays music from a curated playlist of YouTube URLs.

Deployed on Fly.io (Amsterdam region). CI deploys on push to `master`.

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

### Audio pipeline

`Broadcast` is the core loop. It picks a random YouTube URL from the playlist, resolves it to a direct stream URL via yt-dlp, spawns ffmpeg to decode it to raw PCM, and writes the PCM bytes into an `AudioStream`.

`AudioStream` is a shared, bounded ring buffer that bridges the producer (Broadcast/ffmpeg) and consumers (sinks). It implements `Read`, `Write`, and Songbird's `MediaSource` trait. When the buffer is full, writers block. When it's empty, readers get silence (zero-filled buffers) instead of blocking, so playback never stalls.

### Sinks

A `Sink` is anything that consumes the audio stream. Currently there's only `DiscordSink`, but the abstraction exists to support additional outputs later (Icecast, HTTP streaming, etc.).

The `Runtime` owns all sinks, starts them, waits for SIGINT, then cleans them up.

### Discord sink

On `guild_create`, the bot finds or creates a voice channel named "Nuclear Radio", joins it via Songbird, and plays the shared `AudioStream` as a `RawAdapter` input.

The `Handler` struct implements Serenity's `EventHandler`. Event handlers are in `src/sinks/discord/handlers/`, one file per event.

### Data flow

```
tracks.txt (YouTube URLs)
    |
    v
Broadcast (picks random URL, loops forever)
    |
    v
yt-dlp (resolves to direct audio stream URL)
    |
    v
ffmpeg (decodes to f32le PCM, 48kHz, stereo)
    |
    v
AudioStream (shared ring buffer, 16MB)
    |
    v
Songbird RawAdapter -> Discord voice channel
```

## File map

```
src/
  main.rs           Entry point. Loads config, playlist, wires everything together.
  config.rs         Config struct deserialized from env vars (DISCORD_TOKEN, DISCORD_CLIENT_ID).
  playlist.rs       Loads tracks.txt (embedded at compile time via include_str!).
  source.rs         Calls yt-dlp to resolve a YouTube URL to a direct stream URL.
  decode.rs         Spawns ffmpeg as a child process, exposes stdout as a Read + MediaSource.
  audio_stream.rs   Shared ring buffer. The bridge between Broadcast and all sinks.
  broadcast.rs      The main playback loop. Picks tracks, resolves, decodes, writes to stream.
  runtime.rs        Owns sinks, manages startup and shutdown.
  sinks/
    mod.rs          Sink trait definition.
    discord/
      mod.rs        DiscordSink implementation. Creates the Serenity client, manages Songbird.
      handlers/
        mod.rs
        ready.rs    Logs successful login.
        guild_create.rs  Finds/creates voice channel, joins it, starts playing the stream.
tracks.txt          Playlist of YouTube URLs, one per line.
fly.toml            Fly.io deployment config.
Dockerfile          Multi-stage build: cargo-chef for caching, bookworm-slim runtime with ffmpeg + yt-dlp.
```

## Environment variables

| Variable            | Required | Description              |
| ------------------- | -------- | ------------------------ |
| `DISCORD_TOKEN`     | yes      | Bot token                |
| `DISCORD_CLIENT_ID` | yes      | Application/client ID    |

Loaded from `.env.local` first, then `.env`, then the actual environment. Both `.env` files are gitignored.

## Key patterns and conventions

- Handler functions live in their own files under `handlers/`, named after the event. The handler module re-exports them. The `EventHandler` impl in `discord/mod.rs` delegates to these functions.
- External processes (yt-dlp, ffmpeg) are spawned as child processes. `PcmSource` kills its ffmpeg child on drop.
- The codebase avoids `unwrap()` in fallible paths. `expect()` is used only for invariants ("Songbird should be registered at startup", "stdout was piped").
- No command handling. This is a headless radio, not an interactive bot. It just plays music.
- Tracing is used for logging, not `println!`. Log levels: `nuclear_radio=debug, songbird=debug, serenity=warn` by default, overridable via `RUST_LOG`.

## Building and running

```sh
cargo run
```

Requires `yt-dlp` and `ffmpeg` on PATH.

## Deployment

Push to `master` triggers a Fly.io deploy via GitHub Actions. The app runs on a single 256MB VM in Amsterdam. `--ha=false` disables high availability (only one instance should exist to avoid duplicate playback).
