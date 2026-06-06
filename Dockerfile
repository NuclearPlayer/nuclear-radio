FROM rust:1-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin nuclear-radio

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ffmpeg \
    python3 \
    curl \
    ca-certificates \
    && curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o /usr/local/bin/yt-dlp \
    && chmod +x /usr/local/bin/yt-dlp \
    && apt-get purge -y curl \
    && apt-get autoremove -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/nuclear-radio /usr/local/bin/
ENTRYPOINT ["/usr/local/bin/nuclear-radio"]
