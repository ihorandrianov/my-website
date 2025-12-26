FROM rust:1.91.1 AS builder

WORKDIR /var/apps/mysite
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /var/apps/mysite/target/release/mysite /usr/local/bin/
COPY --from=builder /var/apps/mysite/static /static

EXPOSE 6500
CMD ["mysite"]
