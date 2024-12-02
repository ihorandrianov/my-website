FROM rust:1.75 as builder

WORKDIR /var/apps/mysite
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /var/apps/mysite/target/release/mysite /usr/local/bin/
COPY --from=builder /var/apps/mysite/static /static

EXPOSE 6500
CMD ["mysite"]
